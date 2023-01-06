use crate::eol::Eol;

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Meta {
    pub spaces: u8,
    pub len: u8,
    pub eol: Option<Eol>,
}

impl Meta {
    pub const MAX_SPACES: u8 = 63; // 6 bits
    pub const MAX_LEN: u8 = 255; // 8 bits
    pub const PREFIX_SIZE: usize = 2; // 2 bytes
    pub const SUFFIX_SIZE: usize = 1; // 1 byte
    pub const SIZE: usize = Self::PREFIX_SIZE + Self::SUFFIX_SIZE; // 3 bytes

    pub fn prefix(&self) -> [u8; 2] {
        debug_assert!(self.spaces <= Self::MAX_SPACES);

        [self.len, self.spaces | Eol::option_to_u8(self.eol) << 6]
    }

    pub fn suffix(&self) -> u8 {
        self.len
    }

    pub fn encode(&self, buffer: &mut [u8]) {
        let prefix = self.prefix();
        let suffix = self.suffix();

        buffer[0] = prefix[0];
        buffer[1] = prefix[1];
        buffer[2 + self.len as usize] = suffix;
    }
}
