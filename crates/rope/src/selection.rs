use crate::{page::PageRef, text::TextRef, Chunk, Cursor, Index};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A selection of a [`Text`](crate::text::Text).
#[derive(Copy, Clone, Debug)]
pub struct Selection<'text> {
    pub(crate) start: Cursor<'text>,
    pub(crate) end: Cursor<'text>,
}

impl<'text> Selection<'text> {
    /// Returns the underlying [`TextRef`].
    pub fn text(&self) -> TextRef<'text> {
        self.start.text_ref
    }

    /// Returns `true` if `self` has a length of zero bytes.
    pub fn is_empty(&self) -> bool {
        self.start.index.byte < self.end.index.byte
    }

    /// Returns the byte length of this [`Selection`].
    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end.index.byte - self.start.index.byte
        }
    }

    /// Returns an iterator over the [`Chunk`]s of this [`Selection`].
    pub fn chunks(&self) -> Chunks<'text> {
        Chunks { selection: *self }
    }

    /// Returns the start [`Cursor`] of this [`Selection`].
    pub fn start(&self) -> Cursor<'text> {
        self.start
    }

    /// Returns the end [`Cursor`] of this [`Selection`].
    pub fn end(&self) -> Cursor<'text> {
        self.end
    }

    /// Returns a mutable reference to the start [`Cursor`] of this [`Selection`].
    pub fn start_mut(&mut self) -> &mut Cursor<'text> {
        &mut self.start
    }

    /// Returns a mutable reference to the end [`Cursor`] of this [`Selection`].
    pub fn end_mut(&mut self) -> &mut Cursor<'text> {
        &mut self.end
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Chunks                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An iterator over the [`Chunk`]s of a [`Selection`].
#[derive(Clone, Debug)]
pub struct Chunks<'text> {
    selection: Selection<'text>,
}

impl<'text> Iterator for Chunks<'text> {
    type Item = Chunk<'text>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
