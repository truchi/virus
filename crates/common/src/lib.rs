use tree_sitter::{InputEdit, Point};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Cursor                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `(index , line, column)` cursor.
#[derive(Copy, Clone, Eq, Ord, Default, Debug)]
pub struct Cursor {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl Cursor {
    pub const ZERO: Self = Self {
        index: 0,
        line: 0,
        column: 0,
    };

    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Self {
            index,
            line,
            column,
        }
    }

    pub fn into_input_edit(start: Self, old_end: Self, new_end: Self) -> InputEdit {
        InputEdit {
            start_byte: start.index,
            old_end_byte: old_end.index,
            new_end_byte: new_end.index,
            start_position: start.into(),
            old_end_position: old_end.into(),
            new_end_position: new_end.into(),
        }
    }

    pub fn from_input_edit(
        input_edit: InputEdit,
    ) -> (
        Self, // Start
        Self, // Old end
        Self, // New end
    ) {
        (
            Self::new(
                input_edit.start_byte,
                input_edit.start_position.row,
                input_edit.start_position.column,
            ),
            Self::new(
                input_edit.old_end_byte,
                input_edit.old_end_position.row,
                input_edit.old_end_position.column,
            ),
            Self::new(
                input_edit.new_end_byte,
                input_edit.new_end_position.row,
                input_edit.new_end_position.column,
            ),
        )
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        let index = self.index == other.index;

        debug_assert!(if index {
            self.line == other.line && self.column == other.column
        } else {
            self.line != other.line || self.column != other.column
        });

        index
    }
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let index = self.index.cmp(&other.index);

        debug_assert!({
            use std::cmp::Ordering::*;

            let line = self.line.cmp(&other.line);
            let column = self.column.cmp(&other.column);

            match index {
                Less => match line {
                    Less => true,
                    Equal => column == Less,
                    Greater => false,
                },
                Equal => line == Equal && column == Equal,
                Greater => match line {
                    Less => false,
                    Equal => column == Greater,
                    Greater => true,
                },
            }
        });

        Some(index)
    }
}

impl From<Cursor> for Point {
    fn from(cursor: Cursor) -> Self {
        Self {
            row: cursor.line,
            column: cursor.column,
        }
    }
}
