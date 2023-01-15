use crate::{page::PageRef, text::TextRef, CursorIndex, Index, Selection};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Cursor                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A cursor in a [`Text`](crate::text::Text).
#[derive(Copy, Clone, Debug)]
pub struct Cursor<'text> {
    /// Underlying text.
    pub(crate) text_ref: TextRef<'text>,
    /// Current page.
    pub(crate) page_ref: PageRef<'text>,
    /// Current page index.
    pub(crate) page: usize,
    /// Current index.
    pub(crate) index: Index,
}

impl<'text> Cursor<'text> {
    /// Returns a [`CursorMut`] from this [`Cursor`].
    pub fn as_mut(&mut self) -> CursorMut<'text, '_> {
        CursorMut { cursor: self }
    }

    /// Returns the underlying [`TextRef`].
    pub fn text(&self) -> TextRef<'text> {
        self.text_ref
    }

    /// Returns an empty [`Selection`] at this [`Cursor`].
    pub fn selection(&self) -> Selection<'text> {
        Selection {
            start: *self,
            end: *self,
        }
    }

    /// Returns the current [`Index`].
    pub fn index(&self) -> Index {
        self.index
    }

    /// Returns `true` if the current position is a grapheme boundary.
    pub fn is_grapheme_boundary(&self) -> bool {
        todo!()
    }

    /// Returns `true` if the current position is a line boundary.
    pub fn is_line_boundary(&self) -> bool {
        todo!()
    }

    /// Returns `true` if the current position is the start of the [`Text`](crate::text::Text).
    pub fn is_start(&self) -> bool {
        todo!()
    }

    /// Returns `true` if the current position is the end of the [`Text`](crate::text::Text).
    pub fn is_end(&self) -> bool {
        todo!()
    }

    /// Returns a [`Cursor`] at `index`.
    pub fn to<I: CursorIndex>(&self, index: I) -> Cursor<'text> {
        index.cursor_from_cursor(*self)
    }

    /// Returns a [`Cursor`] at the beggining of `line`.
    pub fn to_line(&self, line: usize) -> Cursor<'text> {
        self.to((line, 0))
    }

    /// Returns a [`Cursor`] at `column` on the current line.
    pub fn to_column(&self, column: usize) -> Cursor<'text> {
        self.to((self.index.line, column))
    }

    /// Returns a [`Cursor`] at the previous char.
    pub fn to_prev_char(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the next char.
    pub fn to_next_char(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the previous grapheme boundary.
    pub fn to_prev_grapheme(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the next grapheme boundary.
    pub fn to_next_grapheme(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the previous line boundary.
    pub fn to_prev_line(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the next line boundary.
    pub fn to_next_line(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the start of the [`Text`](crate::text::Text).
    pub fn to_start(&self) -> Cursor<'text> {
        todo!()
    }

    /// Returns a [`Cursor`] at the end of the [`Text`](crate::text::Text).
    pub fn to_end(&self) -> Cursor<'text> {
        todo!()
    }

    /// Moves this [`Cursor`] to `index`.
    pub fn to_mut<I: CursorIndex>(&mut self, index: I) {
        todo!()
    }

    /// Moves this [`Cursor`] at the beggining of `line`.
    pub fn to_line_mut(&mut self, line: usize) {
        self.to_mut((line, 0))
    }

    /// Moves this [`Cursor`] at `column` on the current line.
    pub fn to_column_mut(&mut self, column: usize) {
        self.to_mut((self.index.line, column))
    }

    /// Moves this [`Cursor`] at the previous char.
    pub fn to_prev_char_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the next char.
    pub fn to_next_char_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the previous grapheme boundary.
    pub fn to_prev_grapheme_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the next grapheme boundary.
    pub fn to_next_grapheme_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the previous line boundary.
    pub fn to_prev_line_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the next line boundary.
    pub fn to_next_line_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the start of the [`Text`](crate::text::Text).
    pub fn to_start_mut(&mut self) {
        todo!()
    }

    /// Moves this [`Cursor`] at the end of the [`Text`](crate::text::Text).
    pub fn to_end_mut(&mut self) {
        todo!()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            CursorMut                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A mutable [`Cursor`].
#[derive(Debug)]
pub struct CursorMut<'text, 'cursor> {
    /// Cursor.
    pub(crate) cursor: &'cursor mut Cursor<'text>,
}

impl<'text, 'cursor> CursorMut<'text, 'cursor> {
    /// Moves this [`Cursor`] to `index`.
    fn to<I: CursorIndex>(&mut self, index: I) {
        self.cursor.to_mut(index)
    }

    /// Moves this [`Cursor`] at the beggining of `line`.
    fn to_line(&mut self, line: usize) {
        self.cursor.to_line_mut(line)
    }

    /// Moves this [`Cursor`] at `column` on the current line.
    fn to_column(&mut self, column: usize) {
        self.cursor.to_column_mut(column)
    }

    /// Moves this [`Cursor`] at the previous char.
    fn to_prev_char(&mut self) {
        self.cursor.to_prev_char_mut()
    }

    /// Moves this [`Cursor`] at the next char.
    fn to_next_char(&mut self) {
        self.cursor.to_next_char_mut()
    }

    /// Moves this [`Cursor`] at the previous grapheme boundary.
    fn to_prev_grapheme(&mut self) {
        self.cursor.to_prev_grapheme_mut()
    }

    /// Moves this [`Cursor`] at the next grapheme boundary.
    fn to_next_grapheme(&mut self) {
        self.cursor.to_next_grapheme_mut()
    }

    /// Moves this [`Cursor`] at the previous line boundary.
    fn to_prev_line(&mut self) {
        self.cursor.to_prev_line_mut()
    }

    /// Moves this [`Cursor`] at the next line boundary.
    fn to_next_line(&mut self) {
        self.cursor.to_next_line_mut()
    }

    /// Moves this [`Cursor`] at the start of the [`Text`](crate::text::Text).
    fn to_start(&mut self) {
        self.cursor.to_start_mut()
    }

    /// Moves this [`Cursor`] at the end of the [`Text`](crate::text::Text).
    fn to_end(&mut self) {
        self.cursor.to_end_mut()
    }
}
