use crate::rope::GraphemeCursor;
use ropey::Rope;
use std::{cell::Cell, cmp::Ordering};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Cursor                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `(index , line, column, width)` cursor.
///
/// Values for `line`, `column` and `width` are cached.
/// Since `width()` depends on `column()` and `column()` depends on `line()`,
/// you have to construct the cursor and call these functions with the same `Rope`
/// for results to be valid.
#[derive(Clone, Eq, Ord, Default, Debug)]
pub struct Cursor {
    index: usize,
    line: Cell<Option<usize>>,
    column: Cell<Option<usize>>,
    width: Cell<Option<usize>>,
}

impl Cursor {
    pub const ZERO: Self = Self::at_start();

    pub const fn new(
        index: usize,
        line: Option<usize>,
        column: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        Self {
            index,
            line: Cell::new(line),
            column: Cell::new(column),
            width: Cell::new(width),
        }
    }

    pub const fn at_start() -> Self {
        Self::new(0, Some(0), Some(0), Some(0))
    }

    pub fn at_end(rope: &Rope) -> Self {
        Self::new(rope.len_bytes(), None, None, None)
    }

    pub const fn at_index(index: usize) -> Self {
        Self::new(index, None, None, None)
    }

    pub fn at_line(line: usize, rope: &Rope) -> Self {
        debug_assert!(line < rope.len_lines());

        Self::new(rope.line_to_byte(line), Some(line), Some(0), None)
    }

    pub fn at_line_column(line: usize, column: usize, rope: &Rope) -> Self {
        debug_assert!(line < rope.len_lines());
        debug_assert!(column <= rope.line(line).to_string().trim_end_matches('\n').len());

        Self::new(
            rope.line_to_byte(line) + column,
            Some(line),
            Some(column),
            None,
        )
    }

    pub fn at_line_width(line: usize, width: usize, rope: &Rope) -> Self {
        debug_assert!(line < rope.len_lines());

        let (index, mut graphemes) = {
            let start = rope.line_to_byte(line);
            let end = rope.line_to_byte(line + 1);

            (start, GraphemeCursor::new(rope.byte_slice(start..end), 0))
        };
        let mut current_width = 0;
        let mut current_column = 0;

        while let Some((range, chunks)) = graphemes.next() {
            let grapheme_width = chunks.map(|(_, str)| str.width()).sum::<usize>();

            if grapheme_width == 0 {
                continue;
            }

            current_width += grapheme_width;

            match current_width.cmp(&width) {
                Ordering::Less => {
                    current_column = range.end;
                    continue;
                }
                Ordering::Equal => {
                    current_column = range.end;
                    break;
                }
                Ordering::Greater => {
                    current_width -= grapheme_width;
                    break;
                }
            }
        }

        Cursor::new(
            index + current_column,
            Some(line),
            Some(current_column),
            Some(current_width),
        )
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line<T: CursorArg>(&self, rope: T) -> T::Output {
        rope.line(self)
    }

    pub fn column<T: CursorArg>(&self, rope: T) -> T::Output {
        rope.column(self)
    }

    pub fn width<T: CursorArg>(&self, rope: T) -> T::Output {
        rope.width(self)
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           CursorArg                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait CursorArg {
    type Output;

    fn line(&self, cursor: &Cursor) -> Self::Output;

    fn column(&self, cursor: &Cursor) -> Self::Output;

    fn width(&self, cursor: &Cursor) -> Self::Output;
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl CursorArg for () {
    type Output = Option<usize>;

    fn line(&self, cursor: &Cursor) -> Self::Output {
        cursor.line.get()
    }

    fn column(&self, cursor: &Cursor) -> Self::Output {
        cursor.column.get()
    }

    fn width(&self, cursor: &Cursor) -> Self::Output {
        cursor.width.get()
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> CursorArg for &'a Rope {
    type Output = usize;

    fn line(&self, cursor: &Cursor) -> Self::Output {
        debug_assert!(cursor.index <= self.len_bytes());

        if let Some(cached) = cursor.line.get() {
            debug_assert!(cached == line(cursor, self));
            cached
        } else {
            let line = line(cursor, self);
            cursor.line.set(Some(line));
            line
        }
    }

    fn column(&self, cursor: &Cursor) -> Self::Output {
        debug_assert!(cursor.index <= self.len_bytes());

        if let Some(cached) = cursor.column.get() {
            debug_assert!(cached == column(cursor, self));
            cached
        } else {
            let column = column(cursor, self);
            cursor.column.set(Some(column));
            column
        }
    }

    fn width(&self, cursor: &Cursor) -> Self::Output {
        debug_assert!(cursor.index <= self.len_bytes());

        if let Some(cached) = cursor.width.get() {
            debug_assert!(cached == width(cursor, self));
            cached
        } else {
            let width = width(cursor, self);
            cursor.width.set(Some(width));
            width
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn line(cursor: &Cursor, rope: &Rope) -> usize {
    rope.byte_to_line(cursor.index)
}

fn column(cursor: &Cursor, rope: &Rope) -> usize {
    cursor.index - rope.line_to_byte(cursor.line(rope))
}

fn width(cursor: &Cursor, rope: &Rope) -> usize {
    rope.line(cursor.line(rope))
        .byte_slice(..cursor.column(rope))
        .chars()
        .map(|char| char.width().unwrap_or_default())
        .sum()
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_index_at_line_column_at_end() {
        let data: &[(&str, &[(usize, usize, usize, usize)])] = &[
            ("", &[(0, 0, 0, 0)]),
            ("\n", &[(0, 0, 0, 0), (1, 1, 0, 0)]),
            ("\n\n", &[(0, 0, 0, 0), (1, 1, 0, 0), (2, 2, 0, 0)]),
            (
                "\n\na",
                &[(0, 0, 0, 0), (1, 1, 0, 0), (2, 2, 0, 0), (3, 2, 1, 1)],
            ),
            (
                "\n\nab",
                &[
                    (0, 0, 0, 0),
                    (1, 1, 0, 0),
                    (2, 2, 0, 0),
                    (3, 2, 1, 1),
                    (4, 2, 2, 2),
                ],
            ),
            ("a", &[(0, 0, 0, 0), (1, 0, 1, 1)]),
            ("ab", &[(0, 0, 0, 0), (1, 0, 1, 1), (2, 0, 2, 2)]),
            ("a\r", &[(0, 0, 0, 0), (1, 0, 1, 1), (2, 1, 0, 0)]),
            ("a\n", &[(0, 0, 0, 0), (1, 0, 1, 1), (2, 1, 0, 0)]),
            (
                "a\r\n",
                &[(0, 0, 0, 0), (1, 0, 1, 1), (2, 0, 2, 1), (3, 1, 0, 0)],
            ),
            (
                "a\r\na",
                &[
                    (0, 0, 0, 0),
                    (1, 0, 1, 1),
                    (2, 0, 2, 1),
                    (3, 1, 0, 0),
                    (4, 1, 1, 1),
                ],
            ),
            (
                "a\r\na🦀d\n",
                &[
                    (0, 0, 0, 0),
                    (1, 0, 1, 1),
                    (2, 0, 2, 1),
                    (3, 1, 0, 0),
                    (4, 1, 1, 1),
                    (8, 1, 5, 3),
                    (9, 1, 6, 4),
                    (10, 2, 0, 0),
                ],
            ),
        ];

        for &(str, data) in data {
            let rope = Rope::from(str);

            assert!(data.len() == str.chars().count() + 1, "Wrong test data");

            for (index, line, column, width) in data.iter().copied() {
                for cursor in [
                    Cursor::at_index(index),
                    Cursor::at_line_column(line, column, &rope),
                ] {
                    assert!(cursor.index() == index);
                    assert!(cursor.line(&rope) == line);
                    assert!(cursor.column(&rope) == column);
                    assert!(cursor.width(&rope) == width);
                }

                if index == str.len() {
                    let cursor = Cursor::at_end(&rope);

                    assert!(cursor.index() == index);
                    assert!(cursor.line(&rope) == line);
                    assert!(cursor.column(&rope) == column);
                    assert!(cursor.width(&rope) == width);
                }
            }
        }
    }

    #[test]
    fn at_line() {
        let data: &[(&str, &[(usize, usize, usize, usize)])] = &[
            ("", &[(0, 0, 0, 0)]),
            ("\n", &[(0, 0, 0, 0), (1, 1, 0, 0)]),
            ("\n\n", &[(0, 0, 0, 0), (1, 1, 0, 0), (2, 2, 0, 0)]),
            ("a\r\na", &[(0, 0, 0, 0), (3, 1, 0, 0)]),
            ("a\r\na🦀b\n", &[(0, 0, 0, 0), (3, 1, 0, 0), (10, 2, 0, 0)]),
        ];

        for &(str, data) in data {
            let rope = Rope::from(str);

            assert!(data.len() == rope.len_lines(), "Wrong test data");

            for (i, (index, line, column, width)) in data.iter().copied().enumerate() {
                let cursor = Cursor::at_line(i, &rope);

                assert!(cursor.index() == index);
                assert!(cursor.line(&rope) == line);
                assert!(cursor.column(&rope) == column);
                assert!(cursor.width(&rope) == width);
            }
        }
    }

    #[test]
    fn at_line_width() {
        let rope = Rope::from("\0a\0🦀\0b\0");

        for (i, (index, line, column, width)) in [
            (0, 0, 0, 0),
            (2, 0, 2, 1),
            (2, 0, 2, 1),
            (7, 0, 7, 3),
            (9, 0, 9, 4),
        ]
        .into_iter()
        .enumerate()
        {
            let cursor = Cursor::at_line_width(line, i, &rope);

            assert!(cursor.index() == index);
            assert!(cursor.line(&rope) == line);
            assert!(cursor.column(&rope) == column);
            assert!(cursor.width(&rope) == width);
        }
    }
}
