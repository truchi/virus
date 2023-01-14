pub mod builder;
pub mod meta;

use crate::{
    page::{Page, PageRef},
    segment::SegmentRef,
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Text {
    pub(crate) pages: Arc<Vec<Page>>,
    pub(crate) bytes: usize,
    pub(crate) lines: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct TextRef<'a> {
    pages: &'a [Page],
    bytes: usize,
    lines: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Selection<'a> {
    text: TextRef<'a>,
    start: CursorInner<'a>,
    end: CursorInner<'a>,
}

#[derive(Copy, Clone, Debug)]
pub struct Cursor<'a> {
    text: TextRef<'a>,
    inner: CursorInner<'a>,
}

#[derive(Copy, Clone, Debug)]
pub struct CursorInner<'a> {
    page: PageRef<'a>,
    page_index: usize,
    segment: SegmentRef<'a>,
    segment_offset: u16,
    byte: usize,
    line: usize,
    column: usize,
}
