use crate::{
    cursor::Cursor,
    rope::{GraphemeCursor, WordClass, WordCursor},
};
use ropey::Rope;

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
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         RopeExtCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`RopeExt::cursor()`] API.
pub struct RopeExtCursor<'rope>(&'rope Rope);

impl<'rope> RopeExtCursor<'rope> {
    /// Returns a cursor at the start of the rope.
    pub fn at_start(&self) -> Cursor {
        Cursor::at_start()
    }

    /// Returns a cursor at the end of the rope.
    pub fn at_end(&self) -> Cursor {
        Cursor::at_end(self.0)
    }

    /// Returns a cursor at `index`.
    pub fn at_index(&self, index: usize) -> Cursor {
        Cursor::at_index(index)
    }

    /// Returns a cursor at `line`.
    pub fn at_line(&self, line: usize) -> Cursor {
        Cursor::at_line(line, self.0)
    }

    /// Returns a cursor at `(line, column)`.
    pub fn at_line_column(&self, line: usize, column: usize) -> Cursor {
        Cursor::at_line_column(line, column, self.0)
    }

    /// Returns a cursor at `(line, width)`.
    pub fn at_line_width(&self, line: usize, width: usize) -> Cursor {
        Cursor::at_line_width(line, width, self.0)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        RopeExtGrapheme                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`RopeExt::grapheme()`] API.
pub struct RopeExtGrapheme<'rope>(&'rope Rope);

impl<'rope> RopeExtGrapheme<'rope> {
    /// Returns whether the given `index` is a grapheme boundary.
    pub fn is_boundary(&self, index: usize) -> bool {
        GraphemeCursor::new(self.0.slice(..), index).is_boundary()
    }

    /// Finds the previous grapheme boundary before the given `index`.
    pub fn prev(&self, index: usize) -> Option<Cursor> {
        GraphemeCursor::new(self.0.slice(..), index)
            .prev()
            .map(|(range, _)| self.0.cursor().at_index(range.start))
    }

    /// Finds the next grapheme boundary after the given `index`.
    pub fn next(&self, index: usize) -> Option<Cursor> {
        GraphemeCursor::new(self.0.slice(..), index)
            .next()
            .map(|(range, _)| self.0.cursor().at_index(range.end))
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          RopeExtWord                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`RopeExt::word()`] API.
pub struct RopeExtWord<'rope>(&'rope Rope);

impl<'rope> RopeExtWord<'rope> {
    /// Finds the previous start of word before the given `index`.
    pub fn prev_start(&self, index: usize) -> Option<Cursor> {
        let mut words = WordCursor::new(self.0.slice(..), index);

        words.prev().map(|(range, class)| match class {
            WordClass::Whitespace => words
                .prev()
                .map(|(range, _)| self.0.cursor().at_index(range.start))
                .unwrap_or_else(|| self.0.cursor().at_start()),
            _ => self.0.cursor().at_index(range.start),
        })
    }

    /// Finds the previous end of word before the given `index`.
    pub fn prev_end(&self, index: usize) -> Option<Cursor> {
        let mut words = WordCursor::new(self.0.slice(..), index);

        words.prev().map(|(range, class)| match class {
            WordClass::Whitespace => self.0.cursor().at_index(range.start),
            _ => words
                .prev()
                .map(|(range, class)| match class {
                    WordClass::Whitespace => self.0.cursor().at_index(range.start),
                    _ => self.0.cursor().at_index(range.end),
                })
                .unwrap_or_else(|| self.0.cursor().at_start()),
        })
    }

    /// Finds the next start of word after the given `index`.
    pub fn next_start(&self, index: usize) -> Option<Cursor> {
        let mut words = WordCursor::new(self.0.slice(..), index);

        words.next().map(|(range, class)| match class {
            WordClass::Whitespace => self.0.cursor().at_index(range.end),
            _ => words
                .next()
                .map(|(range, class)| match class {
                    WordClass::Whitespace => self.0.cursor().at_index(range.end),
                    _ => self.0.cursor().at_index(range.start),
                })
                .unwrap_or_else(|| self.0.cursor().at_end()),
        })
    }

    /// Finds the next end of word after the given `index`.
    pub fn next_end(&self, index: usize) -> Option<Cursor> {
        let mut words = WordCursor::new(self.0.slice(..), index);

        words.next().map(|(range, class)| match class {
            WordClass::Whitespace => words
                .next()
                .map(|(range, _)| self.0.cursor().at_index(range.end))
                .unwrap_or_else(|| self.0.cursor().at_end()),
            _ => self.0.cursor().at_index(range.end),
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

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
            let cursor = |index| rope.cursor().at_index(index);

            assert!(rope.word().prev_start(0).is_none());
            assert!(rope.word().prev_end(0).is_none());
            assert!(rope.word().next_start(str.len()).is_none());
            assert!(rope.word().next_end(str.len()).is_none());

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
                assert!(rope.word().prev_start(char.end) == Some(cursor(offset)));
                assert!(rope.word().next_start(char.start) == Some(cursor(offset + word.len())));
            });
            the_loop(ends, |offset, word, char| {
                assert!(rope.word().prev_end(char.end) == Some(cursor(offset)));
                assert!(rope.word().next_end(char.start) == Some(cursor(offset + word.len())));
            });
        }
    }
}
