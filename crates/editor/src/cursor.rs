use crate::rope::GraphemeCursor;
use ropey::{Rope, WeakRope};
use std::{cell::Cell, cmp::Ordering};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A `(index , line, column, width)` cursor.
#[derive(Copy, Clone, Eq, Ord, Default, Debug)]
pub struct Cursor {
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub width: usize,
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
//                                          CachedCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `(index , line, column, width)` cursor.
///
/// Values for `line`, `column` and `width` are cached
/// and recomputed only if [`Rope::is_instance()`] is `false`.
#[derive(Default)]
pub struct CachedCursor {
    index: usize,
    line: Cell<Option<usize>>,
    column: Cell<Option<usize>>,
    width: Cell<Option<usize>>,
    rope: Cell<WeakRope>,
}

impl CachedCursor {
    pub fn at_start(rope: &Rope) -> Self {
        Self::new(0, Some(0), Some(0), Some(0), rope)
    }

    pub fn at_end(rope: &Rope) -> Self {
        Self::new(rope.len_bytes(), None, None, None, rope)
    }

    pub fn at_index(rope: &Rope, index: usize) -> Self {
        Self::new(index, None, None, None, rope)
    }

    pub fn at_line(rope: &Rope, line: usize) -> Self {
        debug_assert!(line < rope.len_lines());

        Self::new(rope.line_to_byte(line), Some(line), Some(0), None, rope)
    }

    pub fn at_line_column(rope: &Rope, line: usize, column: usize) -> Self {
        debug_assert!(line < rope.len_lines());
        debug_assert!(column <= rope.line(line).to_string().trim_end_matches('\n').len());

        Self::new(
            rope.line_to_byte(line) + column,
            Some(line),
            Some(column),
            None,
            rope,
        )
    }

    pub fn at_line_width(rope: &Rope, line: usize, width: usize) -> Self {
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

        Self::new(
            index + current_column,
            Some(line),
            Some(current_column),
            Some(current_width),
            rope,
        )
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line(&self, rope: &Rope) -> usize {
        let new_rope = Rope::downgrade(rope);
        let old_rope = self.rope.replace(new_rope.clone());
        let is_instance = new_rope.is_instance(&old_rope);

        if is_instance {
            debug_assert!(self.clone().debug_asserts());

            if let Some(line) = self.line.get() {
                return line;
            }
        }

        self.set_line(rope)
    }

    pub fn column(&self, rope: &Rope) -> usize {
        let new_rope = Rope::downgrade(rope);
        let old_rope = self.rope.replace(new_rope.clone());
        let is_instance = new_rope.is_instance(&old_rope);

        let line = if is_instance {
            debug_assert!(self.clone().debug_asserts());

            if let Some(column) = self.column.get() {
                return column;
            }

            self.get_or_set_line(rope)
        } else {
            self.set_line(rope)
        };

        self.set_column(rope, line)
    }

    pub fn width(&self, rope: &Rope) -> usize {
        let new_rope = Rope::downgrade(rope);
        let old_rope = self.rope.replace(new_rope.clone());
        let is_instance = new_rope.is_instance(&old_rope);

        let (line, column) = if is_instance {
            debug_assert!(self.clone().debug_asserts());

            if let Some(cached) = self.width.get() {
                return cached;
            }

            let line = self.get_or_set_line(rope);
            let column = self.get_or_set_column(rope, line);

            (line, column)
        } else {
            let line = self.set_line(rope);
            let column = self.set_column(rope, line);

            (line, column)
        };

        self.set_width(rope, line, column)
    }

    pub fn cursor(&self, rope: &Rope) -> Cursor {
        let new_rope = Rope::downgrade(rope);
        let old_rope = self.rope.replace(new_rope.clone());
        let is_instance = new_rope.is_instance(&old_rope);

        let (line, column, width) = if is_instance {
            debug_assert!(self.clone().debug_asserts());

            let line = self.get_or_set_line(rope);
            let column = self.get_or_set_column(rope, line);
            let width = self.get_or_set_width(rope, line, column);

            (line, column, width)
        } else {
            let line = self.set_line(rope);
            let column = self.set_column(rope, line);
            let width = self.set_width(rope, line, column);

            (line, column, width)
        };

        Cursor {
            index: self.index,
            line,
            column,
            width,
        }
    }
}

/// Private.
impl CachedCursor {
    fn new(
        index: usize,
        line: Option<usize>,
        column: Option<usize>,
        width: Option<usize>,
        rope: &Rope,
    ) -> Self {
        let cursor = Self {
            index,
            line: Cell::new(line),
            column: Cell::new(column),
            width: Cell::new(width),
            rope: Cell::new(Rope::downgrade(&rope)),
        };

        debug_assert!(cursor.clone().debug_asserts());
        cursor
    }

    fn set_line(&self, rope: &Rope) -> usize {
        let line = get_line(rope, self.index);
        self.line.set(Some(line));
        line
    }

    fn set_column(&self, rope: &Rope, line: usize) -> usize {
        let column = get_column(rope, self.index, line);
        self.column.set(Some(column));
        column
    }

    fn set_width(&self, rope: &Rope, line: usize, column: usize) -> usize {
        let width = get_width(rope, line, column);
        self.width.set(Some(width));
        width
    }

    fn get_or_set_line(&self, rope: &Rope) -> usize {
        if let Some(line) = self.line.get() {
            line
        } else {
            self.set_line(rope)
        }
    }

    fn get_or_set_column(&self, rope: &Rope, line: usize) -> usize {
        if let Some(column) = self.column.get() {
            column
        } else {
            self.set_column(rope, line)
        }
    }

    fn get_or_set_width(&self, rope: &Rope, line: usize, column: usize) -> usize {
        if let Some(width) = self.width.get() {
            width
        } else {
            self.set_width(rope, line, column)
        }
    }

    fn debug_asserts(self) -> bool {
        let rope = self.rope.take().upgrade().unwrap();

        debug_assert!(self.index <= rope.len_bytes());

        if let Some(line) = self.line.get() {
            debug_assert!(line == get_line(&rope, self.index));
        }

        if let Some(column) = self.column.get() {
            debug_assert!(column == get_column(&rope, self.index, self.line.get().unwrap()));
        }

        if let Some(width) = self.width.get() {
            debug_assert!(
                width == get_width(&rope, self.line.get().unwrap(), self.column.get().unwrap())
            );
        }

        true
    }
}

impl Clone for CachedCursor {
    fn clone(&self) -> Self {
        let rope = self.rope.take();

        // NOTE
        // Std does not allow this, but I don't understand why...
        // This might be an issue!
        self.rope.set(rope.clone());

        Self {
            rope: Cell::new(rope),
            index: self.index,
            line: self.line.clone(),
            column: self.column.clone(),
            width: self.width.clone(),
        }
    }
}

impl PartialEq for CachedCursor {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for CachedCursor {}

impl PartialOrd for CachedCursor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CachedCursor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl std::fmt::Debug for CachedCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unused)]
        #[derive(Debug)]
        struct Cursor {
            index: usize,
            line: Option<usize>,
            column: Option<usize>,
            width: Option<usize>,
        }

        Cursor::fmt(
            &Cursor {
                index: self.index,
                line: self.line.get(),
                column: self.column.get(),
                width: self.width.get(),
            },
            f,
        )
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn get_line(rope: &Rope, index: usize) -> usize {
    rope.byte_to_line(index)
}

fn get_column(rope: &Rope, index: usize, line: usize) -> usize {
    index - rope.line_to_byte(line)
}

fn get_width(rope: &Rope, line: usize, column: usize) -> usize {
    rope.line(line)
        .byte_slice(..column)
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
                    CachedCursor::at_index(&rope, index),
                    CachedCursor::at_line_column(&rope, line, column),
                ] {
                    assert!(cursor.index() == index);
                    assert!(cursor.line(&rope) == line);
                    assert!(cursor.column(&rope) == column);
                    assert!(cursor.width(&rope) == width);
                }

                if index == str.len() {
                    let cursor = CachedCursor::at_end(&rope);

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
                let cursor = CachedCursor::at_line(&rope, i);

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
            let cursor = CachedCursor::at_line_width(&rope, line, i);

            assert!(cursor.index() == index);
            assert!(cursor.line(&rope) == line);
            assert!(cursor.column(&rope) == column);
            assert!(cursor.width(&rope) == width);
        }
    }
}
