use crate::{buffer::Buffer, utils::utf8};
use std::sync::Arc;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Page                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct Page {
    buffer: Arc<Buffer>,
    bytes: u16,
    lines: u16,
    byte: usize,
    line: usize,
}

impl Page {
    pub fn as_ref(&self) -> PageRef {
        PageRef {
            buffer: &self.buffer,
            bytes: self.bytes,
            lines: self.lines,
            byte: self.byte,
            line: self.line,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             PageRef                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct PageRef<'page> {
    buffer: &'page Buffer,
    bytes: u16,
    lines: u16,
    byte: usize,
    line: usize,
}

impl<'page> PageRef<'page> {
    pub fn as_str(&self) -> &'page str {
        unsafe { utf8(&self.buffer[..self.bytes as usize]) }
    }
}
