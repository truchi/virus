use crate::{page::PageRef, text::TextRef, Chunk, Cursor, CursorMut, Index};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A selection of a [`Text`](crate::text::Text).
#[derive(Copy, Clone, Debug)]
pub struct Selection<'text> {
    /// Start cursor.
    pub(crate) start: Cursor<'text>,
    /// End cursor.
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
        Chunks {
            start: self.start,
            end: self.end,
        }
    }

    /// Returns the start [`Cursor`] of this [`Selection`].
    pub fn start(&self) -> Cursor<'text> {
        self.start
    }

    /// Returns the end [`Cursor`] of this [`Selection`].
    pub fn end(&self) -> Cursor<'text> {
        self.end
    }

    /// Returns the start [`CursorMut`] of this [`Selection`].
    pub fn start_mut(&mut self) -> CursorMut<'text, '_> {
        CursorMut {
            cursor: &mut self.start,
        }
    }

    /// Returns the end [`CursorMut`] of this [`Selection`].
    pub fn end_mut(&mut self) -> CursorMut<'text, '_> {
        CursorMut {
            cursor: &mut self.end,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Chunks                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An iterator over the [`Chunk`]s of a [`Selection`].
#[derive(Clone, Debug)]
pub struct Chunks<'text> {
    pub(crate) start: Cursor<'text>, // /!\ Some fields not updated
    pub(crate) end: Cursor<'text>,   // /!\ Some fields not updated
}

impl<'text> Iterator for Chunks<'text> {
    type Item = Chunk<'text>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.index.byte == self.end.index.byte {
            return None;
        }

        if self.start.is_page_boundary() {
            let page = self.start.page_ref;
            let chunk = Chunk {
                str: page.as_str(),
                feeds: page.feeds as usize,
                byte: page.byte,
                line: page.feed,
            };

            // ???
            self.start.page += 1;

            Some(chunk)
        } else {
            None
        }
    }
}
