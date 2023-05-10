use ropey::Rope;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
use virus_common::Cursor;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              RopeExt                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait RopeExt {
    /// Returns a cursor at byte `index`.
    fn cursor_at_index(&self, index: usize) -> Cursor;

    /// Returns a cursor at the end of the text.
    fn cursor_at_end(&self) -> Cursor;

    /// Finds the previous grapheme boundary after the given `cursor`.
    fn prev_grapheme(&self, cursor: Cursor) -> Cursor;

    /// Finds the next grapheme boundary after the given `cursor`.
    fn next_grapheme(&self, cursor: Cursor) -> Cursor;

    /// Returns whether the given `cursor` is a grapheme boundary.
    fn is_grapheme_boundary(&self, cursor: Cursor) -> bool;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Implementations                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

impl RopeExt for Rope {
    fn cursor_at_index(&self, index: usize) -> Cursor {
        let line = self.byte_to_line(index);

        Cursor {
            index,
            line,
            column: index - self.line_to_byte(line),
        }
    }

    fn cursor_at_end(&self) -> Cursor {
        let index = self.len_bytes();
        let line = self.len_lines() - 1;

        Cursor {
            index,
            line,
            column: index - self.line_to_byte(line),
        }
    }

    fn prev_grapheme(&self, cursor: Cursor) -> Cursor {
        // Bounds checks
        debug_assert!(cursor.index <= self.len_bytes());
        debug_assert!(cursor.line <= self.len_lines());

        // Get the chunk with our byte index in it
        let (mut chunk, mut offset, _, _) = self.chunk_at_byte(cursor.index);

        // Set up the grapheme cursor
        let mut graphemes = GraphemeCursor::new(cursor.index, self.len_bytes(), true);

        // Find the previous grapheme cluster boundary
        loop {
            match graphemes.prev_boundary(chunk, offset) {
                Ok(None) => return Cursor::ZERO,
                Ok(Some(index)) => return self.cursor_at_index(index),
                Err(GraphemeIncomplete::PrevChunk) => {
                    let (a, b, _, _) = self.chunk_at_byte(offset - 1);
                    chunk = a;
                    offset = b;
                }
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let chunk = self.chunk_at_byte(index - 1).0;
                    graphemes.provide_context(chunk, index - chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }

    fn next_grapheme(&self, cursor: Cursor) -> Cursor {
        // Bounds checks
        debug_assert!(cursor.index <= self.len_bytes());
        debug_assert!(cursor.line <= self.len_lines());

        // Get the chunk with our byte index in it
        let (mut chunk, mut offset, _, _) = self.chunk_at_byte(cursor.index);

        // Set up the grapheme cursor
        let mut graphemes = GraphemeCursor::new(cursor.index, self.len_bytes(), true);

        // Find the next grapheme cluster boundary
        loop {
            match graphemes.next_boundary(chunk, offset) {
                Ok(None) => return self.cursor_at_end(),
                Ok(Some(index)) => return self.cursor_at_index(index),

                Err(GraphemeIncomplete::NextChunk) => {
                    offset += chunk.len();
                    chunk = self.chunk_at_byte(offset).0;
                }
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let chunk = self.chunk_at_byte(index - 1).0;
                    graphemes.provide_context(chunk, index - chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }

    fn is_grapheme_boundary(&self, cursor: Cursor) -> bool {
        // Bounds checks
        debug_assert!(cursor.index <= self.len_bytes());
        debug_assert!(cursor.line <= self.len_lines());

        // Get the chunk with our byte index in it
        let (chunk, offset, _, _) = self.chunk_at_byte(cursor.index);

        // Set up the grapheme cursor
        let mut graphemes = GraphemeCursor::new(cursor.index, self.len_bytes(), true);

        // Determine if the given position is a grapheme cluster boundary
        loop {
            match graphemes.is_boundary(chunk, offset) {
                Ok(n) => return n,
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let (chunck, offset, _, _) = self.chunk_at_byte(index - 1);
                    graphemes.provide_context(chunck, offset);
                }
                _ => unreachable!(),
            }
        }
    }
}

impl RopeExt for &Rope {
    fn cursor_at_index(&self, index: usize) -> Cursor {
        (*self).cursor_at_index(index)
    }

    fn cursor_at_end(&self) -> Cursor {
        (*self).cursor_at_end()
    }

    fn prev_grapheme(&self, cursor: Cursor) -> Cursor {
        (*self).prev_grapheme(cursor)
    }

    fn next_grapheme(&self, cursor: Cursor) -> Cursor {
        (*self).next_grapheme(cursor)
    }

    fn is_grapheme_boundary(&self, cursor: Cursor) -> bool {
        (*self).is_grapheme_boundary(cursor)
    }
}
