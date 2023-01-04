use super::{Text, TextPrevChars};
use crate::{line::Line, Index};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            TextCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct TextCursor<'a> {
    pub(crate) lines: &'a [Line],
    pub(crate) offset: usize,
    pub(crate) row: usize,
    pub(crate) column: usize,
    pub(crate) len: usize,
}

impl<'a> TextCursor<'a> {
    pub fn from_start(text: &'a Text) -> Self {
        Self {
            lines: text.lines(),
            offset: 0,
            row: 0,
            column: 0,
            len: text.len(),
        }
    }

    pub fn from_end(text: &'a Text) -> Self {
        Self {
            lines: text.lines(),
            offset: text.len(),
            row: text.lines.len(),
            column: 0,
            len: text.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn index(&self) -> Index {
        Index {
            offset: self.offset,
            row: self.row,
            column: self.column,
        }
    }

    /// Returns a fused double-ended iterator over the previous chars in the text.
    pub fn prev_chars(&self) -> TextPrevChars<'a> {
        TextPrevChars::new(*self)
    }

    pub fn start(&mut self) {
        self.offset = 0;
        self.row = 0;
        self.column = 0;
    }

    pub fn end(&mut self) {
        self.offset = self.len;
        self.row = self.lines.len();
        self.column = 0;
    }
}
