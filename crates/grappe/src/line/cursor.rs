use super::{Line, LineNextChars, LinePrevChars};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            LineCursor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Line`].
#[derive(Copy, Clone, Debug)]
pub struct LineCursor<'a> {
    pub(super) string: &'a str,
    pub(super) offset: usize,
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
        self.offset = prev_chars.front();
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
        self.offset = next_chars.front();
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
