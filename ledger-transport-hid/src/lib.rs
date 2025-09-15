mod errors;

use std::{io::Cursor, ops::Deref, sync::Mutex};

use byteorder::{BigEndian, ReadBytesExt};
pub use errors::LedgerHIDError;
pub use hidapi;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use ledger_transport::{async_trait, APDUAnswer, APDUCommand, Exchange};
use log::info;

pub const LEDGER_VENDOR_ID: u16 = 0x2c97;
pub const LEDGER_CHANNEL: u16 = 0x0101;
pub const LEDGER_USAGE_PAGE: u16 = 0xffa0;
// for Windows compatability, we prepend the buffer with a 0x00
// so the actual buffer is 64 bytes
pub const LEDGER_PACKET_WRITE_SIZE: u8 = 65;
pub const LEDGER_PACKET_READ_SIZE: u8 = 64;
pub const LEDGER_TIMEOUT: i32 = 10_000_000;

// USB Product IDs (Normal / Bootloader)
pub mod pid {
    pub const NANO_S_PLUS: u16 = 0x0050; // Identifiers: 0x50
    pub const NANO_S_PLUS_BL: u16 = 0x0005;

    pub const NANO_X: u16 = 0x0040; // Identifiers: 0x40
    pub const NANO_X_BL: u16 = 0x0004;

    pub const STAX: u16 = 0x0060; // Identifiers: 0x60
    pub const STAX_BL: u16 = 0x0006;

    pub const FLEX: u16 = 0x0070; // Identifiers: 0x70
    pub const FLEX_BL: u16 = 0x0007;
}

pub struct TransportNativeHID {
    device: Mutex<HidDevice>,
}

impl TransportNativeHID {
    fn is_ledger(dev: &DeviceInfo) -> bool {
        dev.vendor_id() == LEDGER_VENDOR_ID && dev.usage_page() == LEDGER_USAGE_PAGE
    }

    pub fn list_ledgers(api: &HidApi) -> impl Iterator<Item = &DeviceInfo> {
        api.device_list().filter(|dev| Self::is_ledger(dev))
    }

    pub fn open_device(api: &HidApi, device: &DeviceInfo) -> Result<Self, LedgerHIDError> {
        let device = device.open_device(api)?;
        let _ = device.set_blocking_mode(true);
        let ledger = TransportNativeHID {
            device: Mutex::new(device),
        };

        Ok(ledger)
    }

    pub fn new(api: &HidApi) -> Result<Self, LedgerHIDError> {
        let first_ledger = Self::list_ledgers(api)
            .next()
            .ok_or(LedgerHIDError::DeviceNotFound)?;

        Self::open_device(api, first_ledger)
    }

    fn write_apdu(
        device: &HidDevice,
        channel: u16,
        apdu_command: &[u8],
    ) -> Result<i32, LedgerHIDError> {
        let command_length = apdu_command.len();
        let mut in_data = Vec::with_capacity(command_length + 2);
        in_data.push(((command_length >> 8) & 0xFF) as u8);
        in_data.push((command_length & 0xFF) as u8);
        in_data.extend_from_slice(apdu_command);

        let mut buffer = vec![0u8; LEDGER_PACKET_WRITE_SIZE as usize];
        // Windows platform requires 0x00 prefix and Linux/Mac tolerate this as well
        buffer[0] = 0x00;
        buffer[1] = ((channel >> 8) & 0xFF) as u8;
        buffer[2] = (channel & 0xFF) as u8;
        buffer[3] = 0x05u8;

        for (idx, chunk) in in_data
            .chunks((LEDGER_PACKET_WRITE_SIZE - 6) as usize)
            .enumerate()
        {
            buffer[4] = ((idx >> 8) & 0xFF) as u8;
            buffer[5] = (idx & 0xFF) as u8;
            buffer[6..6 + chunk.len()].copy_from_slice(chunk);

            info!("[{:3}] << {:}", buffer.len(), hex::encode(&buffer));

            let result = device.write(&buffer);

            match result {
                Ok(size) => {
                    if size < buffer.len() {
                        return Err(LedgerHIDError::Comm(
                            "USB write error. Could not send whole message",
                        ));
                    }
                }
                Err(x) => return Err(LedgerHIDError::Hid(x)),
            }
        }

        Ok(1)
    }

    fn read_apdu(
        device: &HidDevice,
        channel: u16,
        apdu_answer: &mut Vec<u8>,
    ) -> Result<usize, LedgerHIDError> {
        let mut buffer: Vec<u8> = vec![0u8; LEDGER_PACKET_READ_SIZE as usize];
        let mut sequence_idx = 0u16;
        let mut expected_apdu_len = 0usize;

        loop {
            let res = device.read_timeout(&mut buffer, LEDGER_TIMEOUT)?;

            if (sequence_idx == 0 && res < 7) || res < 5 {
                return Err(LedgerHIDError::Comm("USB read error. Incomplete header"));
            }

            let mut rdr = Cursor::new(&buffer);

            let rcv_channel: u16 = rdr.read_u16::<BigEndian>()?;
            let rcv_tag: u8 = rdr.read_u8()?;
            let rcv_seq_idx: u16 = rdr.read_u16::<BigEndian>()?;

            if rcv_channel != channel {
                return Err(LedgerHIDError::Comm("Invalid channel"));
            }
            if rcv_tag != 0x05u8 {
                return Err(LedgerHIDError::Comm("Invalid tag"));
            }
            if rcv_seq_idx != sequence_idx {
                return Err(LedgerHIDError::Comm("Invalid sequence index"));
            }
            if rcv_seq_idx == 0 {
                expected_apdu_len = rdr.read_u16::<BigEndian>()? as usize;
            }

            let available: usize = buffer.len() - rdr.position() as usize;
            let missing: usize = expected_apdu_len - apdu_answer.len();
            let end_p = rdr.position() as usize + std::cmp::min(available, missing);

            let new_chunk = &buffer[rdr.position() as usize..end_p];

            info!("[{:3}] << {:}", new_chunk.len(), hex::encode(new_chunk));

            apdu_answer.extend_from_slice(new_chunk);

            if apdu_answer.len() >= expected_apdu_len {
                return Ok(apdu_answer.len());
            }

            sequence_idx += 1;
        }
    }

    pub fn exchange<I: Deref<Target = [u8]>>(
        &self,
        command: &APDUCommand<I>,
    ) -> Result<APDUAnswer<Vec<u8>>, LedgerHIDError> {
        let device = self.device.lock().expect("HID device poisoned");

        Self::write_apdu(&device, LEDGER_CHANNEL, &command.serialize())?;

        let mut answer = Vec::with_capacity(256);
        Self::read_apdu(&device, LEDGER_CHANNEL, &mut answer)?;

        APDUAnswer::from_answer(answer).map_err(|_| LedgerHIDError::Comm("response was too short"))
    }
}

#[async_trait]
impl Exchange for TransportNativeHID {
    type Error = LedgerHIDError;
    type AnswerType = Vec<u8>;

    async fn exchange<I>(
        &self,
        command: &APDUCommand<I>,
    ) -> Result<APDUAnswer<Self::AnswerType>, Self::Error>
    where
        I: Deref<Target = [u8]> + Send + Sync,
    {
        self.exchange(command)
    }
}
