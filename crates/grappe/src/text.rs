use crate::{
    line::{Line, LineCursor, LineNextChars, LinePrevChars},
    Index,
};
use std::sync::Arc;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Text                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An immutable, thread-safe [`String`].
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Text {
    lines: Arc<Vec<Line>>,
    len: usize,
}

impl Text {
    /// Creates a new [`Text`] from `lines` and `len`.
    pub fn new(lines: Arc<Vec<Line>>, len: usize) -> Self {
        Self { lines, len }
    }

    /// Returns the byte length of this [`Text`].
    pub fn len(&self) -> usize {
        self.lines.iter().map(Line::len).sum()
    }

    /// Returns `true` if this [`Text`] has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        // Lines are never empty
        self.lines.len() == 0
    }

    /// Returns the [`Line`]s of this [`Text`].
    pub fn lines(&self) -> &Arc<Vec<Line>> {
        &self.lines
    }

    /// Gets the strong count to the [`lines`](Self::lines).
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.lines)
    }

    /// Gets the weak count to the [`lines`](Self::lines).
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.lines)
    }

    /// Makes a mutable reference into this [`Text`].
    pub fn make_mut(&mut self) -> &mut Vec<Line> {
        Arc::make_mut(&mut self.lines)
    }
}

impl From<&str> for Text {
    fn from(str: &str) -> Self {
        Self {
            lines: Arc::new(str.lines().map(Into::into).collect()),
            len: str.len(),
        }
    }
}

impl AsRef<[Line]> for Text {
    fn as_ref(&self) -> &[Line] {
        &self.lines
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.lines.iter() {
            std::fmt::Display::fmt(line, f)?;
        }

        Ok(())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            TextCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct TextCursor<'a> {
    lines: &'a [Line],
    offset: usize,
    row: usize,
    column: usize,
    len: usize,
}

impl<'a> TextCursor<'a> {
    pub fn from_start(text: &'a Text) -> Self {
        Self {
            lines: text.lines(),
            offset: 0,
            row: 0,
            column: 0,
            len: text.len(),
        }
    }

    pub fn from_end(text: &'a Text) -> Self {
        Self {
            lines: text.lines(),
            offset: text.len(),
            row: text.lines.len(),
            column: 0,
            len: text.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn index(&self) -> Index {
        Index {
            offset: self.offset,
            row: self.row,
            column: self.column,
        }
    }

    /// Returns a fused double-ended iterator over the previous chars in the text.
    pub fn prev_chars(&self) -> TextPrevChars<'a> {
        TextPrevChars::new(*self)
    }

    pub fn start(&mut self) {
        self.offset = 0;
        self.row = 0;
        self.column = 0;
    }

    pub fn end(&mut self) {
        self.offset = self.len;
        self.row = self.lines.len();
        self.column = 0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPrevChars                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A fused double-ended iterator over the previous chars of a [`TextCursor`].
///
/// See [`TextCursor::prev_chars()`].
#[derive(Clone, Debug)]
pub struct TextPrevChars<'a> {
    lines: &'a [Line],
    front_chars: LinePrevChars<'a>,
    front_offset: usize,
    front_row: usize,
    back_chars: LinePrevChars<'a>,
    back_offset: usize,
    back_row: usize,
    len: usize,
}

impl<'a> TextPrevChars<'a> {
    /// Returns a new [`TextPrevChars`] from `cursor`.
    pub fn new(cursor: TextCursor<'a>) -> Self {
        Self {
            lines: cursor.lines,
            front_chars: LineCursor::new_unchecked(
                cursor
                    .lines
                    .get(cursor.row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default(),
                cursor.column,
            )
            .prev_chars(),
            front_offset: cursor.offset,
            front_row: cursor.row,
            back_chars: if let Some(line) = cursor.lines.first() {
                LineCursor::from_end(line)
            } else {
                LineCursor::from_empty()
            }
            .prev_chars(),
            back_offset: 0,
            back_row: 0,
            len: cursor.len,
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

impl<'a> Iterator for TextPrevChars<'a> {
    type Item = (TextCursor<'a>, char);

    fn next(&mut self) -> Option<Self::Item> {
        if self.front_offset == self.back_offset {
            return None;
        }

        let (line_cursor, char) = match self.front_chars.next() {
            Some((line_cursor, char)) => (line_cursor, char),
            None => {
                if self.front_row > self.back_row {
                    self.front_row -= 1;
                    self.front_chars =
                        LineCursor::from_end(&self.lines[self.front_row]).prev_chars();
                    self.front_chars.next()?
                } else {
                    return None;
                }
            }
        };

        self.front_offset -= char.len_utf8();
        Some((
            TextCursor {
                lines: self.lines,
                offset: self.front_offset,
                row: self.front_row,
                column: line_cursor.offset(),
                len: self.len,
            },
            char,
        ))
    }
}

impl<'a> DoubleEndedIterator for TextPrevChars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front_offset == self.back_offset {
            return None;
        }

        let offset = self.back_offset;
        let row = self.back_row;
        let (line_cursor, char) = self.back_chars.next_back()?;

        if self.back_chars.back() == self.back_chars.string.len() + /* newline */ 1 {
            self.back_row += 1;
            self.back_chars = if let Some(line) = self.lines.get(self.back_row) {
                LineCursor::from_end(line)
            } else {
                LineCursor::from_empty()
            }
            .prev_chars()
        }

        self.back_offset += char.len_utf8();
        Some((
            TextCursor {
                lines: self.lines,
                offset,
                row,
                column: line_cursor.offset(),
                len: self.len,
            },
            char,
        ))
    }
}

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
                    .get(cursor.row)
                    .map(|line| line.as_ref())
                    .unwrap_or_default(),
                cursor.column,
            )
            .next_chars(),
            front_offset: cursor.offset,
            front_row: cursor.row,
            back_chars: if let Some(line) = cursor.lines.last() {
                LineCursor::from_start(line)
            } else {
                LineCursor::from_empty()
            }
            .next_chars(),
            back_offset: 0,
            back_row: 0,
            len: cursor.len,
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Tests                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Dir {
        Next,
        NextBack,
    }

    fn i(offset: usize, row: usize, column: usize) -> Index {
        Index {
            offset,
            row,
            column,
        }
    }

    #[test]
    fn text_prev_chars() {
        fn assert_prev_chars(
            chars: &[char],
            indexes: &[Index],
            mut prev_chars: TextPrevChars,
            front: usize,
            back: usize,
            dir: Dir,
        ) {
            fn next(prev_chars: &mut TextPrevChars) -> Option<(Index, char)> {
                prev_chars
                    .next()
                    .map(|(cursor, char)| (cursor.index(), char))
            }
            fn next_back(prev_chars: &mut TextPrevChars) -> Option<(Index, char)> {
                prev_chars
                    .next_back()
                    .map(|(cursor, char)| (cursor.index(), char))
            }

            if front + back == chars.len() {
                let front = prev_chars.front().index();
                let back = prev_chars.back().index();

                for _ in 0..10 {
                    assert!(next(&mut prev_chars) == None);
                    assert!(prev_chars.front().index() == front);
                    assert!(prev_chars.back().index() == back);

                    assert!(next_back(&mut prev_chars) == None);
                    assert!(prev_chars.front().index() == front);
                    assert!(prev_chars.back().index() == back);
                }

                return;
            }

            match dir {
                Dir::Next => {
                    let old_back = prev_chars.back().index();
                    assert!(
                        next(&mut prev_chars)
                            == Some((
                                indexes[indexes.len() - 2 - front],
                                chars[chars.len() - 1 - front]
                            ))
                    );
                    assert!(prev_chars.front().index() == indexes[indexes.len() - 2 - front]);
                    assert!(prev_chars.back().index() == old_back);

                    assert_prev_chars(
                        chars,
                        indexes,
                        prev_chars.clone(),
                        front + 1,
                        back,
                        Dir::Next,
                    );
                    assert_prev_chars(chars, indexes, prev_chars, front + 1, back, Dir::NextBack);
                }
                Dir::NextBack => {
                    let old_front = prev_chars.front().index();
                    assert!(next_back(&mut prev_chars) == Some((indexes[back], chars[back])));
                    assert!(prev_chars.front().index() == old_front);
                    assert!(prev_chars.back().index() == indexes[back + 1]);

                    assert_prev_chars(
                        chars,
                        indexes,
                        prev_chars.clone(),
                        front,
                        back + 1,
                        Dir::Next,
                    );
                    assert_prev_chars(chars, indexes, prev_chars, front, back + 1, Dir::NextBack);
                }
            }
        }

        fn assert(str: &str, chars: &[char], indexes: &[Index]) {
            let text = Text::from(str);
            let cursor = TextCursor::from_end(&text);

            let prev_chars = cursor.prev_chars();

            assert!(prev_chars.front().index() == indexes[indexes.len() - 1]);
            assert!(prev_chars.back().index() == indexes[0]);

            assert_prev_chars(&chars, &indexes, prev_chars.clone(), 0, 0, Dir::Next);
            assert_prev_chars(&chars, &indexes, prev_chars, 0, 0, Dir::NextBack);
        }

        assert("\n", &['\n'], &[i(0, 0, 0), i(1, 1, 0)]);
        assert("\n\n", &['\n', '\n'], &[i(0, 0, 0), i(1, 1, 0), i(2, 2, 0)]);
        assert(
            "😍🦀\n",
            &['😍', '🦀', '\n'],
            &[i(0, 0, 0), i(4, 0, 4), i(8, 0, 8), i(9, 1, 0)],
        );
        assert(
            "😍\n🦀\n",
            &['😍', '\n', '🦀', '\n'],
            &[i(0, 0, 0), i(4, 0, 4), i(5, 1, 0), i(9, 1, 4), i(10, 2, 0)],
        );
    }
}
