use crate::rope::GraphemesForward;
use ropey::RopeSlice;
use std::cmp::Ordering;
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
    pub fn build(slice: RopeSlice) -> CursorBuilder {
        CursorBuilder { slice }
    }

    // TODO tests
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
                assert!(!(before.ends_with('\r') && after.starts_with('\n')));

                rope.append(ropey::Rope::from(before));
                cursors.push({
                    let line = rope.line(rope.len_lines() - 1);
                    Self {
                        index: rope.len_bytes(),
                        line: rope.len_lines() - 1,
                        column: line.len_bytes(),
                        width: line.chunks().map(str::width).sum(),
                    }
                });
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

    #[test]
    fn at_index_at_column_at_width_at_end() {
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
            ("a\r\n", &[(0, 0, 0, 0), (1, 0, 1, 1), (3, 1, 0, 0)]),
            (
                "a\r\na",
                &[(0, 0, 0, 0), (1, 0, 1, 1), (3, 1, 0, 0), (4, 1, 1, 1)],
            ),
            (
                "a\r\na🦀d\n",
                &[
                    (0, 0, 0, 0),
                    (1, 0, 1, 1),
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

            for (index, line, column, width) in data.iter().copied() {
                for cursor in [
                    Cursor::build(rope).at_index(index),
                    Cursor::build(rope).at_column(line, column),
                    Cursor::build(rope).at_width(line, width),
                ] {
                    assert!(cursor.index == index);
                    assert!(cursor.line == line);
                    assert!(cursor.column == column);
                    assert!(cursor.width == width);
                }

                if index == str.len() {
                    let cursor = Cursor::build(rope).at_end();

                    assert!(cursor.index == index);
                    assert!(cursor.line == line);
                    assert!(cursor.column == column);
                    assert!(cursor.width == width);
                }
            }
        }
    }

    #[test]
    fn at_width() {
        let str = "\0a\0🦀\0b\0";
        let rope = Rope::from(str);
        let cursor = Cursor::build(rope.slice(..));

        assert_eq!(&str[..cursor.at_width(0, 0).index], "");
        assert_eq!(&str[..cursor.at_width(0, 1).index], "\0a");
        assert_eq!(&str[..cursor.at_width(0, 2).index], "\0a");
        assert_eq!(&str[..cursor.at_width(0, 3).index], "\0a\0🦀");
        assert_eq!(&str[..cursor.at_width(0, 4).index], "\0a\0🦀\0b");
        assert_eq!(&str[..cursor.at_width(0, 5).index], "\0a\0🦀\0b");
    }
}
