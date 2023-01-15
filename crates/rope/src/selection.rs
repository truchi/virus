use crate::{page::PageRef, text::TextRef, Cursor, Index};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct Selection<'text> {
    pub(crate) start: Cursor<'text>,
    pub(crate) end: Cursor<'text>,
}

impl<'text> Selection<'text> {
    pub fn is_empty(&self) -> bool {
        self.start.index.byte < self.end.index.byte
    }

    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end.index.byte - self.start.index.byte
        }
    }

    pub fn chunks(&self) -> Chunks<'text> {
        Chunks { selection: *self }
    }

    pub fn start(&self) -> Cursor<'text> {
        self.start
    }

    pub fn end(&self) -> Cursor<'text> {
        self.end
    }

    pub fn start_mut(&mut self) -> &mut Cursor<'text> {
        &mut self.start
    }

    pub fn end_mut(&mut self) -> &mut Cursor<'text> {
        &mut self.end
    }
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
