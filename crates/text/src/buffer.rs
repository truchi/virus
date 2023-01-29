use crate::{
    info::Info,
    utils::{count_feeds, split_at, unchecked},
};
use std::{mem::size_of, sync::atomic::AtomicUsize};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Buffer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone)]
pub struct Buffer([u8; Self::CAPACITY]);

impl Buffer {
    pub const CAPACITY: usize = 1_024 - /* Arc */ 2 * size_of::<AtomicUsize>() - /* enum */ 1;

    pub fn new() -> Self {
        Self([0; Self::CAPACITY])
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", unsafe { unchecked(&self.0) }))
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             BufferRef                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct BufferRef<'buffer> {
    str: &'buffer str,
    feeds: usize,
}

impl<'buffer> BufferRef<'buffer> {
    pub unsafe fn from_buffer(buffer: &'buffer Buffer, info: Info) -> Self {
        let Info { bytes, feeds } = info;
        let str = unchecked(&buffer.0[..bytes]);

        debug_assert!(count_feeds(str) == feeds);
        Self { str, feeds }
    }

    pub fn as_str(&self) -> &'buffer str {
        self.str
    }

    pub fn bytes(&self) -> usize {
        self.str.len()
    }

    pub fn feeds(&self) -> usize {
        self.feeds
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             BufferMut                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct BufferMut<'buffer> {
    buffer: &'buffer mut Buffer,
    info: &'buffer mut Info,
}

impl<'buffer> BufferMut<'buffer> {
    pub unsafe fn from_buffer(buffer: &'buffer mut Buffer, info: &'buffer mut Info) -> Self {
        debug_assert!(
            if let Ok(str) = std::str::from_utf8(&buffer.0[..info.bytes]) {
                count_feeds(str) == info.feeds
            } else {
                false
            }
        );

        Self { buffer, info }
    }

    pub fn as_str(&self) -> &str {
        unsafe { unchecked(&self.buffer.0[..self.info.bytes]) }
    }

    pub fn bytes(&self) -> usize {
        self.info.bytes
    }

    pub fn feeds(&self) -> usize {
        self.info.feeds
    }

    pub fn as_ref(&self) -> BufferRef {
        BufferRef {
            str: self.as_str(),
            feeds: self.feeds(),
        }
    }

    pub fn push_str<'str>(&mut self, str: &'str str) -> &'str str {
        let bytes = self.bytes();
        let (s, rest) = split_at(str, Buffer::CAPACITY - bytes);

        self.buffer.0[bytes..][..s.len()].copy_from_slice(s.as_bytes());
        self.info.bytes += s.len();
        self.info.feeds += s.matches('\n').count();

        rest
    }

    pub fn push_char(&mut self, char: char) -> Option<char> {
        let bytes = self.bytes();
        let char_len = char.len_utf8();

        if bytes + char_len <= Buffer::CAPACITY {
            char.encode_utf8(&mut self.buffer.0[bytes..][..char_len]);
            self.info.bytes += char_len;
            self.info.feeds += (char == '\n') as usize;

            None
        } else {
            Some(char)
        }
    }
}
