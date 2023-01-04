use super::TextCursor;
use crate::line::{Line, LineCursor, LinePrevChars};

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
                    .get(cursor.row())
                    .map(|line| line.as_ref())
                    .unwrap_or_default(),
                cursor.column(),
            )
            .prev_chars(),
            front_offset: cursor.offset(),
            front_row: cursor.row(),
            back_chars: if let Some(line) = cursor.lines.first() {
                LineCursor::from_end(line)
            } else {
                LineCursor::from_empty()
            }
            .prev_chars(),
            back_offset: 0,
            back_row: 0,
            len: cursor.len(),
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
//                                               Tests                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{text::Text, Index};

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
