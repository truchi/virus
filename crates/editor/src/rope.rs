use ropey::Rope;
use std::{cmp::Ordering::*, iter::Peekable, ops::Range};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
use unicode_width::UnicodeWidthChar;
use virus_common::Cursor;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            RopeExt                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Extensions for [`Rope`].
pub trait RopeExt {
    /// Returns the [`RopeExtCursorProxy`].
    fn cursor<'rope>(&'rope self) -> RopeExtCursorProxy<'rope>;

    /// Returns the [`RopeExtGraphemeProxy`].
    fn grapheme<'rope>(&'rope self) -> RopeExtGraphemeProxy<'rope>;

    /// Returns the [`RopeExtWordProxy`].
    fn word<'rope>(&'rope self) -> RopeExtWordProxy<'rope>;

    /// Replaces `selection` with `str`.
    fn replace(&mut self, selection: Range<Cursor>, str: &str);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       RopeExtCursorProxy                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct RopeExtCursorProxy<'rope>(&'rope Rope);

impl<'rope> RopeExtCursorProxy<'rope> {
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
//                                      RopeExtGraphemeProxy                                      //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct RopeExtGraphemeProxy<'rope>(&'rope Rope);

impl<'rope> RopeExtGraphemeProxy<'rope> {
    /// Returns whether the given `cursor` is a grapheme boundary.
    pub fn is_boundary(&self, cursor: Cursor) -> bool {
        // Bounds checks
        debug_assert!(cursor.index <= self.0.len_bytes());
        debug_assert!(cursor.line <= self.0.len_lines());

        // Get the chunk with our byte index in it
        let (chunk, offset, _, _) = self.0.chunk_at_byte(cursor.index);

        // Set up the grapheme cursor
        let mut graphemes = GraphemeCursor::new(cursor.index, self.0.len_bytes(), true);

        // Determine if the given position is a grapheme cluster boundary
        loop {
            match graphemes.is_boundary(chunk, offset) {
                Ok(is_boundary) => return is_boundary,
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let (chunk, offset, _, _) = self.0.chunk_at_byte(index - 1);
                    graphemes.provide_context(chunk, offset);
                }
                _ => unreachable!(),
            }
        }
    }

    /// Finds the previous grapheme boundary after the given `cursor`.
    pub fn before(&self, cursor: Cursor) -> Cursor {
        // Bounds checks
        debug_assert!(cursor.index <= self.0.len_bytes());
        debug_assert!(cursor.line <= self.0.len_lines());

        // Get the chunk with our byte index in it
        let (mut chunk, mut offset, _, _) = self.0.chunk_at_byte(cursor.index);

        // Set up the grapheme cursor
        let mut graphemes = GraphemeCursor::new(cursor.index, self.0.len_bytes(), true);

        // Find the previous grapheme cluster boundary
        loop {
            match graphemes.prev_boundary(chunk, offset) {
                Ok(None) => return self.0.cursor().start(),
                Ok(Some(index)) => return self.0.cursor().index(index),
                Err(GraphemeIncomplete::PrevChunk) => {
                    let (a, b, _, _) = self.0.chunk_at_byte(offset - 1);
                    chunk = a;
                    offset = b;
                }
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let chunk = self.0.chunk_at_byte(index - 1).0;
                    graphemes.provide_context(chunk, index - chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }

    /// Finds the next grapheme boundary after the given `cursor`.
    pub fn after(&self, cursor: Cursor) -> Cursor {
        // Bounds checks
        debug_assert!(cursor.index <= self.0.len_bytes());
        debug_assert!(cursor.line <= self.0.len_lines());

        // Get the chunk with our byte index in it
        let (mut chunk, mut offset, _, _) = self.0.chunk_at_byte(cursor.index);

        // Set up the grapheme cursor
        let mut graphemes = GraphemeCursor::new(cursor.index, self.0.len_bytes(), true);

        // Find the next grapheme cluster boundary
        loop {
            match graphemes.next_boundary(chunk, offset) {
                Ok(None) => return self.0.cursor().end(),
                Ok(Some(index)) => return self.0.cursor().index(index),
                Err(GraphemeIncomplete::NextChunk) => {
                    offset += chunk.len();
                    chunk = self.0.chunk_at_byte(offset).0;
                }
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let chunk = self.0.chunk_at_byte(index - 1).0;
                    graphemes.provide_context(chunk, index - chunk.len());
                }
                _ => unreachable!(),
            }
        }
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        RopeExtWordProxy                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct RopeExtWordProxy<'rope>(&'rope Rope);

impl<'rope> RopeExtWordProxy<'rope> {
    pub fn before(&self, cursor: Cursor) -> Cursor {
        todo!()
    }

    pub fn next_start(&self, cursor: Cursor) -> Cursor {
        self.next(cursor, false)
    }

    pub fn next_end(&self, cursor: Cursor) -> Cursor {
        self.next(cursor, true)
    }

    fn next(&self, cursor: Cursor, end: bool) -> Cursor {
        let mut chars = self.chars_at(cursor.index).peekable();

        while let Some(_) =
            chars.next_if(|(_, char)| Self::is_word(*char) || Self::is_special(*char))
        {}

        // FIXME
        if end {
            if let Some((index, _)) = chars.next() {
                if cursor.index != index {
                    return self.0.cursor().index(index);
                }
            }
        }

        while let Some((index, char)) = chars.next() {
            dbg!((index, char));
            if Self::is_special(char) {
                while let Some(_) = chars.next_if(|(_, char)| Self::is_special(*char)) {}

                if chars.peek().map(|(_, char)| Self::is_word(*char)) == Some(true) {
                    return self.0.cursor().index(index);
                };
            } else if Self::is_word(char) {
                println!("Word");
                return self.0.cursor().index(index);
            }
        }

        self.0.cursor().end()
    }

    pub fn test_next(&self, cursor: Cursor) -> Cursor {
        use Class::*;

        #[derive(PartialEq)]
        enum Class {
            Whitespace,
            Punctuation(char),
            Numeric,
            Lowercase,
            Uppercase,
        }

        impl From<char> for Class {
            fn from(char: char) -> Self {
                if char.is_whitespace() {
                    Self::Whitespace
                } else if char.is_ascii_punctuation() {
                    Self::Punctuation(char)
                } else if char.is_numeric() {
                    Self::Numeric
                } else if char.is_uppercase() {
                    Self::Uppercase
                } else {
                    Self::Lowercase
                }
            }
        }

        let mut chars = self
            .chars_at(cursor.index)
            .map(|(i, char)| (i, Class::from(char)))
            .peekable();
        let mut initial = if let Some((_, char)) = chars.next() {
            Class::from(char)
        } else {
            return cursor;
        };

        // Cursor before an uppercase is a special case
        if initial == Uppercase {
            match chars.peek() {
                // Followed by uppercases then lowercase,
                // return the index before the last uppercase
                Some((_, Uppercase)) => {
                    let prev = std::iter::from_fn(|| chars.next_if(|(_, class)| *class == initial))
                        .last()
                        .expect("Just peeked it")
                        .0;

                    if matches!(chars.peek(), Some((_, Lowercase))) {
                        return self.0.cursor().index(prev);
                    }
                }
                // Followed by a lowercase, pretend it was lowercase
                Some((_, Lowercase)) => initial = Lowercase,
                _ => {}
            }
        }

        // Remove chars until another class
        while let Some(_) = chars.next_if(|(_, class)| *class == initial) {}

        // Return that index, or end
        if let Some((index, _)) = chars.next() {
            self.0.cursor().index(index)
        } else {
            self.0.cursor().end()
        }
    }

    pub fn test_prev(&self, cursor: Cursor) -> Cursor {
        use Class::*;

        #[derive(PartialEq)]
        enum Class {
            Whitespace,
            Punctuation(char),
            Numeric,
            Lowercase,
            Uppercase,
        }

        impl From<char> for Class {
            fn from(char: char) -> Self {
                if char.is_whitespace() {
                    Self::Whitespace
                } else if char.is_ascii_punctuation() {
                    Self::Punctuation(char)
                } else if char.is_numeric() {
                    Self::Numeric
                } else if char.is_uppercase() {
                    Self::Uppercase
                } else {
                    Self::Lowercase
                }
            }
        }

        let mut chars = self
            .chars_at(cursor.index)
            .map(|(i, char)| (i, Class::from(char)))
            .peekable();
        let mut initial = if let Some((_, char)) = chars.next() {
            Class::from(char)
        } else {
            return cursor;
        };

        // Cursor before an uppercase is a special case
        if initial == Uppercase {
            match chars.peek() {
                // Followed by uppercases then lowercase,
                // return the index before the last uppercase
                Some((_, Uppercase)) => {
                    let prev = std::iter::from_fn(|| chars.next_if(|(_, class)| *class == initial))
                        .last()
                        .expect("Just peeked it")
                        .0;

                    if matches!(chars.peek(), Some((_, Lowercase))) {
                        return self.0.cursor().index(prev);
                    }
                }
                // Followed by a lowercase, pretend it was lowercase
                Some((_, Lowercase)) => initial = Lowercase,
                _ => {}
            }
        }

        // Remove chars until another class
        while let Some(_) = chars.next_if(|(_, class)| *class == initial) {}

        // Return that index, or end
        if let Some((index, _)) = chars.next() {
            self.0.cursor().index(index)
        } else {
            self.0.cursor().end()
        }
    }

    fn chars_at(&self, index: usize) -> impl '_ + Iterator<Item = (usize, char)> {
        let (chunks, first_chunk_index, ..) = self.0.chunks_at_byte(index);
        chunks
            .into_iter()
            .enumerate()
            .map(move |(i, chunk)| {
                if i == 0 {
                    &chunk[index - first_chunk_index..]
                } else {
                    chunk
                }
            })
            .scan(index, |next_chunk_index, chunk| {
                let chunk_index = *next_chunk_index;
                *next_chunk_index += chunk.len();
                Some((chunk_index, chunk))
            })
            .flat_map(|(chunk_index, chunk)| {
                chunk
                    .char_indices()
                    .map(move |(i, char)| (chunk_index + i, char))
            })
    }

    fn is_word(char: char) -> bool {
        char.is_ascii_alphanumeric() || !char.is_ascii()
    }

    fn is_special(char: char) -> bool {
        char == '-' || char == '_'
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         Implementation                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

impl RopeExt for Rope {
    fn cursor<'rope>(&'rope self) -> RopeExtCursorProxy<'rope> {
        RopeExtCursorProxy(self)
    }

    fn grapheme<'rope>(&'rope self) -> RopeExtGraphemeProxy<'rope> {
        RopeExtGraphemeProxy(self)
    }

    fn word<'rope>(&'rope self) -> RopeExtWordProxy<'rope> {
        RopeExtWordProxy(self)
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
        rope.grapheme().before(cursor)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Tests                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use unicode_segmentation::UnicodeSegmentation;

    const HUGE_GRAPHEMES: &str = "Hẽ̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃llõ̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃ wõ̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃̃rld!";
    const REGIONAL_SYMBOLS: &str = "🇬🇧🇯🇵🇺🇸🇫🇷🇷🇺🇨🇳🇩🇪🇪🇸🇬🇧🇯🇵🇺🇸🇫🇷🇷🇺🇨🇳🇩🇪🇪🇸🇬🇧🇯🇵🇺🇸🇫🇷🇷🇺🇨🇳🇩🇪🇪";

    #[test]
    fn graphemes() {
        for str in [HUGE_GRAPHEMES, REGIONAL_SYMBOLS] {
            assert!(
                str.lines().count() == 1,
                "This test assumes one-line inputs",
            );

            let rope = Rope::from_str(str);
            let indices = str
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();

            for (i, &index) in indices.iter().enumerate() {
                // Prev grapheme
                if i != 0 {
                    assert_eq!(
                        rope.grapheme().before(Cursor::new(index, 0, index)),
                        Cursor::new(indices[i - 1], 0, indices[i - 1]),
                    );
                }

                // Next grapheme
                if i != indices.len() - 1 {
                    assert_eq!(
                        rope.grapheme().after(Cursor::new(index, 0, index)),
                        Cursor::new(indices[i + 1], 0, indices[i + 1]),
                    );
                }
            }

            // Is grapheme
            for (i, _) in str.char_indices() {
                assert_eq!(
                    rope.grapheme().is_boundary(Cursor::new(i, 0, i)),
                    indices.contains(&i)
                );
            }
        }
    }
}
