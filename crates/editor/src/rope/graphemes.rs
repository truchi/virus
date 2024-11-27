use ropey::{iter::Chunks, RopeSlice};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Grapheme                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub enum Grapheme<'rope> {
    Str(&'rope str),
    String(String),
}

impl<'rope> Grapheme<'rope> {
    pub fn as_str(&self) -> &str {
        match self {
            Grapheme::Str(grapheme) => grapheme,
            Grapheme::String(grapheme) => grapheme,
        }
    }

    fn default() -> Self {
        Self::Str("")
    }

    fn append(&mut self, str: &'rope str) {
        match self {
            Self::Str(grapheme) if grapheme.is_empty() => {
                *self = Self::Str(str);
            }
            Self::Str(grapheme) => {
                *self = Self::String(format!("{grapheme}{str}"));
            }
            Self::String(grapheme) => {
                grapheme.insert_str(grapheme.len(), str);
            }
        }
    }

    fn prepend(&mut self, str: &'rope str) {
        match self {
            Self::Str(grapheme) if grapheme.is_empty() => {
                *self = Self::Str(str);
            }
            Self::Str(grapheme) => {
                *self = Self::String(format!("{str}{grapheme}"));
            }
            Self::String(grapheme) => {
                grapheme.insert_str(0, str);
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        GraphemesForward                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct GraphemesForward<'rope> {
    chunks: Chunks<'rope>,
    chunk: Option<&'rope str>,
    chunk_start: usize,
    graphemes: GraphemeCursor,
}

impl<'rope> GraphemesForward<'rope> {
    pub fn new(slice: RopeSlice<'rope>) -> Self {
        let (mut chunks, chunk_start, ..) = slice.chunks_at_byte(0);
        let chunk = chunks.next();

        Self {
            chunks,
            chunk,
            chunk_start,
            graphemes: GraphemeCursor::new(0, slice.len_bytes(), true),
        }
    }
}

impl<'rope> Iterator for GraphemesForward<'rope> {
    type Item = Grapheme<'rope>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut grapheme = Grapheme::default();

        loop {
            let chunk = self.chunk?;
            let index = self.graphemes.cur_cursor();

            match self.graphemes.next_boundary(chunk, self.chunk_start) {
                Ok(None) => {
                    self.chunk = self.chunks.next();
                    debug_assert!(self.chunk.is_none());
                    return None;
                }
                Ok(Some(next)) => {
                    grapheme.append(&chunk[index - self.chunk_start..next - self.chunk_start]);
                    return Some(grapheme);
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    grapheme.append(&chunk[index - self.chunk_start..]);
                    self.chunk = self.chunks.next();
                    self.chunk_start += chunk.len();
                    debug_assert!(self.chunk.is_some());
                }
                _ => unreachable!(),
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       GraphemesBackward                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct GraphemesBackward<'rope> {
    chunks: Chunks<'rope>,
    chunk: Option<&'rope str>,
    chunk_start: usize,
    graphemes: GraphemeCursor,
}

impl<'rope> GraphemesBackward<'rope> {
    pub fn new(slice: RopeSlice<'rope>) -> Self {
        let (mut chunks, ..) = slice.chunks_at_byte(slice.len_bytes());
        let chunk = chunks.prev();
        let chunk_start = slice.len_bytes() - chunk.unwrap_or_default().len();

        Self {
            chunks,
            chunk,
            chunk_start,
            graphemes: GraphemeCursor::new(slice.len_bytes(), slice.len_bytes(), true),
        }
    }
}

impl<'rope> Iterator for GraphemesBackward<'rope> {
    type Item = Grapheme<'rope>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut grapheme = Grapheme::default();

        loop {
            let chunk = self.chunk?;
            let index = self.graphemes.cur_cursor();

            match self.graphemes.prev_boundary(chunk, self.chunk_start) {
                Ok(None) => {
                    self.chunk = self.chunks.prev();
                    debug_assert!(self.chunk.is_none());
                    return None;
                }
                Ok(Some(prev)) => {
                    grapheme.prepend(&chunk[prev - self.chunk_start..index - self.chunk_start]);
                    return Some(grapheme);
                }
                Err(GraphemeIncomplete::PrevChunk) => {
                    grapheme.prepend(&chunk[..index - self.chunk_start]);
                    self.chunk = self.chunks.prev();
                    self.chunk_start -= self.chunk.unwrap_or_default().len();
                    debug_assert!(self.chunk.is_some());
                }
                _ => unreachable!(),
            }
        }
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

    const COMBINING: &str = "\u{0300}\u{0301}\u{0302}\u{0303}\u{0304}\u{0305}\u{0306}\u{0307}";

    #[test]
    fn forward() {
        let str = (String::from('e') + &COMBINING.repeat(200) + " abc").repeat(2);
        let rope = Rope::from(str.as_str());

        assert_eq!(
            GraphemesForward::new(rope.slice(..))
                .map(|grapheme| grapheme.as_str().to_owned())
                .collect::<Vec<_>>(),
            str.graphemes(true).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn backward() {
        let str = (String::from('e') + &COMBINING.repeat(2048) + " abc").repeat(2);
        let rope = Rope::from(str.as_str());

        assert_eq!(
            GraphemesBackward::new(rope.slice(..))
                .map(|grapheme| grapheme.as_str().to_owned())
                .collect::<Vec<_>>(),
            str.graphemes(true).rev().collect::<Vec<_>>(),
        );
    }
}
