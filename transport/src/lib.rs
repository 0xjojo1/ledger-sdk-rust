use apdu::ApduCommand;

pub trait Transport {
    fn exchange(&self, command: &ApduCommand) -> Vec<u8>;
}

pub struct MockTransport;

impl Transport for MockTransport {
    fn exchange(&self, command: &ApduCommand) -> Vec<u8> {
        // For now just return success status word
        let _ = command.to_bytes();
        vec![0x90, 0x00]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apdu::ApduCommand;

    #[test]
    fn mock_transport_returns_success() {
        let transport = MockTransport;
        let cmd = ApduCommand::new(0x00, 0x00, 0x00, 0x00, vec![]);
        let resp = transport.exchange(&cmd);
        assert_eq!(resp, vec![0x90, 0x00]);
    }
}
