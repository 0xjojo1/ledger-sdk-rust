#[derive(Debug)]
pub struct ApduCommand {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
}

impl ApduCommand {
    pub fn new(cla: u8, ins: u8, p1: u8, p2: u8, data: Vec<u8>) -> Self {
        Self { cla, ins, p1, p2, data }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.cla, self.ins, self.p1, self.p2, self.data.len() as u8];
        bytes.extend(&self.data);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_command() {
        let cmd = ApduCommand::new(0xE0, 0x02, 0x00, 0x00, vec![1, 2, 3]);
        assert_eq!(cmd.to_bytes(), vec![0xE0, 0x02, 0x00, 0x00, 3, 1, 2, 3]);
    }
}
