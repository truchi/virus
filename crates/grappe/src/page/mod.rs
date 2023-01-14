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
    pub(crate) data: Arc<[u8; Self::LEN]>,
    pub(crate) len: u16,
    pub(crate) sol: u16,
    pub(crate) bytes: u16,
    pub(crate) lines: u16,
    pub(crate) offset_bytes: usize,
    pub(crate) offset_lines: usize,
}

impl Page {
    pub const LEN: usize = 1_024 - 2 * size_of::<AtomicUsize>();
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
