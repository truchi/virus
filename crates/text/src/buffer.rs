use crate::{
    info::Info,
    utils::{count_feeds, split_at, unchecked},
};
use std::{mem::size_of, sync::atomic::AtomicUsize};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Buffer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             BufferRef                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct BufferRef<'buffer> {
    str: &'buffer str,
    feeds: usize,
}

impl<'buffer> BufferRef<'buffer> {
    pub unsafe fn from_buffer(buffer: &'buffer Buffer, bytes: usize, feeds: usize) -> Self {
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
    bytes: usize,
    feeds: usize,
}

impl<'buffer> BufferMut<'buffer> {
    pub unsafe fn from_buffer(buffer: &'buffer mut Buffer, bytes: usize, feeds: usize) -> Self {
        debug_assert!(if let Ok(str) = std::str::from_utf8(&buffer.0[..bytes]) {
            count_feeds(str) == feeds
        } else {
            false
        });

        Self {
            buffer,
            bytes,
            feeds,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { unchecked(&self.buffer.0[..self.bytes]) }
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn feeds(&self) -> usize {
        self.feeds
    }

    pub fn as_ref(&self) -> BufferRef {
        BufferRef {
            str: self.as_str(),
            feeds: self.feeds,
        }
    }

    pub fn push_str<'str>(&mut self, str: &'str str) -> (Info, &'str str) {
        let bytes = self.bytes();
        let (s, rest) = split_at(str, Buffer::CAPACITY - bytes);

        self.buffer.0[bytes..][..s.len()].copy_from_slice(s.as_bytes());
        let info = Info {
            bytes: s.len(),
            feeds: s.matches('\n').count(),
        };
        self.bytes += info.bytes;
        self.feeds += info.feeds;

        (info, rest)
    }

    pub fn push_char(&mut self, char: char) -> (Info, Option<char>) {
        let bytes = self.bytes();
        let char_len = char.len_utf8();

        if bytes + char_len <= Buffer::CAPACITY {
            char.encode_utf8(&mut self.buffer.0[bytes..][..char_len]);
            let info = Info {
                bytes: char_len,
                feeds: (char == '\n') as usize,
            };
            self.bytes += info.bytes;
            self.feeds += info.feeds;

            (info, None)
        } else {
            (Info::default(), Some(char))
        }
    }
}
