use std::{fmt::Write, iter::FusedIterator, sync::Arc};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Line                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An immutable, thread-safe line in a [`Text`](crate::text::Text).
///
/// A thread-safe reference-counted `String` of line content (***without newlines***)
/// with a virtual final `\n`.
///
/// ***Do not insert newlines!***
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Line {
    string: Arc<String>,
}

impl Line {
    /// Creates a new [`Line`] from `string`.
    ///
    /// ***Does not check for newlines!***
    pub fn new(string: Arc<String>) -> Self {
        debug_assert!(!string.contains('\n'));
        Self { string }
    }

    /// Returns the byte length of this [`Line`].
    ///
    /// At least `1`, the newline.
    pub fn len(&self) -> usize {
        self.string.len() + /* newline */ 1
    }

    /// Returns wheter this [`Line`] is empty, i.e. `false`.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the `String` of this [`Line`].
    ///
    /// ***Does not include the final newline!***
    pub fn string(&self) -> &Arc<String> {
        &self.string
    }

    /// Gets the strong count to the [`string`](Self::string).
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.string)
    }

    /// Gets the weak count to the [`string`](Self::string).
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.string)
    }

    /// Makes a mutable reference into this [`Line`].
    ///
    /// ***Do not insert newlines!***
    pub fn make_mut(&mut self) -> &mut String {
        Arc::make_mut(&mut self.string)
    }
}

impl From<&str> for Line {
    /// ***Does not check for newlines!***
    fn from(str: &str) -> Self {
        debug_assert!(!str.contains('\n'));
        Self {
            string: Arc::new(str.to_string()),
        }
    }
}

impl AsRef<str> for Line {
    /// ***Does not include the final newline!***
    fn as_ref(&self) -> &str {
        &self.string
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string)?;
        f.write_char('\n')
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            LineCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Line`].
#[derive(Copy, Clone, Debug)]
pub struct LineCursor<'a> {
    string: &'a str,
    offset: usize,
}

impl<'a> LineCursor<'a> {
    pub fn new(string: &'a str, offset: usize) -> Option<Self> {
        if offset == string.len() + /* newline */ 1 || string.is_char_boundary(offset) {
            Some(Self { string, offset })
        } else {
            None
        }
    }

    pub fn new_unchecked(string: &'a str, offset: usize) -> Self {
        debug_assert!(Self::new(string, offset).is_some());
        Self { string, offset }
    }

    /// Creates a new [`LineCursor`] at the start of `line`.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let cursor = LineCursor::from_start(&line);
    /// assert!(cursor.offset() == 0);
    /// ```
    pub fn from_start(line: &'a Line) -> Self {
        Self {
            string: &line.string,
            offset: 0,
        }
    }

    /// Creates a new [`LineCursor`] at the end of `line`.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let cursor = LineCursor::from_end(&line);
    /// assert!(cursor.offset() == 9);
    /// ```
    pub fn from_end(line: &'a Line) -> Self {
        Self {
            string: &line.string,
            offset: line.len(),
        }
    }

    pub fn from_empty() -> Self {
        Self {
            string: "",
            offset: 0,
        }
    }

    /// Returns the length of the [`Line`].
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let cursor = LineCursor::from_start(&line);
    /// assert!(cursor.len() == 9);
    /// ```
    pub fn len(&self) -> usize {
        self.string.len() + /* newline */ 1
    }

    /// Returns the current index of this [`LineCursor`].
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let cursor = LineCursor::from_start(&line);
    /// assert!(cursor.offset() == 0);
    ///
    /// let cursor = LineCursor::from_end(&line);
    /// assert!(cursor.offset() == 9);
    /// ```
    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn is_start(&self) -> bool {
        self.offset == 0
    }

    pub fn is_end(&self) -> bool {
        self.offset == self.len()
    }

    /// Returns a fused double-ended iterator over the previous chars in the line.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    /// let cursor = LineCursor::from_end(&line);
    /// let map = |o: Option<(LineCursor, char)>| o.map(|(cursor, char)| (cursor.offset(), char));
    ///
    /// let mut prev_chars = cursor.prev_chars();
    /// assert!((prev_chars.front(), prev_chars.back()) == (9, 0));
    ///
    /// assert!(map(prev_chars.next()) == Some((8, '\n')));
    /// assert!((prev_chars.front(), prev_chars.back()) == (8, 0));
    ///
    /// assert!(map(prev_chars.next_back()) == Some((0, '😍')));
    /// assert!((prev_chars.front(), prev_chars.back()) == (8, 4));
    ///
    /// assert!(map(prev_chars.next()) == Some((4, '🦀')));
    /// assert!((prev_chars.front(), prev_chars.back()) == (4, 4));
    ///
    /// assert!(map(prev_chars.next_back()) == None);
    /// assert!(map(prev_chars.next()) == None);
    /// ```
    pub fn prev_chars(&self) -> LinePrevChars<'a> {
        LinePrevChars::new(*self)
    }

    /// Returns a fused double-ended iterator over the next chars in the line.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    /// let cursor = LineCursor::from_start(&line);
    /// let map = |o: Option<(LineCursor, char)>| o.map(|(cursor, char)| (cursor.offset(), char));
    ///
    /// let mut next_chars = cursor.next_chars();
    /// assert!((next_chars.front(), next_chars.back()) == (0, 9));
    ///
    /// assert!(map(next_chars.next()) == Some((0, '😍')));
    /// assert!((next_chars.front(), next_chars.back()) == (4, 9));
    ///
    /// assert!(map(next_chars.next_back()) == Some((8, '\n')));
    /// assert!((next_chars.front(), next_chars.back()) == (4, 8));
    ///
    /// assert!(map(next_chars.next()) == Some((4, '🦀')));
    /// assert!((next_chars.front(), next_chars.back()) == (8, 8));
    ///
    /// assert!(map(next_chars.next_back()) == None);
    /// assert!(map(next_chars.next()) == None);
    /// ```
    pub fn next_chars(&self) -> LineNextChars<'a> {
        LineNextChars::new(*self)
    }

    /// Goes to the previous char in the line and returns it,
    /// or `None` if already at the first char.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let mut cursor = LineCursor::from_end(&line);
    /// assert!(cursor.offset() == 9);
    ///
    /// assert!(cursor.prev_char() == Some('\n'));
    /// assert!(cursor.offset() == 8);
    ///
    /// assert!(cursor.prev_char() == Some('🦀'));
    /// assert!(cursor.offset() == 4);
    ///
    /// assert!(cursor.prev_char() == Some('😍'));
    /// assert!(cursor.offset() == 0);
    ///
    /// assert!(cursor.prev_char() == None);
    /// assert!(cursor.offset() == 0);
    /// ```
    pub fn prev_char(&mut self) -> Option<char> {
        let mut prev_chars = self.prev_chars();
        let (_, char) = prev_chars.next()?;
        self.offset = prev_chars.front;
        Some(char)
    }

    /// Goes to the next char in the line and returns it,
    /// or `None` if already at the last char.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let mut cursor = LineCursor::from_start(&line);
    /// assert!(cursor.offset() == 0);
    ///
    /// assert!(cursor.next_char() == Some('😍'));
    /// assert!(cursor.offset() == 4);
    ///
    /// assert!(cursor.next_char() == Some('🦀'));
    /// assert!(cursor.offset() == 8);
    ///
    /// assert!(cursor.next_char() == Some('\n'));
    /// assert!(cursor.offset() == 9);
    ///
    /// assert!(cursor.next_char() == None);
    /// assert!(cursor.offset() == 9);
    /// ```
    pub fn next_char(&mut self) -> Option<char> {
        let mut next_chars = self.next_chars();
        let (_, char) = next_chars.next()?;
        self.offset = next_chars.front;
        Some(char)
    }

    /// Goes to the start of the line.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let mut cursor = LineCursor::from_end(&line);
    /// assert!(cursor.offset() == 9);
    ///
    /// cursor.start();
    /// assert!(cursor.offset() == 0);
    /// ```
    pub fn start(&mut self) {
        self.offset = 0;
    }

    /// Goes to the end of the line.
    ///
    /// ```
    /// # use grappe::line::{Line, LineCursor};
    /// let line = Line::from("😍🦀");
    ///
    /// let mut cursor = LineCursor::from_start(&line);
    /// assert!(cursor.offset() == 0);
    ///
    /// cursor.end();
    /// assert!(cursor.offset() == 9);
    /// ```
    pub fn end(&mut self) {
        self.offset = self.len();
    }
}

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
            front: cursor.offset,
            back: 0,
            chars: if let Some(string) = string.get(..cursor.offset) {
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
