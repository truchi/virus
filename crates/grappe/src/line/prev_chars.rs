use super::LineCursor;
use std::iter::FusedIterator;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LinePrevChars                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A fused double-ended iterator over the previous chars of a [`LineCursor`].
///
/// See [`LineCursor::prev_chars()`].
#[derive(Clone, Debug)]
pub struct LinePrevChars<'a> {
    pub(crate) string: &'a str,
    front: usize,
    back: usize,
    chars: std::iter::Rev<std::iter::Chain<std::str::Chars<'a>, std::option::IntoIter<char>>>,
}

impl<'a> LinePrevChars<'a> {
    /// Returns a new [`LinePrevChars`] from `cursor`.
    pub fn new(cursor: LineCursor<'a>) -> Self {
        let string = cursor.string;

        Self {
            string,
            front: cursor.offset(),
            back: 0,
            chars: if let Some(string) = string.get(..cursor.offset()) {
                string.chars().chain(None)
            } else {
                string.chars().chain(Some('\n'))
            }
            .rev(),
        }
    }

    /// The front index of the iterator.
    pub fn front(&self) -> usize {
        self.front
    }

    /// The back index of the iterator.
    pub fn back(&self) -> usize {
        self.back
    }
}

impl<'a> Iterator for LinePrevChars<'a> {
    type Item = (LineCursor<'a>, char);

    fn next(&mut self) -> Option<Self::Item> {
        let char = self.chars.next()?;
        self.front -= char.len_utf8();

        let cursor = LineCursor {
            string: self.string,
            offset: self.front,
        };

        Some((cursor, char))
    }
}

impl<'a> DoubleEndedIterator for LinePrevChars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let cursor = LineCursor {
            string: self.string,
            offset: self.back,
        };

        let char = self.chars.next_back()?;
        self.back += char.len_utf8();

        Some((cursor, char))
    }
}

impl<'a> FusedIterator for LinePrevChars<'a> {}
