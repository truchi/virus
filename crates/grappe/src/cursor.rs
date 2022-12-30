use crate::{line::Line, text::Text};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Cursor                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct Cursor<'a> {
    /// Lines in the text.
    lines: &'a [Line],
    /// Current line.
    /// Is empty when:
    /// - line is "\n"
    /// - text is empty
    /// - cursor is at the end of the text
    string: &'a str,
    /// Byte index in the text. `0..=text.len()`.
    /// Equals `text.len()` when cursor is at the end of the text.
    index: usize,
    /// Line index in the text. `0..=lines.len()`.
    /// Equals `lines.len()` when cursor is at the end of the text.
    line: usize,
    /// Byte index in the line. `0..line.len()`.
    /// Equals `0` when:
    /// - cursor is at the start of the line
    /// - cursor is at the end of the text
    column: usize,
    /// Byte length of the text.
    len: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a new [`Cursor`] at the start of `text`.
    pub fn new(text: &'a Text) -> Self {
        let lines = text.lines();
        let string = lines
            .get(0)
            .map(|line| &line.string()[..])
            .unwrap_or_default();

        Self {
            lines,
            string,
            index: 0,
            line: 0,
            column: 0,
            len: text.len(),
        }
    }

    /// Returns the lines of the text.
    pub fn lines(&self) -> &[Line] {
        self.lines
    }

    /// Returns the byte index of this [`Cursor`] in the text.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the line index of this [`Cursor`] in the text.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the byte index of this [`Cursor`] in the line.
    pub fn column(&self) -> usize {
        self.column
    }

    pub fn prev_stuff(&mut self) -> impl '_ + Iterator<Item = (Cursor<'a>, &'a str)> {
        std::iter::from_fn(|| {
            self.index = 0;

            Some((self.clone(), self.string))
        })
    }

    /// Goes to the previous `char` and returns it,
    /// or `None` if on the first `char` already.
    pub fn prev_char(&mut self) -> Option<char> {
        if self.is_start() {
            None
        } else {
            let char = self.string[0..self.column].chars().rev().next();

            if let Some(char) = char {
                self.index -= char.len_utf8();
                self.column -= char.len_utf8();

                Some(char)
            } else {
                self.line -= 1;
                self.string = self.string(self.line);
                self.index -= /* newline */ 1;
                self.column = self.string.len();

                Some('\n')
            }
        }
    }

    /// Goes to the next `char` and returns it,
    /// or `None` if on the last `char` already.
    pub fn next_char(&mut self) -> Option<char> {
        let char = self.string[self.column..].chars().next();

        if let Some(char) = char {
            self.index += char.len_utf8();
            self.column += char.len_utf8();
        } else {
            self.line += 1;
            self.string = self.string(self.line);
            self.index += /* newline */ 1;
            self.column = 0;
        }

        None
    }

    /// Goes to the previous grapheme and returns it,
    /// or `None` if on the first grapheme already.
    pub fn prev_grapheme(&mut self) -> Option<&'a str> {
        todo!()
    }

    /// Goes to the next grapheme and returns it,
    /// or `None` if on the last grapheme already.
    pub fn next_grapheme(&mut self) -> Option<&'a str> {
        todo!()
    }

    /// Goes to the start of the previous line and returns it,
    /// or returns `false` if on the first line already.
    pub fn prev_line(&mut self) -> Option<&'a Line> {
        if self.line != 0 {
            self.line -= 1;
            self.string = self.string(self.line);
            self.index -= self.string.len() + /* newline */ 1 + self.column;
            self.column = 0;

            Some(&self.lines[self.line])
        } else {
            None
        }
    }

    /// Goes to the start of the next line and returs it,
    /// or returns `false` if on the last line already.
    pub fn next_line(&mut self) -> Option<&'a Line> {
        if self.line + 1 < self.lines.len() {
            self.line += 1;
            self.index += self.string.len() + /* newline */ 1 - self.column;
            self.column = 0;
            self.string = self.string(self.line);

            Some(&self.lines[self.line])
        } else {
            None
        }
    }

    /// Goes to the start of the text.
    pub fn start(&mut self) {
        self.string = self.string(0);
        self.line = 0;
        self.index = 0;
        self.column = 0;
    }

    /// Goes to the end of the text.
    pub fn end(&mut self) {
        self.string = "";
        self.line = self.lines.len();
        self.index = self.len;
        self.column = 0;
    }

    /// Returns whether this [`Cursor`] is on a `char` boundary, i.e. `true`.
    pub fn is_char_boundary(&self) -> bool {
        todo!()
    }

    /// Returns whether this [`Cursor`] is on a grapheme boundary.
    pub fn is_grapheme_boundary(&self) -> bool {
        todo!()
    }

    /// Returns whether this [`Cursor`] is on a line boundary.
    pub fn is_line_boundary(&self) -> bool {
        self.column == 0
    }

    /// Returns whether this [`Cursor`] is at the start of the text.
    pub fn is_start(&self) -> bool {
        self.index == 0
    }

    /// Returns whether this [`Cursor`] is at the end of the text.
    pub fn is_end(&self) -> bool {
        self.index == self.len
    }
}

impl<'a> Cursor<'a> {
    fn string(&self, index: usize) -> &'a str {
        self.lines
            .get(index)
            .map(|line| &line.string()[..])
            .unwrap_or_default()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            PrevChars                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct PrevChars<'a> {
    cursor: Cursor<'a>,
    chars: std::iter::Rev<std::iter::Chain<std::str::Chars<'a>, std::option::IntoIter<char>>>,
}

impl<'a> From<Cursor<'a>> for PrevChars<'a> {
    fn from(cursor: Cursor<'a>) -> Self {
        let string = cursor.string;
        let chars = string
            .chars()
            .chain((cursor.column == string.len() + /* newline */ 1).then_some('\n'))
            .rev();

        Self { cursor, chars }
    }
}

impl<'a> Iterator for PrevChars<'a> {
    type Item = (Cursor<'a>, char);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(char) = self.chars.next() {
            self.cursor.index -= char.len_utf8();
            self.cursor.column -= char.len_utf8();

            Some((self.cursor, char))
        } else if let Some(line) = self.cursor.line.checked_sub(1) {
            self.cursor.line = line;
            self.cursor.string = self.cursor.string(line);
            self.cursor.column = self.cursor.string.len() + /* newline */ 1;
            *self = Self::from(self.cursor);

            self.next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;

    struct Data {
        text: Text,
        lines: Vec<Line>,
        chars: Vec<char>,
        graphemes: Vec<String>,
    }

    fn data<T: AsRef<str>>(str: T) -> Data {
        let str = str.as_ref();
        let text = Text::from(str);
        let lines = text.lines().as_ref().clone();
        let chars = str.chars().collect();
        let graphemes = str.graphemes(true).map(Into::into).collect();
        Data {
            text,
            lines,
            chars,
            graphemes,
        }
    }

    fn prev_chars(cursor: &mut Cursor) -> Vec<char> {
        let mut chars = vec![];
        while let Some(char) = cursor.prev_char() {
            chars.push(char);
        }
        chars
    }

    fn next_chars(cursor: &mut Cursor) -> Vec<char> {
        let mut chars = vec![];
        while let Some(char) = cursor.next_char() {
            chars.push(char);
        }
        chars
    }

    fn prev_graphemes(cursor: &mut Cursor) -> Vec<String> {
        let mut graphemes = vec![];
        while let Some(grapheme) = cursor.prev_grapheme() {
            graphemes.push(grapheme.into());
        }
        graphemes
    }

    fn next_graphemes(cursor: &mut Cursor) -> Vec<String> {
        let mut graphemes = vec![];
        while let Some(grapheme) = cursor.next_grapheme() {
            graphemes.push(grapheme.into());
        }
        graphemes
    }

    fn prev_lines(cursor: &mut Cursor) -> Vec<Line> {
        let mut lines = vec![];
        while let Some(line) = cursor.prev_line() {
            lines.push(line.clone());
        }
        lines
    }

    fn next_lines(cursor: &mut Cursor) -> Vec<Line> {
        let mut lines = vec![];
        while let Some(line) = cursor.next_line() {
            lines.push(line.clone());
        }
        lines
    }

    // #[test]
    // #[ignore = "TODO"]
    fn test() {
        for data in [
            data(""),
            data("hello"),
            data("hello\nworld"),
            data(format!("Hello 🦀\n👩\u{200D}🔬\n")),
        ] {
            let text = data.text;
            let mut cursor = Cursor::new(&text);

            assert!(cursor.prev_char().is_none());
            assert!(cursor.prev_grapheme().is_none());
            assert!(cursor.prev_line().is_none());
        }
    }
}
