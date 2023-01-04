use super::LineCursor;
use std::iter::FusedIterator;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LineNextChars                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A fused double-ended iterator over the next chars of a [`LineCursor`].
///
/// See [`LineCursor::next_chars()`].
#[derive(Clone, Debug)]
pub struct LineNextChars<'a> {
    pub(crate) string: &'a str,
    front: usize,
    back: usize,
    chars: std::iter::Chain<std::str::Chars<'a>, std::option::IntoIter<char>>,
}

impl<'a> LineNextChars<'a> {
    /// Returns a new [`LineNextChars`] from `¢ursor`.
    pub fn new(cursor: LineCursor<'a>) -> Self {
        let string = cursor.string;

        Self {
            string,
            front: cursor.offset,
            back: cursor.len(),
            chars: if let Some(string) = string.get(cursor.offset..) {
                string.chars().chain(Some('\n'))
            } else {
                "".chars().chain(None)
            },
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

impl<'a> Iterator for LineNextChars<'a> {
    type Item = (LineCursor<'a>, char);

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = LineCursor {
            string: self.string,
            offset: self.front,
        };

        let char = self.chars.next()?;
        self.front += char.len_utf8();

        Some((cursor, char))
    }
}

impl<'a> DoubleEndedIterator for LineNextChars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let char = self.chars.next_back()?;
        self.back -= char.len_utf8();

        let cursor = LineCursor {
            string: self.string,
            offset: self.back,
        };

        Some((cursor, char))
    }
}

impl<'a> FusedIterator for LineNextChars<'a> {}
