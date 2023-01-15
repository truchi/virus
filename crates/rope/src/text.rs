use crate::{builder::Builder, page::Page, Chunks, Cursor, Selection};
use std::{
    ops::{Bound, Range, RangeFull},
    sync::Arc,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Text                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Thread-safe, structurally shared ***Text***.
#[derive(Clone, Default, Debug)]
pub struct Text {
    pub(crate) pages: Arc<Vec<Page>>,
    pub(crate) bytes: usize,
    pub(crate) lines: usize,
}

impl Text {
    /// Creates a new empty [`Text`].
    pub fn new() -> Self {
        Self {
            pages: Default::default(),
            bytes: 0,
            lines: 0,
        }
    }

    /// Creates a new empty [`Builder`].
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Returns a [`TextRef`] of `self`.
    pub fn as_ref(&self) -> TextRef {
        TextRef {
            pages: &self.pages,
            bytes: self.bytes,
            lines: self.lines,
        }
    }

    /// Returns `true` if `self` has a length of zero bytes.
    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    /// Returns the byte length of this [`Text`].
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Returns the count of `\n` in this [`Text`].
    pub fn lines(&self) -> usize {
        self.as_ref().lines()
    }

    /// Returns a [`Cursor`] at the start of this [`Text`].
    pub fn start(&self) -> Cursor {
        self.as_ref().start()
    }

    /// Returns a [`Cursor`] at the end of this [`Text`].
    pub fn end(&self) -> Cursor {
        self.as_ref().end()
    }

    /// Returns a [`Cursor`] at `index`.
    pub fn cursor<I: CursorIndex>(&self, index: I) -> Cursor {
        self.as_ref().cursor(index)
    }

    /// Returns a [`Selection`] at `range`.
    pub fn selection<R: SelectionRange>(&self, range: R) -> Selection {
        self.as_ref().selection(range)
    }

    /// Returns an iterator over the [`Chunk`](crate::Chunk)s of this [`Text`].
    pub fn chunks(&self) -> Chunks {
        self.as_ref().chunks()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             TextRef                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A reference to a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct TextRef<'text> {
    pages: &'text [Page],
    bytes: usize,
    lines: usize,
}

impl<'text> TextRef<'text> {
    /// Returns `true` if `self` has a length of zero bytes.
    pub fn is_empty(&self) -> bool {
        self.bytes == 0
    }

    /// Returns the byte length of this [`Text`].
    pub fn len(&self) -> usize {
        self.bytes
    }

    /// Returns the count of `\n` in this [`Text`].
    pub fn lines(&self) -> usize {
        self.lines
    }

    /// Returns a [`Cursor`] at the start of this [`Text`].
    pub fn start(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the end of this [`Text`].
    pub fn end(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at `index`.
    pub fn cursor<I: CursorIndex>(&self, index: I) -> Cursor<'text> {
        index.cursor_from_text(*self)
    }

    /// Returns a [`Selection`] at `range`.
    pub fn selection<R: SelectionRange>(&self, range: R) -> Selection<'text> {
        range.selection(*self)
    }

    /// Returns an iterator over the [`Chunk`](crate::Chunk)s of this [`Text`].
    pub fn chunks(&self) -> Chunks<'text> {
        self.selection(..).chunks()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           CursorIndex                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Cursoring operations.
pub trait CursorIndex {
    /// Returns a [`Cursor`] at this index from a [`TextRef`].
    fn cursor_from_text(self, text: TextRef) -> Cursor;

    /// Returns a [`Cursor`] at this index from another [`Cursor`].
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

/// Selecting operations.
pub trait SelectionRange {
    /// Returns a [`Selection`] at this range from a [`TextRef`].
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
