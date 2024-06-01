use crate::rope3::{GraphemeCursor, WordClass, WordCursor};
use ropey::Rope;
use std::{cmp::Ordering::*, ops::Range};
use unicode_width::UnicodeWidthChar;
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
    debug_assert!(line < rope.len_lines() - 1);

    // Get this line
    let start = rope.line_to_byte(line);
    let end = rope.line_to_byte(line + 1);
    let slice = rope.byte_slice(start..end);

    // Find the cursor at that width on the line (falling back left)
    let mut w = 0;
    let mut cursor = Cursor::new(start, line, 0);

    for char in slice.chars() {
        let prev = cursor;
        cursor.column += char.len_utf8();
        cursor.index += char.len_utf8();
        w += char.width().unwrap_or_default();

        match w.cmp(&width) {
            Less => continue,
            Equal => break,
            Greater => {
                cursor = prev;
                break;
            }
        }
    }

    // Ensure grapheme boundary
    if rope.grapheme().is_boundary(cursor) {
        cursor
    } else {
        rope.grapheme().prev(cursor)
    }
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
    fn grapheme_above() {}

    #[test]
    fn grapheme_below() {}

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

            assert!(str == starts.iter().copied().collect::<String>());
            assert!(str == ends.iter().copied().collect::<String>());

            let rope = Rope::from(str);
            let cursor = |index| rope.cursor().index(index);

            assert!(rope.word().prev_start(cursor(0)) == cursor(0));
            assert!(rope.word().prev_end(cursor(0)) == cursor(0));
            assert!(rope.word().next_start(cursor(str.len())) == cursor(str.len()));
            assert!(rope.word().next_end(cursor(str.len())) == cursor(str.len()));

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
