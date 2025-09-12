use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerHIDError {
    /// Device not found error
    #[error("Ledger device not found")]
    DeviceNotFound,
    /// Communication error
    #[error("Ledger device: communication error `{0}`")]
    Comm(&'static str),
    /// i/o error
    #[error("Ledger device: i/o error")]
    Io(#[from] std::io::Error),
    /// HID error
    #[error("Ledger device: Io error")]
    Hid(#[from] hidapi::HidError),
    /// UT8F error
    #[error("Ledger device: UTF8 error")]
    UTF8(#[from] std::str::Utf8Error),
}
