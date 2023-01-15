use crate::{page::PageRef, text::TextRef, CursorIndex, Index, Selection};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Cursor                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct Cursor<'text> {
    pub(crate) text_ref: TextRef<'text>,
    pub(crate) page_ref: PageRef<'text>,
    pub(crate) page: usize,
    pub(crate) index: Index,
}

impl<'text> Cursor<'text> {
    pub fn selection(&self) -> Selection<'text> {
        Selection {
            start: *self,
            end: *self,
        }
    }

    pub fn index(&self) -> Index {
        self.index
    }

    pub fn is_grapheme_boundary(&self) -> bool {
        todo!()
    }

    pub fn is_line_boundary(&self) -> bool {
        todo!()
    }

    pub fn is_start(&self) -> bool {
        todo!()
    }

    pub fn is_end(&self) -> bool {
        todo!()
    }

    pub fn to<I: CursorIndex>(&self, index: I) -> Cursor<'text> {
        index.cursor_from_cursor(*self)
    }

    pub fn to_line(&self, line: usize) -> Cursor<'text> {
        self.to((line, 0))
    }

    pub fn to_column(&self, column: usize) -> Cursor<'text> {
        self.to((self.index.line, column))
    }

    pub fn to_prev_char(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_next_char(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_prev_grapheme(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_next_grapheme(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_prev_line(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_next_line(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_start(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_end(&self) -> Cursor<'text> {
        todo!()
    }

    pub fn to_mut<I: CursorIndex>(&mut self, index: I) {
        todo!()
    }

    pub fn to_line_mut(&mut self, line: usize) {
        self.to_mut((line, 0))
    }

    pub fn to_column_mut(&mut self, column: usize) {
        self.to_mut((self.index.line, column))
    }

    pub fn to_prev_char_mut(&mut self) {
        todo!()
    }

    pub fn to_next_char_mut(&mut self) {
        todo!()
    }

    pub fn to_prev_grapheme_mut(&mut self) {
        todo!()
    }

    pub fn to_next_grapheme_mut(&mut self) {
        todo!()
    }

    pub fn to_prev_line_mut(&mut self) {
        todo!()
    }

    pub fn to_next_line_mut(&mut self) {
        todo!()
    }

    pub fn to_start_mut(&mut self) {
        todo!()
    }

    pub fn to_end_mut(&mut self) {
        todo!()
    }
}
