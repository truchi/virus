use crate::rope::ChunkCursor;
use ropey::RopeSlice;
use std::ops::{
    Bound::{self, *},
    Range, RangeBounds,
};
use unicode_segmentation::GraphemeIncomplete;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         GraphemeCursor                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Cursor in a `RopeSlice`'s graphemes.
pub struct GraphemeCursor<'rope> {
    /// The rope slice we are operating on.
    slice: RopeSlice<'rope>,
    /// The chunk cursor of that slice.
    chunks: ChunkCursor<'rope>,
    /// The current chunk.
    chunk: Option<(usize, &'rope str)>,
    /// Whether `self.chunks` is:
    /// -  after `self.chunk` (`self.chunk = self.chunks.next()` was last)
    /// - before `self.chunk` (`self.chunk = self.chunks.prev()` was last)
    is_chunks_after_chunk: bool,
    /// The underlying unicode grapheme cursor.
    graphemes: unicode_segmentation::GraphemeCursor,
}

impl<'rope> GraphemeCursor<'rope> {
    pub fn new(slice: RopeSlice<'rope>, index: usize) -> Self {
        let mut chunks = ChunkCursor::new(slice, index);
        let chunk = chunks.next();

        Self {
            slice,
            chunks,
            chunk,
            is_chunks_after_chunk: true,
            graphemes: unicode_segmentation::GraphemeCursor::new(index, slice.len_bytes(), true),
        }
    }

    pub fn slice(&self) -> RopeSlice<'rope> {
        self.slice
    }

    pub fn index(&self) -> usize {
        self.graphemes.cur_cursor()
    }

    pub fn set_index(&mut self, index: usize) {
        let index = index.min(self.slice.len_bytes());

        if self.index() != index {
            self.graphemes.set_cursor(index);
            self.goto_index();
        }
    }

    pub fn is_boundary(&self) -> bool {
        let Some((chunk_start, chunk)) = self.chunk else {
            return true;
        };

        let mut graphemes = self.graphemes.clone();
        loop {
            match graphemes.is_boundary(chunk, chunk_start) {
                Ok(is_boundary) => return is_boundary,
                Err(GraphemeIncomplete::PreContext(index)) => {
                    let (chunk, chunk_start, ..) = self.slice.chunk_at_byte(index - 1);
                    graphemes.provide_context(chunk, chunk_start);
                }
                _ => unreachable!("Not returned by `is_boundary()`"),
            }
        }
    }

    pub fn prev(&mut self) -> Option<(Range<usize>, impl Iterator<Item = (usize, &'rope str)>)> {
        // `self.chunks` must be before `self.chunk` and will be through the end of this function
        if self.is_chunks_after_chunk {
            self.chunk = self.chunks.prev();
            self.is_chunks_after_chunk = false;
        }

        // Find the prev grapheme boundary
        let (chunk_start, chunk) = self.chunk?;
        let grapheme = Range {
            end: self.graphemes.cur_cursor(),
            start: {
                let (mut chunk_start, mut chunk) = (chunk_start, chunk);

                // FIXME: quick fix for Self::new(..., 0).prev()
                if self.graphemes.cur_cursor() == 0 {
                    return None;
                }

                loop {
                    match self.graphemes.prev_boundary(chunk, chunk_start) {
                        Ok(None) => unreachable!("`self.chunk` is `None` already"),
                        Ok(Some(boundary)) => break boundary,
                        Err(GraphemeIncomplete::PrevChunk) => {
                            (chunk, chunk_start, ..) = self.slice.chunk_at_byte(chunk_start - 1);
                        }
                        Err(GraphemeIncomplete::PreContext(index)) => {
                            let (chunk, chunk_start, ..) = self.slice.chunk_at_byte(index - 1);
                            self.graphemes.provide_context(chunk, chunk_start);
                        }
                        _ => unreachable!("Not returned by `prev_boundary()`"),
                    }
                }
            },
        };
        let in_chunk = grapheme.start >= chunk_start;

        debug_assert!(self.graphemes.cur_cursor() == grapheme.start);
        self.goto_index();
        Some((
            grapheme.start..grapheme.end,
            self.grapheme_chunks(in_chunk, grapheme, chunk, chunk_start),
        ))
    }

    pub fn next(&mut self) -> Option<(Range<usize>, impl Iterator<Item = (usize, &'rope str)>)> {
        // `self.chunks` must be after `self.chunk` and will be through the end of this function
        if !self.is_chunks_after_chunk {
            self.chunk = self.chunks.next();
            self.is_chunks_after_chunk = true;
        }

        // Find the next grapheme boundary
        // FIXME: `next_boundary()` panics if its cursor is at the end of the given chunk!
        //        (see https://github.com/unicode-rs/unicode-segmentation/issues/135)
        let (chunk_start, chunk) = self.chunk?;
        let grapheme = Range {
            start: self.graphemes.cur_cursor(),
            end: {
                let (mut chunk_start, mut chunk) = (chunk_start, chunk);

                loop {
                    match self.graphemes.next_boundary(chunk, chunk_start) {
                        Ok(None) => unreachable!("`self.chunk` is `None` already"),
                        Ok(Some(boundary)) => break boundary,
                        Err(GraphemeIncomplete::NextChunk) => {
                            chunk_start += chunk.len();
                            chunk = self.slice.chunk_at_byte(chunk_start).0;
                        }
                        Err(GraphemeIncomplete::PreContext(index)) => {
                            let (chunk, chunk_start, ..) = self.slice.chunk_at_byte(index - 1);
                            self.graphemes.provide_context(chunk, chunk_start);
                        }
                        _ => unreachable!("Not returned by `next_boundary()`"),
                    }
                }
            },
        };
        let in_chunk = grapheme.end <= chunk_start + chunk.len();

        debug_assert!(self.graphemes.cur_cursor() == grapheme.end);
        self.goto_index();
        Some((
            grapheme.start..grapheme.end,
            self.grapheme_chunks(in_chunk, grapheme, chunk, chunk_start),
        ))
    }
}

/// Private.
impl<'rope> GraphemeCursor<'rope> {
    fn goto_index(&mut self) {
        type PrevOrNext<'rope> = fn(&mut ChunkCursor<'rope>) -> Option<(usize, &'rope str)>;

        fn goto_index<'rope>(
            (index, cursor, prev_or_next): (usize, &mut GraphemeCursor<'rope>, PrevOrNext<'rope>),
            range: fn(usize, usize) -> (Bound<usize>, Bound<usize>),
            none_assertions: impl FnOnce(),
        ) {
            cursor.chunk = loop {
                match prev_or_next(&mut cursor.chunks)
                    .map(|(index, chunk)| (index, index + chunk.len(), chunk))
                {
                    Some((start, end, chunk)) if range(start, end).contains(&index) => {
                        break Some((start, chunk));
                    }
                    Some(_) => {}
                    None => {
                        none_assertions();
                        break None;
                    }
                };
            }
        }

        let index = self.graphemes.cur_cursor();
        let len = self.slice.len_bytes();
        let goto_prev_index = |cursor: &mut GraphemeCursor, index| {
            goto_index(
                (index, cursor, ChunkCursor::prev),
                |start, end| (Excluded(start), Included(end)),
                || debug_assert!(index == 0),
            );
        };
        let goto_next_index = |cursor: &mut GraphemeCursor, index| {
            goto_index(
                (index, cursor, ChunkCursor::next),
                |start, end| (Included(start), Excluded(end)),
                || debug_assert!(index == len),
            )
        };

        match (self.chunk, self.is_chunks_after_chunk) {
            (Some((start, chunk)), true) => {
                if index < start {
                    self.is_chunks_after_chunk = false;
                    goto_prev_index(self, index);
                } else if index < start + chunk.len() {
                } else {
                    goto_next_index(self, index);
                }
            }
            (Some((start, chunk)), false) => {
                if index <= start {
                    goto_prev_index(self, index);
                } else if index <= start + chunk.len() {
                } else {
                    self.is_chunks_after_chunk = true;
                    goto_next_index(self, index);
                }
            }
            (None, true) => {
                self.is_chunks_after_chunk = false;
                goto_prev_index(self, index);
            }
            (None, false) => {
                self.is_chunks_after_chunk = true;
                goto_next_index(self, index);
            }
        }
    }

    fn grapheme_chunks(
        &self,
        in_chunk: bool,
        grapheme: Range<usize>,
        chunk: &'rope str,
        chunk_start: usize,
    ) -> impl Iterator<Item = (usize, &'rope str)> {
        enum GraphemeChunks<'rope> {
            One(Option<&'rope str>),
            Many(ChunkCursor<'rope>),
        }

        let mut chunks = if in_chunk {
            let start = grapheme.start - chunk_start;
            let end = grapheme.end - chunk_start;
            GraphemeChunks::One(Some(&chunk[start..end]))
        } else {
            let start = self.slice.byte_to_char(grapheme.start);
            let end = self.slice.byte_to_char(grapheme.end);
            GraphemeChunks::Many(ChunkCursor::new(self.slice.slice(start..end), 0))
        };

        std::iter::from_fn(move || match &mut chunks {
            GraphemeChunks::One(chunk) => chunk.take().map(|chunk| (0, chunk)),
            GraphemeChunks::Many(chunks) => chunks.next(),
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;
    use unicode_segmentation::UnicodeSegmentation;

    fn strings() -> Vec<String> {
        vec![
            "".to_string(),
            "🦠🦀🦠🦀".to_string(),
            "Hello, world!".repeat(200),
            "e".to_string() + &"\u{0300}".repeat(1000) + " e" + &"\u{0301}".repeat(1000),
        ]
    }

    #[test]
    fn is_boundary() {
        for string in strings() {
            let rope = Rope::from(string.as_str());

            for index in (0..=string.len()).filter(|index| string.is_char_boundary(*index)) {
                assert!(
                    GraphemeCursor::new(rope.slice(..), index).is_boundary()
                        == unicode_segmentation::GraphemeCursor::new(index, string.len(), true)
                            .is_boundary(&string, 0)
                            .unwrap()
                );
            }
        }
    }

    #[test]
    fn next_and_prev() {
        for string in strings() {
            let rope = Rope::from(string.as_str());
            let slice = rope.slice(..);

            // Start to end to start
            {
                let indices = string
                    .grapheme_indices(true)
                    .map(|(index, str)| (index, str.to_string()))
                    .collect::<Vec<_>>();

                let mut cursor = GraphemeCursor::new(rope.slice(..), 0);

                // Next
                let collected_indices = std::iter::from_fn(|| {
                    let (range, chunks) = cursor.next()?;
                    let grapheme = chunks.map(|(_, chunk)| chunk).collect::<String>();

                    assert!(cursor.is_boundary());
                    assert!(cursor.index() == range.end);
                    assert!(range.len() == grapheme.len());

                    Some((range.start, grapheme))
                })
                .collect::<Vec<_>>();
                let collected_string = collected_indices
                    .iter()
                    .map(|(_, string)| string.as_str())
                    .collect::<String>();

                assert!(collected_indices == indices);
                assert!(collected_string == string);

                // Prev
                let collected_indices = std::iter::from_fn(|| {
                    let (range, chunks) = cursor.prev()?;
                    let grapheme = chunks.map(|(_, chunk)| chunk).collect::<String>();

                    assert!(cursor.is_boundary());
                    assert!(cursor.index() == range.start);
                    assert!(range.len() == grapheme.len());

                    Some((range.start, grapheme))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>();
                let collected_string = collected_indices
                    .iter()
                    .map(|(_, string)| string.as_str())
                    .collect::<String>();

                assert!(collected_indices == indices);
                assert!(collected_string == string);
            }

            // Prev/Next at
            for index in (0..=string.len()).filter(|i| string.is_char_boundary(*i)) {
                let prev = GraphemeCursor::new(slice, index)
                    .prev()
                    .map(|(range, chunks)| {
                        (
                            range.start,
                            chunks.map(|(_, chunk)| chunk).collect::<String>(),
                        )
                    });
                let next = GraphemeCursor::new(slice, index)
                    .next()
                    .map(|(range, chunks)| {
                        (
                            range.start,
                            chunks.map(|(_, chunk)| chunk).collect::<String>(),
                        )
                    });
                let expected_prev = string[..index]
                    .grapheme_indices(true)
                    .next_back()
                    .map(|(i, str)| (i, str.to_string()));
                let expected_next = string[index..]
                    .grapheme_indices(true)
                    .next()
                    .map(|(i, str)| (index + i, str.to_string()));

                assert!(prev == expected_prev);
                assert!(next == expected_next);
            }
        }
    }
}
