pub mod builder;
pub mod meta;

use crate::Offset;

use self::meta::PageMeta;
use std::{
    mem::size_of,
    sync::{atomic::AtomicUsize, Arc},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Page                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct Page {
    data: Arc<[u8; Self::BYTES]>,
    len: u16,
    sol: u16,
    bytes: u16,
    lines: u16,
    offset_bytes: usize,
    offset_lines: usize,
}

impl Page {
    pub const BYTES: usize = 1_024 - 2 * size_of::<AtomicUsize>();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             PageRef                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct PageRef<'a> {
    data: &'a [u8],
    bytes: u16,
    lines: u16,
    offset_bytes: usize,
    offset_lines: usize,
}
