use crate::rope3::{GraphemeCursor, WordClass, WordCursor};
use ropey::Rope;
use std::{cmp::Ordering, ops::Range};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use virus_common::Cursor;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            RopeExt                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Extensions for `Rope`s.
pub trait RopeExt {
    /// Returns the [`RopeExtCursor`] API.
    fn cursor<'rope>(&'rope self) -> RopeExtCursor<'rope>;

    /// Returns the [`RopeExtGrapheme`] API.
    fn grapheme<'rope>(&'rope self) -> RopeExtGrapheme<'rope>;

    /// Returns the [`RopeExtWord`] API.
    fn word<'rope>(&'rope self) -> RopeExtWord<'rope>;

    /// Replaces `selection` with `str`.
    fn replace(&mut self, selection: Range<Cursor>, str: &str);
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl RopeExt for Rope {
    fn cursor<'rope>(&'rope self) -> RopeExtCursor<'rope> {
        RopeExtCursor(self)
    }

    fn grapheme<'rope>(&'rope self) -> RopeExtGrapheme<'rope> {
        RopeExtGrapheme(self)
    }

    fn word<'rope>(&'rope self) -> RopeExtWord<'rope> {
        RopeExtWord(self)
    }

    fn replace(&mut self, selection: Range<Cursor>, str: &str) {
        let (start, end) = (selection.start.index, selection.end.index);
        let (start_char, end_char) = (self.byte_to_char(start), self.byte_to_char(end));

        if start != end {
            self.remove(start_char..end_char);
        }

        if !str.is_empty() {
            self.insert(start_char, str);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         RopeExtCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`RopeExt::cursor()`] API.
pub struct RopeExtCursor<'rope>(&'rope Rope);

impl<'rope> RopeExtCursor<'rope> {
    /// Returns a cursor at the start of the text.
    pub fn start(&self) -> Cursor {
        Cursor::ZERO
    }

    /// Returns a cursor at byte `index`.
    pub fn index(&self, index: usize) -> Cursor {
        let line = self.0.byte_to_line(index);
        let column = index - self.0.line_to_byte(line);

        Cursor {
            index,
            line,
            column,
        }
    }

    /// Returns a cursor at the end of the text.
    pub fn end(&self) -> Cursor {
        let index = self.0.len_bytes();
        let line = self.0.len_lines() - 1;
        let column = index - self.0.line_to_byte(line);

        Cursor {
            index,
            line,
            column,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        RopeExtGrapheme                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`RopeExt::grapheme()`] API.
pub struct RopeExtGrapheme<'rope>(&'rope Rope);

impl<'rope> RopeExtGrapheme<'rope> {
    /// Returns whether the given `cursor` is a grapheme boundary.
    pub fn is_boundary(&self, cursor: Cursor) -> bool {
        GraphemeCursor::new(self.0.slice(..), cursor.index).is_boundary()
    }

    /// Finds the previous grapheme boundary after the given `cursor`.
    pub fn prev(&self, cursor: Cursor) -> Cursor {
        GraphemeCursor::new(self.0.slice(..), cursor.index)
            .prev()
            .map(|(range, _)| self.0.cursor().index(range.start))
            .unwrap_or(cursor)
    }

    /// Finds the next grapheme boundary after the given `cursor`.
    pub fn next(&self, cursor: Cursor) -> Cursor {
        GraphemeCursor::new(self.0.slice(..), cursor.index)
            .next()
            .map(|(range, _)| self.0.cursor().index(range.end))
            .unwrap_or(cursor)
    }

    /// Finds the grapheme visually above `cursor`.
    pub fn above(&self, cursor: Cursor) -> Cursor {
        if cursor.line == 0 {
            cursor
        } else {
            find_width(self.0, cursor.line - 1, width(self.0, cursor))
        }
    }

    /// Finds the grapheme visually below `cursor`.
    pub fn below(&self, cursor: Cursor) -> Cursor {
        if cursor.line == self.0.len_lines() - 1 {
            cursor
        } else {
            find_width(self.0, cursor.line + 1, width(self.0, cursor))
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn width(rope: &Rope, cursor: Cursor) -> usize {
    let line = rope.line_to_byte(cursor.line);
    let slice = rope.byte_slice(line..line + cursor.column);

    slice
        .chars()
        .map(|char| char.width().unwrap_or_default())
        .sum()
}

fn find_width(rope: &Rope, line: usize, width: usize) -> Cursor {
    let start = rope.line_to_byte(line);
    let end = rope.line_to_byte(line + 1);
    let slice = rope.byte_slice(start..end);

    let mut graphemes = GraphemeCursor::new(slice, 0);
    let mut current_width = 0;
    let mut column = 0;

    while let Some((range, chunks)) = graphemes.next() {
        let grapheme_width = chunks.map(|(_, str)| str.width()).sum::<usize>();

        if grapheme_width == 0 {
            continue;
        }

        current_width += grapheme_width;

        match current_width.cmp(&width) {
            Ordering::Less => {
                column = range.end;
                continue;
            }
            Ordering::Equal => {
                column = range.end;
                break;
            }
            Ordering::Greater => break,
        }
    }

    Cursor::new(start + column, line, column)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          RopeExtWord                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`RopeExt::word()`] API.
pub struct RopeExtWord<'rope>(&'rope Rope);

impl<'rope> RopeExtWord<'rope> {
    pub fn prev_start(&self, cursor: Cursor) -> Cursor {
        let mut words = WordCursor::new(self.0.slice(..), cursor.index);

        match words.prev() {
            Some((_, WordClass::Whitespace)) => match words.prev() {
                Some((range, _)) => self.0.cursor().index(range.start),
                None => self.0.cursor().start(),
            },
            Some((range, _)) => self.0.cursor().index(range.start),
            None => cursor,
        }
    }

    pub fn prev_end(&self, cursor: Cursor) -> Cursor {
        let mut words = WordCursor::new(self.0.slice(..), cursor.index);

        match words.prev() {
            Some((range, WordClass::Whitespace)) => self.0.cursor().index(range.start),
            Some(_) => match words.prev() {
                Some((range, WordClass::Whitespace)) => self.0.cursor().index(range.start),
                Some((range, _)) => self.0.cursor().index(range.end),
                None => self.0.cursor().start(),
            },
            None => cursor,
        }
    }

    pub fn next_start(&self, cursor: Cursor) -> Cursor {
        let mut words = WordCursor::new(self.0.slice(..), cursor.index);

        match words.next() {
            Some((range, WordClass::Whitespace)) => self.0.cursor().index(range.end),
            Some(_) => match words.next() {
                Some((range, WordClass::Whitespace)) => self.0.cursor().index(range.end),
                Some((range, _)) => self.0.cursor().index(range.start),
                None => self.0.cursor().end(),
            },
            None => cursor,
        }
    }

    pub fn next_end(&self, cursor: Cursor) -> Cursor {
        let mut words = WordCursor::new(self.0.slice(..), cursor.index);

        match words.next() {
            Some((_, WordClass::Whitespace)) => match words.next() {
                Some((range, _)) => self.0.cursor().index(range.end),
                None => self.0.cursor().end(),
            },
            Some((range, _)) => self.0.cursor().index(range.end),
            None => cursor,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor() {
        let data: &[(&str, &[(usize, usize)])] = &[
            ("", &[(0, 0)]),
            ("\n", &[(0, 0), (1, 0)]),
            ("\n\n", &[(0, 0), (1, 0), (2, 0)]),
            ("\n\na", &[(0, 0), (1, 0), (2, 0), (2, 1)]),
            ("\n\nab", &[(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]),
            ("a", &[(0, 0), (0, 1)]),
            ("ab", &[(0, 0), (0, 1), (0, 2)]),
            ("a\r", &[(0, 0), (0, 1), (1, 0)]),
            ("a\n", &[(0, 0), (0, 1), (1, 0)]),
            ("a\r\n", &[(0, 0), (0, 1), (0, 2), (1, 0)]),
            ("a\r\na", &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 1)]),
            (
                "a\r\na\n",
                &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (2, 0)],
            ),
        ];

        for &(str, data) in data {
            let rope = Rope::from(str);

            assert!(data.len() == str.len() + 1, "Wrong test data");
            assert!(rope.cursor().start() == Cursor::new(0, 0, 0));

            for index in 0..=str.len() {
                let (line, column) = data[index];
                let cursor = Cursor::new(index, line, column);

                assert!(rope.cursor().index(index) == cursor);

                if index == str.len() {
                    assert!(rope.cursor().end() == cursor);
                }
            }
        }
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
    fn grapheme_above() {
        // Single line
        assert!(Rope::from("").grapheme().above(Cursor::new(0, 0, 0)) == Cursor::new(0, 0, 0));
        assert!(Rope::from("ab").grapheme().above(Cursor::new(0, 0, 0)) == Cursor::new(0, 0, 0));
        assert!(Rope::from("ab").grapheme().above(Cursor::new(1, 0, 1)) == Cursor::new(1, 0, 1));
        assert!(Rope::from("ab").grapheme().above(Cursor::new(2, 0, 2)) == Cursor::new(2, 0, 2));

        // 🦀 is 1 char, 4 bytes
        let data: &[((&str, &str), &[usize])] = &[
            (("\n", "ab"), &[0, 0, 0]),
            (("ab\n", "ab"), &[0, 1, 2]),
            (("\0a\0b\0\n", "ab"), &[0, 2, 4]),
            (("a🦀d\n", "abcd"), &[0, 1, 1, 5, 6]),
        ];

        for &((top, bottom), belows) in data {
            assert!(top.ends_with('\n'));

            let rope = Rope::from(top.to_string() + bottom);
            let mut columns = belows.iter();

            for index in (0..=bottom.len()).filter(|index| bottom.is_char_boundary(*index)) {
                let column = *columns.next().unwrap();
                let cursor = Cursor::new(top.len() + index, 1, index);
                let above = Cursor::new(column, 0, column);

                assert!(rope.grapheme().above(cursor) == above);
            }

            assert!(columns.next().is_none());
        }
    }

    #[test]
    fn grapheme_below() {
        // Single line
        assert!(Rope::from("").grapheme().below(Cursor::new(0, 0, 0)) == Cursor::new(0, 0, 0));
        assert!(Rope::from("ab").grapheme().below(Cursor::new(0, 0, 0)) == Cursor::new(0, 0, 0));
        assert!(Rope::from("ab").grapheme().below(Cursor::new(1, 0, 1)) == Cursor::new(1, 0, 1));
        assert!(Rope::from("ab").grapheme().below(Cursor::new(2, 0, 2)) == Cursor::new(2, 0, 2));

        // 🦀 is 1 char, 4 bytes
        let data: &[((&str, &str), &[usize])] = &[
            (("ab", "\n"), &[0, 0, 0]),
            (("ab", "\nab"), &[0, 1, 2]),
            (("ab", "\n\0a\0b\0"), &[0, 2, 4]),
            (("abcd", "\na🦀d"), &[0, 1, 1, 5, 6]),
        ];

        for &((top, bottom), belows) in data {
            assert!(bottom.starts_with('\n'));

            let rope = Rope::from(top.to_string() + bottom);
            let mut columns = belows.iter();

            for index in (0..=top.len()).filter(|index| top.is_char_boundary(*index)) {
                let column = *columns.next().unwrap();
                let cursor = Cursor::new(index, 0, index);
                let below = Cursor::new(top.len() + 1 + column, 1, column);

                assert!(rope.grapheme().below(cursor) == below);
            }

            assert!(columns.next().is_none());
        }
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
            let cursor = |index| rope.cursor().index(index);

            assert!(rope.word().prev_start(cursor(0)) == cursor(0));
            assert!(rope.word().prev_end(cursor(0)) == cursor(0));
            assert!(rope.word().next_start(cursor(str.len())) == cursor(str.len()));
            assert!(rope.word().next_end(cursor(str.len())) == cursor(str.len()));

            fn the_loop(words: &[&str], f: impl Fn(usize, &str, Range<usize>)) {
                let mut offset = 0;

                for word in words {
                    for char in word
                        .char_indices()
                        .map(|(i, char)| (offset + i, char.len_utf8()))
                        .map(|(i, len)| i..i + len)
                    {
                        f(offset, word, char);
                    }

                    offset += word.len();
                }
            }

            the_loop(starts, |offset, word, char| {
                assert!(rope.word().prev_start(cursor(char.end)) == cursor(offset));
                assert!(rope.word().next_start(cursor(char.start)) == cursor(offset + word.len()));
            });
            the_loop(ends, |offset, word, char| {
                assert!(rope.word().prev_end(cursor(char.end)) == cursor(offset));
                assert!(rope.word().next_end(cursor(char.start)) == cursor(offset + word.len()));
            });
        }
    }
}
