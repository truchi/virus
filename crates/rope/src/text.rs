use crate::{page::Page, Byte, Cursor, LineColumn, Selection};
use std::{
    ops::{Bound, Range, RangeFull},
    sync::Arc,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Text                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Default, Debug)]
pub struct Text {
    pages: Arc<Vec<Page>>,
    bytes: usize,
    lines: usize,
}

impl Text {
    pub fn new() -> Self {
        Self {
            pages: Default::default(),
            bytes: 0,
            lines: 0,
        }
    }

    pub fn as_ref(&self) -> TextRef {
        TextRef {
            pages: &self.pages,
            bytes: self.bytes,
            lines: self.lines,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn lines(&self) -> usize {
        self.as_ref().lines()
    }

    pub fn start(&self) -> Cursor {
        self.as_ref().start()
    }

    pub fn end(&self) -> Cursor {
        self.as_ref().end()
    }

    pub fn cursor<I: CursorIndex>(&self, index: I) -> Option<Cursor> {
        self.as_ref().cursor(index)
    }

    pub fn selection<R: SelectionRange>(&self, range: R) -> Option<Selection> {
        self.as_ref().selection(range)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             TextRef                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct TextRef<'text> {
    pages: &'text [Page],
    bytes: usize,
    lines: usize,
}

impl<'text> TextRef<'text> {
    pub fn is_empty(&self) -> bool {
        self.bytes == 0
    }

    pub fn len(&self) -> usize {
        self.bytes
    }

    pub fn lines(&self) -> usize {
        self.lines
    }

    pub fn start(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn end(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn cursor<I: CursorIndex>(&self, index: I) -> Option<Cursor<'text>> {
        index.cursor_from_text(*self)
    }

    pub fn selection<R: SelectionRange>(&self, range: R) -> Option<Selection<'text>> {
        range.selection(*self)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           CursorIndex                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait CursorIndex {
    fn cursor_from_text(self, text: TextRef) -> Option<Cursor>;
    fn cursor_from_cursor(self, cursor: Cursor) -> Option<Cursor>;
}

impl CursorIndex for Byte {
    fn cursor_from_text(self, text: TextRef) -> Option<Cursor> {
        todo!()
    }

    fn cursor_from_cursor(self, cursor: Cursor) -> Option<Cursor> {
        todo!()
    }
}

impl CursorIndex for LineColumn {
    fn cursor_from_text(self, text: TextRef) -> Option<Cursor> {
        todo!()
    }

    fn cursor_from_cursor(self, cursor: Cursor) -> Option<Cursor> {
        todo!()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          SelectionRange                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait SelectionRange {
    fn selection(self, text: TextRef) -> Option<Selection>;
}

impl<T: CursorIndex> SelectionRange for T {
    fn selection(self, text: TextRef) -> Option<Selection> {
        todo!()
    }
}

impl<S: CursorIndex, E: CursorIndex> SelectionRange for (Bound<S>, Bound<E>) {
    fn selection(self, text: TextRef) -> Option<Selection> {
        todo!()
    }
}

impl<T: CursorIndex> SelectionRange for Range<T> {
    fn selection(self, text: TextRef) -> Option<Selection> {
        todo!()
    }
}

impl SelectionRange for RangeFull {
    fn selection(self, text: TextRef) -> Option<Selection> {
        todo!()
    }
}

// etc...
