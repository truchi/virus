use crate::{page::Page, Chunks, Cursor, Selection};
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

    pub fn cursor<I: CursorIndex>(&self, index: I) -> Cursor {
        self.as_ref().cursor(index)
    }

    pub fn selection<R: SelectionRange>(&self, range: R) -> Selection {
        self.as_ref().selection(range)
    }

    pub fn chunks(&self) -> Chunks {
        self.as_ref().chunks()
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

    pub fn cursor<I: CursorIndex>(&self, index: I) -> Cursor<'text> {
        index.cursor_from_text(*self)
    }

    pub fn selection<R: SelectionRange>(&self, range: R) -> Selection<'text> {
        range.selection(*self)
    }

    pub fn chunks(&self) -> Chunks<'text> {
        self.selection(..).chunks()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           CursorIndex                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait CursorIndex {
    fn cursor_from_text(self, text: TextRef) -> Cursor;
    fn cursor_from_cursor(self, cursor: Cursor) -> Cursor;
}

impl CursorIndex for usize {
    fn cursor_from_text(self, text: TextRef) -> Cursor {
        todo!()
    }

    fn cursor_from_cursor(self, cursor: Cursor) -> Cursor {
        todo!()
    }
}

impl CursorIndex for (usize, usize) {
    fn cursor_from_text(self, text: TextRef) -> Cursor {
        todo!()
    }

    fn cursor_from_cursor(self, cursor: Cursor) -> Cursor {
        todo!()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          SelectionRange                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait SelectionRange {
    fn selection(self, text: TextRef) -> Selection;
}

impl<T: CursorIndex> SelectionRange for T {
    fn selection(self, text: TextRef) -> Selection {
        todo!()
    }
}

impl<S: CursorIndex, E: CursorIndex> SelectionRange for (Bound<S>, Bound<E>) {
    fn selection(self, text: TextRef) -> Selection {
        todo!()
    }
}

impl<T: CursorIndex> SelectionRange for Range<T> {
    fn selection(self, text: TextRef) -> Selection {
        todo!()
    }
}

impl SelectionRange for RangeFull {
    fn selection(self, text: TextRef) -> Selection {
        todo!()
    }
}

// etc...
