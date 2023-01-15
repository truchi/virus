use crate::{page::PageRef, text::TextRef, Cursor, Index};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct Selection<'text> {
    pub(crate) text_ref: TextRef<'text>,
    pub(crate) start_page_ref: PageRef<'text>,
    pub(crate) start_page: usize,
    pub(crate) start_index: Index,
    pub(crate) end_page_ref: PageRef<'text>,
    pub(crate) end_page: usize,
    pub(crate) end_index: Index,
}

impl<'text> Selection<'text> {
    pub fn chunks(&self) -> Chunks<'text> {
        Chunks { selection: *self }
    }

    pub fn start(&self) -> Cursor<'text> {
        Cursor {
            text_ref: self.text_ref,
            page_ref: self.start_page_ref,
            page: self.start_page,
            index: self.start_index,
        }
    }

    pub fn end(&self) -> Cursor<'text> {
        Cursor {
            text_ref: self.text_ref,
            page_ref: self.end_page_ref,
            page: self.end_page,
            index: self.end_index,
        }
    }

    pub fn start_mut(&mut self) -> StartMut<'text, '_> {
        StartMut { selection: self }
    }

    pub fn end_mut(&mut self) -> EndMut<'text, '_> {
        EndMut { selection: self }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            StartMut                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct StartMut<'text, 'selection> {
    selection: &'selection mut Selection<'text>,
}

impl<'text, 'selection> StartMut<'text, 'selection> {
    // All cursor functions, but ensure start <= end
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             EndMut                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct EndMut<'text, 'selection> {
    selection: &'selection mut Selection<'text>,
}

impl<'text, 'selection> EndMut<'text, 'selection> {
    // All cursor functions, but ensure start <= end
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Chunks                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Chunks<'text> {
    selection: Selection<'text>,
}

impl<'text> Iterator for Chunks<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
