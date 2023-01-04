use super::TextCursor;
use crate::line::{Line, LineCursor, LineNextChars};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextNextChars                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A fused double-ended iterator over the next chars of a [`TextCursor`].
///
/// See [`TextCursor::next_chars()`].
#[derive(Clone, Debug)]
pub struct TextNextChars<'a> {
    lines: &'a [Line],
    front_chars: LineNextChars<'a>,
    front_offset: usize,
    front_row: usize,
    back_chars: LineNextChars<'a>,
    back_offset: usize,
    back_row: usize,
    len: usize,
}

impl<'a> TextNextChars<'a> {
    /// Returns a new [`TextNextChars`] from `cursor`.
    pub fn new(cursor: TextCursor<'a>) -> Self {
        Self {
            lines: cursor.lines,
            front_chars: LineCursor::new_unchecked(
                cursor
                    .lines
                    .get(cursor.row())
                    .map(|line| line.as_ref())
                    .unwrap_or_default(),
                cursor.column(),
            )
            .next_chars(),
            front_offset: cursor.offset(),
            front_row: cursor.row(),
            back_chars: if let Some(line) = cursor.lines.last() {
                LineCursor::from_start(line)
            } else {
                LineCursor::from_empty()
            }
            .next_chars(),
            back_offset: 0,
            back_row: 0,
            len: cursor.len(),
        }
    }

    pub fn front(&self) -> TextCursor<'a> {
        TextCursor {
            lines: self.lines,
            offset: self.front_offset,
            row: self.front_row,
            column: self.front_chars.front(),
            len: self.len,
        }
    }

    pub fn back(&self) -> TextCursor<'a> {
        TextCursor {
            lines: self.lines,
            offset: self.back_offset,
            row: self.back_row,
            column: self.back_chars.back(),
            len: self.len,
        }
    }
}
