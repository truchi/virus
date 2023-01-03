use crate::{
    line::{Line, LineCursor, LineNextChars, LinePrevChars},
    Index,
};
use std::sync::Arc;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Text                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An immutable, thread-safe [`String`].
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Text {
    lines: Arc<Vec<Line>>,
    len: usize,
}

impl Text {
    /// Creates a new [`Text`] from `lines` and `len`.
    pub fn new(lines: Arc<Vec<Line>>, len: usize) -> Self {
        Self { lines, len }
    }

    /// Returns the byte length of this [`Text`].
    pub fn len(&self) -> usize {
        self.lines.iter().map(Line::len).sum()
    }

    /// Returns `true` if this [`Text`] has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        // Lines are never empty
        self.lines.len() == 0
    }

    /// Returns the [`Line`]s of this [`Text`].
    pub fn lines(&self) -> &Arc<Vec<Line>> {
        &self.lines
    }

    /// Gets the strong count to the [`lines`](Self::lines).
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.lines)
    }

    /// Gets the weak count to the [`lines`](Self::lines).
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.lines)
    }

    /// Makes a mutable reference into this [`Text`].
    pub fn make_mut(&mut self) -> &mut Vec<Line> {
        Arc::make_mut(&mut self.lines)
    }
}

impl From<&str> for Text {
    fn from(str: &str) -> Self {
        Self {
            lines: Arc::new(str.lines().map(Into::into).collect()),
            len: str.len(),
        }
    }
}

impl AsRef<[Line]> for Text {
    fn as_ref(&self) -> &[Line] {
        &self.lines
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.lines.iter() {
            std::fmt::Display::fmt(line, f)?;
        }

        Ok(())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            TextCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct TextCursor<'a> {
    lines: &'a [Line],
    line_cursor: LineCursor<'a>,
    offset: usize,
    row: usize,
    len: usize,
}

impl<'a> TextCursor<'a> {
    pub fn from_start(text: &'a Text) -> Self {
        let lines = text.lines();
        let line_cursor = if let Some(line) = lines.first() {
            LineCursor::from_start(line)
        } else {
            LineCursor {
                string: "",
                offset: 0,
            }
        };

        Self {
            lines,
            line_cursor,
            offset: 0,
            row: 0,
            len: text.len(),
        }
    }

    pub fn from_end(text: &'a Text) -> Self {
        let lines = text.lines();
        let line_cursor = LineCursor {
            string: "",
            offset: 0,
        };

        Self {
            lines,
            line_cursor,
            offset: text.len(),
            row: lines.len(),
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
        self.line_cursor.offset()
    }

    pub fn index(&self) -> Index {
        Index {
            offset: self.offset,
            row: self.row,
            column: self.line_cursor.offset(),
        }
    }

    /// Returns a fused double-ended iterator over the previous chars in the text.
    ///
    /// ```
    /// ```
    pub fn prev_chars(&self) -> TextPrevChars<'a> {
        TextPrevChars::new(*self)
    }

    pub fn start(&mut self) {
        self.line_cursor = if let Some(line) = self.lines.first() {
            LineCursor::from_start(line)
        } else {
            LineCursor {
                string: "",
                offset: 0,
            }
        };
        self.offset = 0;
        self.row = 0;
    }

    pub fn end(&mut self) {
        self.line_cursor = LineCursor {
            string: "",
            offset: 0,
        };
        self.offset = self.len;
        self.row = self.lines.len();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPrevChars                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A fused double-ended iterator over the previous chars of a [`TextCursor`].
///
/// See [`TextCursor::prev_chars()`].
pub struct TextPrevChars<'a> {
    lines: &'a [Line],
    iter: std::iter::Rev<std::slice::Iter<'a, Line>>,
    front_line_prev_chars: LinePrevChars<'a>,
    front_offset: usize,
    front_row: usize,
    back_line_next_chars: LineNextChars<'a>,
    back_offset: usize,
    back_row: usize,
    len: usize,
}

impl<'a> TextPrevChars<'a> {
    /// Returns a new [`TextPrevChars`] from `cursor`.
    pub fn new(cursor: TextCursor<'a>) -> Self {
        let lines = cursor.lines;
        let mut iter = lines[..cursor.row].iter().rev();
        // iter.next_back();

        Self {
            lines,
            iter,
            front_line_prev_chars: cursor.line_cursor.prev_chars(),
            front_offset: cursor.offset,
            front_row: cursor.row,
            back_line_next_chars: {
                let mut cursor = cursor;
                cursor.start();
                cursor
            }
            .line_cursor
            .next_chars(),
            back_offset: 0,
            back_row: 0,
            len: cursor.len,
        }
    }

    pub fn front_offset(&self) -> usize {
        self.front_offset
    }

    pub fn front_row(&self) -> usize {
        self.front_row
    }

    pub fn front_column(&self) -> usize {
        self.front_line_prev_chars.front()
    }

    pub fn front(&self) -> Index {
        Index {
            offset: self.front_offset,
            row: self.front_row,
            column: self.front_line_prev_chars.front(),
        }
    }

    pub fn back_offset(&self) -> usize {
        self.back_offset
    }

    pub fn back_row(&self) -> usize {
        self.back_row
    }

    pub fn back_column(&self) -> usize {
        self.back_line_next_chars.front()
    }

    pub fn back(&self) -> Index {
        Index {
            offset: self.back_offset,
            row: self.back_row,
            column: self.back_line_next_chars.front(),
        }
    }
}

impl<'a> Iterator for TextPrevChars<'a> {
    type Item = (TextCursor<'a>, char);

    fn next(&mut self) -> Option<Self::Item> {
        let (line_cursor, char) = match self.front_line_prev_chars.next() {
            Some((line_cursor, char)) => (line_cursor, char),
            None => {
                self.front_line_prev_chars = LineCursor::from_end(self.iter.next()?).prev_chars();
                self.front_row -= 1;
                self.front_line_prev_chars.next()?
            }
        };

        self.front_offset -= char.len_utf8();
        Some((
            TextCursor {
                lines: self.lines,
                line_cursor,
                offset: self.front_offset,
                row: self.front_row,
                len: self.len,
            },
            char,
        ))
    }
}

impl<'a> DoubleEndedIterator for TextPrevChars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let offset = self.back_offset;
        let row = self.back_row;
        // let line_cursor_front = self.back_line_next_chars.front();

        // let line_cursor = LineCursor {
        //     string: self.back_line_next_chars.string,
        //     offset: self.back_line_next_chars.front(),
        // };

        let (line_cursor, char) = match self.back_line_next_chars.next() {
            Some((line_cursor, char)) => (line_cursor, char),
            None => {
                self.back_line_next_chars =
                    LineCursor::from_start(self.iter.next_back()?).next_chars();
                self.back_row += 1;
                self.back_line_next_chars.next()?
            }
        };

        self.back_offset += char.len_utf8();
        Some((
            TextCursor {
                lines: self.lines,
                line_cursor,
                offset,
                row: self.back_row,
                len: self.len,
            },
            char,
        ))
    }
}
