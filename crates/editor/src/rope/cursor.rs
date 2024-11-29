use crate::rope::{GraphemeCursor, GraphemesForward, WordClass, WordCursor};
use ropey::RopeSlice;
use std::{cell::Cell, cmp::Ordering};
use unicode_width::UnicodeWidthStr;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Cursor                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An `(index , line, column, width)` cursor.
#[derive(Copy, Clone, Eq, Ord, Default, Debug)]
pub struct Cursor {
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub width: usize,
}

impl Cursor {
    pub fn builder(slice: RopeSlice) -> CursorBuilder {
        CursorBuilder { slice }
    }

    pub fn as_cursor_ref<'rope>(&self, rope: RopeSlice<'rope>) -> CursorRef<'rope> {
        CursorRef::with(rope).at_cursor(*self)
    }

    pub fn edit(&self, start: Self, removed_end: Self, inserted_end: Self) -> Option<Self> {
        if self.index <= start.index {
            return Some(*self);
        }

        if start.index < self.index && self.index < removed_end.index {
            return None;
        }

        debug_assert!(removed_end.line <= self.line);

        let (index, line) = (
            self.index - (removed_end.index - start.index) + (inserted_end.index - start.index),
            self.line - (removed_end.line - start.line) + (inserted_end.line - start.line),
        );

        if removed_end.line < self.line {
            return Some(Self {
                index,
                line,
                column: self.column,
                width: self.width,
            });
        }

        debug_assert!(removed_end.line == self.line);

        let (column, width) = (
            self.column - removed_end.column + inserted_end.column,
            self.width - removed_end.width + inserted_end.width,
        );

        Some(Self {
            index,
            line,
            column,
            width,
        })
    }

    /// Extracts multiple `┃` as cursors in the rope.
    ///
    /// (`┣`, `┫`)
    #[cfg(test)]
    pub(crate) fn extract(mut str: &str) -> (ropey::Rope, Vec<Self>) {
        let mut cursors = Vec::new();
        let mut rope = ropey::Rope::from("");

        loop {
            if let Some((before, after)) = str.split_once('┃') {
                rope.append(ropey::Rope::from(before));

                if !(before.ends_with('\r') && after.starts_with('\n')) {
                    cursors.push({
                        let line = rope.line(rope.len_lines() - 1);
                        Self {
                            index: rope.len_bytes(),
                            line: rope.len_lines() - 1,
                            column: line.len_bytes(),
                            width: line.chunks().map(str::width).sum(),
                        }
                    });
                }
                str = after;
            } else {
                break;
            }
        }

        rope.append(ropey::Rope::from(str));

        (rope, cursors)
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
//                                           CursorRef                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// TODO REMOVE

/// An `(index , line, column, width)` cursor with a `Rope` reference for cache.
#[derive(Clone, Eq, Ord)]
pub struct CursorRef<'rope> {
    rope: RopeSlice<'rope>,
    index: usize,
    line: Cell<Option<usize>>,
    column: Cell<Option<usize>>,
    width: Cell<Option<usize>>,
}

impl<'rope> CursorRef<'rope> {
    pub fn with(rope: RopeSlice<'rope>) -> CursorRefBuilder<'rope> {
        CursorRefBuilder { rope }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line(&self) -> usize {
        if let Some(line) = self.line.get() {
            line
        } else {
            let line = utils::line(self.rope, self.index);
            self.line.set(Some(line));
            line
        }
    }

    pub fn column(&self) -> usize {
        if let Some(column) = self.column.get() {
            column
        } else {
            let column = utils::column(self.rope, self.index, self.line());
            self.column.set(Some(column));
            column
        }
    }

    pub fn width(&self) -> usize {
        if let Some(width) = self.width.get() {
            width
        } else {
            let width = utils::width(self.rope, self.line(), self.column());
            self.width.set(Some(width));
            width
        }
    }

    /// Returns whether on a grapheme boundary.
    pub fn is_grapheme_boundary(&self) -> bool {
        GraphemeCursor::new(self.rope, self.index).is_boundary()
    }

    /// Finds the previous grapheme boundary.
    pub fn prev_grapheme(&self) -> Option<Self> {
        GraphemeCursor::new(self.rope, self.index)
            .prev()
            .map(|(range, _)| range.start)
            .map(|index| Self::with(self.rope).at_index(index))
    }

    /// Finds the next grapheme boundary.
    pub fn next_grapheme(&self) -> Option<Self> {
        GraphemeCursor::new(self.rope, self.index)
            .next()
            .map(|(range, _)| range.end)
            .map(|index| Self::with(self.rope).at_index(index))
    }

    /// Finds the previous start of word.
    pub fn prev_start_of_subword(&self) -> Option<Self> {
        let mut words = WordCursor::new(self.rope, self.index);

        words
            .prev()
            .map(|(range, class)| match class {
                WordClass::Whitespace => words.prev().map(|(range, _)| range.start).unwrap_or(0),
                _ => range.start,
            })
            .map(|index| Self::with(self.rope).at_index(index))
    }

    /// Finds the previous end of word.
    pub fn prev_end_of_subword(&self) -> Option<Self> {
        let mut words = WordCursor::new(self.rope, self.index);

        words
            .prev()
            .map(|(range, class)| match class {
                WordClass::Whitespace => range.start,
                _ => words
                    .prev()
                    .map(|(range, class)| match class {
                        WordClass::Whitespace => range.start,
                        _ => range.end,
                    })
                    .unwrap_or(0),
            })
            .map(|index| Self::with(self.rope).at_index(index))
    }

    /// Finds the next start of word.
    pub fn next_start_of_subword(&self) -> Option<Self> {
        let mut words = WordCursor::new(self.rope, self.index);

        words
            .next()
            .map(|(range, class)| match class {
                WordClass::Whitespace => range.end,
                _ => words
                    .next()
                    .map(|(range, class)| match class {
                        WordClass::Whitespace => range.end,
                        _ => range.start,
                    })
                    .unwrap_or_else(|| self.rope.len_bytes()),
            })
            .map(|index| Self::with(self.rope).at_index(index))
    }

    /// Finds the next end of word.
    pub fn next_end_of_subword(&self) -> Option<Self> {
        let mut words = WordCursor::new(self.rope, self.index);

        words
            .next()
            .map(|(range, class)| match class {
                WordClass::Whitespace => words
                    .next()
                    .map(|(range, _)| range.end)
                    .unwrap_or_else(|| self.rope.len_bytes()),
                _ => range.end,
            })
            .map(|index| Self::with(self.rope).at_index(index))
    }

    pub fn as_cursor(&self) -> Cursor {
        Cursor {
            index: self.index,
            line: self.line(),
            column: self.column(),
            width: self.width(),
        }
    }
}

impl<'rope> PartialEq for CursorRef<'rope> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<'rope> PartialOrd for CursorRef<'rope> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl<'rope> std::fmt::Debug for CursorRef<'rope> {
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        CursorRefBuilder                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone)]
pub struct CursorRefBuilder<'rope> {
    rope: RopeSlice<'rope>,
}

impl<'rope> CursorRefBuilder<'rope> {
    pub fn at_start(&self) -> CursorRef<'rope> {
        CursorRef {
            rope: self.rope,
            index: 0,
            line: Cell::new(Some(0)),
            column: Cell::new(Some(0)),
            width: Cell::new(Some(0)),
        }
    }

    pub fn at_end(&self) -> CursorRef<'rope> {
        CursorRef {
            rope: self.rope,
            index: self.rope.len_bytes(),
            line: Cell::new(None),
            column: Cell::new(None),
            width: Cell::new(None),
        }
    }

    pub fn at_index(&self, index: usize) -> CursorRef<'rope> {
        debug_assert!(index <= self.rope.len_bytes());
        debug_assert!(utils::is_char_boundary(self.rope, index));

        CursorRef {
            rope: self.rope,
            index,
            line: Cell::new(None),
            column: Cell::new(None),
            width: Cell::new(None),
        }
    }

    pub fn at_line(&self, line: usize) -> CursorRef<'rope> {
        debug_assert!(line < self.rope.len_lines());

        CursorRef {
            rope: self.rope,
            index: self.rope.line_to_byte(line),
            line: Cell::new(Some(line)),
            column: Cell::new(Some(0)),
            width: Cell::new(None),
        }
    }

    pub fn at_line_column(&self, line: usize, column: usize) -> CursorRef<'rope> {
        debug_assert!(line < self.rope.len_lines());
        debug_assert!(
            column
                <= self
                    .rope
                    .line(line)
                    .to_string()
                    .trim_end_matches('\n')
                    .len()
        );

        CursorRef {
            rope: self.rope,
            index: self.rope.line_to_byte(line) + column,
            line: Cell::new(Some(line)),
            column: Cell::new(Some(column)),
            width: Cell::new(None),
        }
    }

    pub fn at_line_width(&self, line: usize, width: usize) -> CursorRef<'rope> {
        debug_assert!(line < self.rope.len_lines());

        let (index, mut graphemes) = {
            let start = self.rope.line_to_byte(line);
            let end = self.rope.line_to_byte(line + 1);

            (
                start,
                GraphemesForward::new(self.rope.byte_slice(start..end)),
            )
        };
        let mut current_width = 0;
        let mut current_column = 0;
        let mut grapheme_end = 0;

        while let Some(grapheme) = graphemes.next() {
            grapheme_end += grapheme.as_str().len();

            let grapheme_width = grapheme.as_str().width();

            if grapheme_width == 0 {
                continue;
            }

            current_width += grapheme_width;

            match current_width.cmp(&width) {
                Ordering::Less => {
                    current_column = grapheme_end;
                    continue;
                }
                Ordering::Equal => {
                    current_column = grapheme_end;
                    break;
                }
                Ordering::Greater => {
                    current_width -= grapheme_width;
                    break;
                }
            }
        }

        CursorRef {
            rope: self.rope,
            index: index + current_column,
            line: Cell::new(Some(line)),
            column: Cell::new(Some(current_column)),
            width: Cell::new(Some(current_width)),
        }
    }

    pub fn at_cursor(&self, cursor: Cursor) -> CursorRef<'rope> {
        debug_assert!(cursor.index <= self.rope.len_bytes());
        debug_assert!(utils::is_char_boundary(self.rope, cursor.index));
        debug_assert!(cursor.line == utils::line(self.rope, cursor.index));
        debug_assert!(cursor.column == utils::column(self.rope, cursor.index, cursor.line));
        debug_assert!(cursor.width == utils::width(self.rope, cursor.line, cursor.column));

        CursorRef {
            rope: self.rope,
            index: cursor.index,
            line: Cell::new(Some(cursor.line)),
            column: Cell::new(Some(cursor.column)),
            width: Cell::new(Some(cursor.width)),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         CursorBuilder                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone)]
pub struct CursorBuilder<'rope> {
    slice: RopeSlice<'rope>,
}

impl<'rope> CursorBuilder<'rope> {
    pub fn at_start(self) -> Cursor {
        Cursor::default()
    }

    pub fn at_end(self) -> Cursor {
        let index = self.slice.len_bytes();
        let line = self.slice.len_lines() - 1;
        let column = utils::column(self.slice, index, line);
        let width = utils::width(self.slice, line, column);

        Cursor {
            index,
            line,
            column,
            width,
        }
    }

    pub fn at_index(&self, index: usize) -> Cursor {
        debug_assert!(index <= self.slice.len_bytes());
        debug_assert!(utils::is_char_boundary(self.slice, index));

        let line = utils::line(self.slice, index);
        let column = utils::column(self.slice, index, line);
        let width = utils::width(self.slice, line, column);

        Cursor {
            index,
            line,
            column,
            width,
        }
    }

    pub fn at_column(&self, line: usize, column: usize) -> Cursor {
        debug_assert!(line < self.slice.len_lines());
        debug_assert!(
            column
                <= self
                    .slice
                    .line(line)
                    .to_string()
                    .trim_end_matches('\n')
                    .trim_end_matches('\r')
                    .len()
        );

        let index = self.slice.line_to_byte(line) + column;
        let width = utils::width(self.slice, line, column);

        Cursor {
            index,
            line,
            column,
            width,
        }
    }

    pub fn at_width(&self, line: usize, width: usize) -> Cursor {
        debug_assert!(line < self.slice.len_lines());

        let (index, mut graphemes) = {
            let start = self.slice.line_to_byte(line);
            let end = self.slice.line_to_byte(line + 1);

            (
                start,
                GraphemesForward::new(self.slice.byte_slice(start..end)),
            )
        };
        let mut current_width = 0;
        let mut current_column = 0;
        let mut grapheme_end = 0;

        while let Some(grapheme) = graphemes.next() {
            grapheme_end += grapheme.as_str().len();

            let grapheme_width = grapheme.as_str().width();

            if grapheme_width == 0 {
                continue;
            }

            current_width += grapheme_width;

            match current_width.cmp(&width) {
                Ordering::Less => {
                    current_column = grapheme_end;
                    continue;
                }
                Ordering::Equal => {
                    current_column = grapheme_end;
                    break;
                }
                Ordering::Greater => {
                    current_width -= grapheme_width;
                    break;
                }
            }
        }

        Cursor {
            index: index + current_column,
            line,
            column: current_column,
            width: current_width,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Utils                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

mod utils {
    use super::*;

    pub fn is_char_boundary(slice: RopeSlice, index: usize) -> bool {
        let (str, offset, _, _) = slice.chunk_at_byte(index);

        str.is_char_boundary(index - offset)
    }

    pub fn line(slice: RopeSlice, index: usize) -> usize {
        slice.byte_to_line(index)
    }

    pub fn column(slice: RopeSlice, index: usize, line: usize) -> usize {
        index - slice.line_to_byte(line)
    }

    pub fn width(slice: RopeSlice, line: usize, column: usize) -> usize {
        slice
            .line(line)
            .byte_slice(..column)
            .chunks()
            .map(|chunk| chunk.width())
            .sum()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use std::ops::Range;

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
            let rope = rope.slice(..);

            assert!(data.len() == str.chars().count() + 1, "Wrong test data");

            for (index, line, column, width) in data.iter().copied() {
                for cursor in [
                    CursorRef::with(rope).at_index(index),
                    CursorRef::with(rope).at_line_column(line, column),
                ] {
                    assert!(cursor.index() == index);
                    assert!(cursor.line() == line);
                    assert!(cursor.column() == column);
                    assert!(cursor.width() == width);
                }

                if index == str.len() {
                    let cursor = CursorRef::with(rope).at_end();

                    assert!(cursor.index() == index);
                    assert!(cursor.line() == line);
                    assert!(cursor.column() == column);
                    assert!(cursor.width() == width);
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
            let rope = rope.slice(..);

            assert!(data.len() == rope.len_lines(), "Wrong test data");

            for (i, (index, line, column, width)) in data.iter().copied().enumerate() {
                let cursor = CursorRef::with(rope).at_line(i);

                assert!(cursor.index() == index);
                assert!(cursor.line() == line);
                assert!(cursor.column() == column);
                assert!(cursor.width() == width);
            }
        }
    }

    #[test]
    fn at_line_width() {
        let str = "\0a\0🦀\0b\0";
        let rope = Rope::from(str);
        let cursor = CursorRef::with(rope.slice(..));

        assert_eq!(&str[..cursor.at_line_width(0, 0).index], "");
        assert_eq!(&str[..cursor.at_line_width(0, 1).index], "\0a");
        assert_eq!(&str[..cursor.at_line_width(0, 2).index], "\0a");
        assert_eq!(&str[..cursor.at_line_width(0, 3).index], "\0a\0🦀");
        assert_eq!(&str[..cursor.at_line_width(0, 4).index], "\0a\0🦀\0b");
        assert_eq!(&str[..cursor.at_line_width(0, 5).index], "\0a\0🦀\0b");
    }

    #[test]
    fn grapheme_prev() {
        // TODO
    }

    #[test]
    fn grapheme_next() {
        // TODO
    }

    #[test]
    fn word() {
        let data: &[(&str, &[&str], &[&str])] = &[
            (
                "hello -world_ || HELLOWorld12;",
                &[
                    "hello ", "-", "world", "_ ", "|| ", "HELLO", "World", "12", ";",
                ],
                &[
                    "hello", " -", "world", "_", " ||", " HELLO", "World", "12", ";",
                ],
            ),
            (
                "pub fn foo_bar(&mut self, baz2: Baz2) {}",
                &[
                    "pub ", "fn ", "foo", "_", "bar", "(", "&", "mut ", "self", ", ", "baz", "2",
                    ": ", "Baz", "2", ") ", "{", "}",
                ],
                &[
                    "pub", " fn", " foo", "_", "bar", "(", "&", "mut", " self", ",", " baz", "2",
                    ":", " Baz", "2", ")", " {", "}",
                ],
            ),
            (
                "a\nb\nc\n",
                &["a\n", "b\n", "c\n"],
                &["a", "\nb", "\nc", "\n"],
            ),
            (
                "\n\nb\nc\n",
                &["\n\n", "b\n", "c\n"],
                &["\n\nb", "\nc", "\n"],
            ),
        ];

        for &(str, starts, ends) in data {
            assert!(
                str == starts.iter().copied().collect::<String>(),
                "Wrong test data",
            );
            assert!(
                str == ends.iter().copied().collect::<String>(),
                "Wrong test data",
            );

            let rope = Rope::from(str);
            let cursor = CursorRef::with(rope.slice(..));

            assert!(cursor.at_index(0).prev_start_of_subword().is_none());
            assert!(cursor.at_index(0).prev_end_of_subword().is_none());
            assert!(cursor.at_index(str.len()).next_start_of_subword().is_none());
            assert!(cursor.at_index(str.len()).next_end_of_subword().is_none());

            fn the_loop(words: &[&str], f: impl Fn(Range<usize>, Range<usize>)) {
                let mut offset = 0;

                for word in words {
                    for char in word
                        .char_indices()
                        .map(|(i, char)| (offset + i, char.len_utf8()))
                        .map(|(i, len)| i..i + len)
                    {
                        f(offset..offset + word.len(), char);
                    }

                    offset += word.len();
                }
            }

            the_loop(starts, |w, c| {
                assert!(
                    cursor
                        .at_index(c.end)
                        .prev_start_of_subword()
                        .unwrap()
                        .index()
                        == w.start
                );
                assert!(
                    cursor
                        .at_index(c.start)
                        .next_start_of_subword()
                        .unwrap()
                        .index()
                        == w.end
                );
            });
            the_loop(ends, |w, c| {
                assert!(
                    cursor
                        .at_index(c.end)
                        .prev_end_of_subword()
                        .unwrap()
                        .index()
                        == w.start
                );
                assert!(
                    cursor
                        .at_index(c.start)
                        .next_end_of_subword()
                        .unwrap()
                        .index()
                        == w.end
                );
            });
        }
    }
}
