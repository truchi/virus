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
            text_ref: self.text_ref,
            start_page_ref: self.page_ref,
            start_page: self.page,
            start_index: self.index,
            end_page_ref: self.page_ref,
            end_page: self.page,
            end_index: self.index,
        }
    }

    pub fn index(&self) -> Index {
        self.index
    }

    pub fn is_char_boundary(&self) -> bool {
        true
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

    pub fn to<I: CursorIndex>(&self, index: I) -> Option<Cursor<'text>> {
        index.cursor_from_cursor(*self)
    }

    pub fn to_line(&self, line: usize) -> Option<Cursor<'text>> {
        self.to((line, 0))
    }

    pub fn to_column(&self, column: usize) -> Option<Cursor<'text>> {
        self.to((self.index.line, column))
    }

    pub fn to_prev_char_boundary(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_next_char_boundary(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_prev_grapheme_boundary(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_next_grapheme_boundary(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_prev_line_boundary(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_next_line_boundary(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_start(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_end(&self) -> Option<Cursor<'text>> {
        todo!()
    }

    pub fn to_mut<I: CursorIndex>(&mut self, index: I) -> bool {
        todo!()
    }

    pub fn to_line_mut(&mut self, line: usize) -> bool {
        self.to_mut((line, 0))
    }

    pub fn to_column_mut(&mut self, column: usize) -> bool {
        self.to_mut((self.index.line, column))
    }

    pub fn to_prev_char_boundary_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_next_char_boundary_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_prev_grapheme_boundary_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_next_grapheme_boundary_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_prev_line_boundary_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_next_line_boundary_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_start_mut(&mut self) -> bool {
        todo!()
    }

    pub fn to_end_mut(&mut self) -> bool {
        todo!()
    }
}
