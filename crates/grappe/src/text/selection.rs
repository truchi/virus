use crate::text::TextCursor;

use super::Line;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextSelection                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A selection in a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct TextSelection<'a> {
    lines: &'a [Line],
    start_offset: usize,
    start_row: usize,
    start_column: usize,
    end_offset: usize,
    end_row: usize,
    end_column: usize,
    len: usize,
}

impl<'a> TextSelection<'a> {
    pub fn start(&self) -> TextCursor<'a> {
        TextCursor {
            lines: self.lines,
            offset: self.start_offset,
            row: self.start_row,
            column: self.start_column,
            len: self.len,
        }
    }

    pub fn end(&self) -> TextCursor<'a> {
        TextCursor {
            lines: self.lines,
            offset: self.end_offset,
            row: self.end_row,
            column: self.end_column,
            len: self.len,
        }
    }

    pub fn inside(&self) -> (&'a str, bool, &'a [Line], &'a str) {
        debug_assert!(
            self.start_column
                <= self
                    .lines
                    .get(self.start_row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default()
                    .len()
        );
        debug_assert!(
            self.end_column
                <= self
                    .lines
                    .get(self.end_row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default()
                    .len()
        );

        if self.start_row == self.end_row {
            (
                self.lines
                    .get(self.start_row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default()
                    .get(self.start_column..self.end_column)
                    .unwrap_or_default(),
                false,
                &[],
                "",
            )
        } else {
            (
                self.lines
                    .get(self.start_row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default()
                    .get(self.start_column..)
                    .unwrap_or_default(),
                true,
                self.lines
                    .get(self.start_row + 1..self.end_row)
                    .unwrap_or_default(),
                self.lines
                    .get(self.end_row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default()
                    .get(..self.end_column)
                    .unwrap_or_default(),
            )
        }
    }
}
