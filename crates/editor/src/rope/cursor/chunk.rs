use ropey::{iter::Chunks, RopeSlice};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          ChunkCursor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Cursor in a `RopeSlice`'s chunks.
pub struct ChunkCursor<'rope> {
    chunks: Chunks<'rope>,
    index: usize,
}

impl<'rope> ChunkCursor<'rope> {
    pub fn new(slice: RopeSlice<'rope>, index: usize) -> Self {
        let (chunks, index, ..) = slice.chunks_at_byte(index);

        Self { chunks, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn prev(&mut self) -> Option<(usize, &'rope str)> {
        let chunk = self.chunks.prev()?;

        self.index -= chunk.len();
        Some((self.index, chunk))
    }

    pub fn next(&mut self) -> Option<(usize, &'rope str)> {
        let chunk = self.chunks.next()?;
        let index = self.index;

        self.index += chunk.len();
        Some((index, chunk))
    }
}
