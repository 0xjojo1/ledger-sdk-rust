use apdu::ApduCommand;
use transport::Transport;

pub struct EthApp<T: Transport> {
    transport: T,
}

impl<T: Transport> EthApp<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn get_address(&self) -> Vec<u8> {
        let cmd = ApduCommand::new(0xE0, 0x02, 0x00, 0x00, vec![]);
        self.transport.exchange(&cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport::MockTransport;

    #[test]
    fn retrieves_address() {
        let app = EthApp::new(MockTransport);
        let response = app.get_address();
        assert_eq!(response, vec![0x90, 0x00]);
    }
}
