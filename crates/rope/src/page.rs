use crate::{buffer::Bytes, utils::utf8};
use std::sync::Arc;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Page                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct Page {
    /// Raw bytes.
    pub(crate) buffer: Arc<Bytes>,
    /// Byte count.
    pub(crate) bytes: u16,
    /// Feed count.
    pub(crate) feeds: u16,
    /// Byte offset.
    pub(crate) byte: usize,
    /// Feed offset.
    pub(crate) feed: usize,
}

impl Page {
    pub fn as_ref(&self) -> PageRef {
        PageRef {
            buffer: &self.buffer,
            bytes: self.bytes,
            feeds: self.feeds,
            byte: self.byte,
            feed: self.feed,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             PageRef                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct PageRef<'page> {
    /// Raw bytes.
    pub(crate) buffer: &'page Bytes,
    /// Byte count.
    pub(crate) bytes: u16,
    /// Feed count.
    pub(crate) feeds: u16,
    /// Byte offset.
    pub(crate) byte: usize,
    /// Feed offset.
    pub(crate) feed: usize,
}

impl<'page> PageRef<'page> {
    pub fn as_str(&self) -> &'page str {
        unsafe { utf8(&self.buffer[..self.bytes as usize]) }
    }
}
