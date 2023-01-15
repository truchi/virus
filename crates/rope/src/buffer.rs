use crate::utils::split_at;
use std::{mem::size_of, sync::atomic::AtomicUsize};

pub const CAPACITY: usize = 1_024 - 2 * size_of::<AtomicUsize>();

pub type Bytes = [u8; CAPACITY];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             BufferMut                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct BufferMut<'a> {
    /// Raw bytes.
    pub(crate) buffer: &'a mut Bytes,
    /// Byte count.
    pub(crate) bytes: &'a mut u16,
    /// Feed count.
    pub(crate) feeds: &'a mut u16,
}

impl<'a> BufferMut<'a> {
    pub fn len(&self) -> usize {
        *self.bytes as usize
    }

    pub fn push_str<'str>(&mut self, str: &'str str) -> &'str str {
        let len = self.len();
        let (s, rest) = split_at(str, CAPACITY - len);

        self.buffer[len..][..s.len()].copy_from_slice(s.as_bytes());
        *self.bytes += s.len() as u16;
        *self.feeds += s.matches('\n').count() as u16;

        rest
    }

    pub fn push_char(&mut self, char: char) -> Option<char> {
        let len = self.len();
        let char_len = char.len_utf8();

        if len + char_len <= CAPACITY {
            char.encode_utf8(&mut self.buffer[len..][..char_len]);
            *self.bytes += char_len as u16;
            *self.feeds += (char == '\n') as u16;

            None
        } else {
            Some(char)
        }
    }
}
