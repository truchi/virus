use ropey::{iter::Chunks, RopeSlice};
use std::ops::{
    Bound::{self, *},
    Range, RangeBounds,
};
use unicode_segmentation::GraphemeIncomplete;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          ChunkCursor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         GraphemeCursor                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

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

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[cfg(test)]
mod grapheme_cursor_tests {
    use super::*;
    use ropey::Rope;
    use unicode_segmentation::UnicodeSegmentation;

    fn strings() -> Vec<String> {
        vec![
            "".to_string(),
            "Hello, world!".repeat(200),
            "e".to_string() + &"\u{0300}".repeat(2000) + " e" + &"\u{0301}".repeat(2000),
        ]
    }

    #[test]
    fn is_boundary() {
        for string in strings() {
            let rope = Rope::from(string.as_str());

            for (index, _) in string.char_indices().chain([(string.len(), ' ')]) {
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
        fn collect_indices_next(cursor: &mut GraphemeCursor) -> Vec<(usize, String)> {
            std::iter::from_fn(|| {
                let (range, chunks) = cursor.next()?;
                let grapheme = chunks.map(|(_, chunk)| chunk).collect::<String>();

                assert!(cursor.is_boundary());
                assert!(cursor.index() == range.end);
                assert!(range.len() == grapheme.len());

                Some((range.start, grapheme))
            })
            .collect()
        }
        fn collect_indices_prev(cursor: &mut GraphemeCursor) -> Vec<(usize, String)> {
            std::iter::from_fn(|| {
                let (range, chunks) = cursor.prev()?;
                let grapheme = chunks.map(|(_, chunk)| chunk).collect::<String>();

                assert!(cursor.is_boundary());
                assert!(cursor.index() == range.start);
                assert!(range.len() == grapheme.len());

                Some((range.start, grapheme))
            })
            .collect()
        }
        fn collect_string(collected_indices: &[(usize, String)]) -> String {
            collected_indices
                .iter()
                .map(|(_, string)| string.as_str())
                .collect()
        }

        for string in strings() {
            let indices = string
                .grapheme_indices(true)
                .map(|(index, str)| (index, str.to_string()))
                .collect::<Vec<_>>();

            let rope = Rope::from(string.as_str());
            let mut cursor = GraphemeCursor::new(rope.slice(..), 0);

            // Next...
            {
                let collected_indices = collect_indices_next(&mut cursor);
                let collected_string = collect_string(&collected_indices);

                assert!(collected_string == string);
                assert!(collected_indices == indices);
            }

            // ...Prev
            {
                let collected_indices = collect_indices_prev(&mut cursor)
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();
                let collected_string = collect_string(&collected_indices);

                assert!(collected_string == string);
                assert!(collected_indices == indices);
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           WordCursor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq)]
enum CharClass {
    Whitespace,
    Punctuation(char),
    Numeric,
    Lowercase,
    Uppercase,
    Unknown,
}

impl From<char> for CharClass {
    fn from(char: char) -> Self {
        if char.is_whitespace() {
            Self::Whitespace
        } else if char.is_ascii_punctuation() {
            Self::Punctuation(char)
        } else if char.is_numeric() {
            Self::Numeric
        } else if char.is_lowercase() {
            Self::Lowercase
        } else if char.is_uppercase() {
            Self::Uppercase
        } else {
            Self::Unknown
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum WordClass {
    Whitespace,
    Punctuation(char),
    Word,
}

impl From<CharClass> for WordClass {
    fn from(class: CharClass) -> Self {
        match class {
            CharClass::Whitespace => Self::Whitespace,
            CharClass::Punctuation(char) => Self::Punctuation(char),
            CharClass::Numeric => Self::Word,
            CharClass::Lowercase => Self::Word,
            CharClass::Uppercase => Self::Word,
            CharClass::Unknown => Self::Word,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct WordCursor<'rope> {
    graphemes: GraphemeCursor<'rope>,
}

impl<'rope> WordCursor<'rope> {
    pub fn new(slice: RopeSlice<'rope>, index: usize) -> Self {
        Self {
            graphemes: GraphemeCursor::new(slice, index),
        }
    }

    pub fn index(&self) -> usize {
        self.graphemes.index()
    }

    pub fn prev(&mut self) -> Option<(Range<usize>, WordClass)> {
        use CharClass::*;

        let prev = |graphemes: &mut GraphemeCursor| {
            graphemes.prev().map(|(grapheme, chunks)| {
                (
                    grapheme,
                    CharClass::from(
                        chunks
                            .flat_map(|(_, chunk)| chunk.chars())
                            .next()
                            .expect("Empty grapheme"),
                    ),
                )
            })
        };
        let skip = |graphemes: &mut GraphemeCursor, initial: CharClass| loop {
            loop {
                match prev(graphemes) {
                    Some((grapheme, class)) if class != initial => return Some((grapheme, class)),
                    Some(_) => {}
                    None => return None,
                }
            }
        };

        let (mut grapheme, class) = prev(&mut self.graphemes)?;
        grapheme.start = match (class, skip(&mut self.graphemes, class)) {
            // Lowercases then uppercase (`Hello`)
            (Lowercase, Some((prev, Uppercase))) => prev.start,

            // Lowercases (`hello`)
            (Lowercase, Some((prev, _))) => {
                self.graphemes.set_index(prev.end);
                prev.end
            }
            (Lowercase, _) => 0,

            // Other graphemes can be repeated to form a word
            (_, Some((prev, _))) => {
                self.graphemes.set_index(prev.end);
                prev.end
            }
            _ => 0,
        };

        Some((grapheme, class.into()))
    }

    pub fn next(&mut self) -> Option<(Range<usize>, WordClass)> {
        use CharClass::*;

        let next = |graphemes: &mut GraphemeCursor| {
            graphemes.next().map(|(grapheme, chunks)| {
                (
                    grapheme,
                    CharClass::from(
                        chunks
                            .flat_map(|(_, chunk)| chunk.chars())
                            .next()
                            .expect("Empty grapheme"),
                    ),
                )
            })
        };
        let skip = |graphemes: &mut GraphemeCursor, initial: CharClass| loop {
            let mut prev = None;

            loop {
                match next(graphemes) {
                    Some((grapheme, class)) if class != initial => {
                        return (prev, Some((grapheme, class)))
                    }
                    Some((grapheme, _)) => prev = Some(grapheme),
                    None => return (prev, None),
                }
            }
        };

        let len = self.graphemes.slice.len_bytes();
        let (mut grapheme, class) = next(&mut self.graphemes)?;
        grapheme.end = match (class, skip(&mut self.graphemes, class)) {
            // Uppercase then lowercase (`Hello`)
            (Uppercase, (None, Some((_, Lowercase)))) => {
                match skip(&mut self.graphemes, Lowercase).1 {
                    Some((next, _)) => {
                        self.graphemes.set_index(next.start);
                        next.start
                    }
                    _ => len,
                }
            }

            // Uppercases then lowercase (`HELLOWorld`)
            (Uppercase, (Some(prev), Some((_, Lowercase)))) => {
                self.graphemes.set_index(prev.start);
                prev.start
            }

            // Uppercases (`HELLO`)
            (Uppercase, (_, Some((next, _)))) => {
                self.graphemes.set_index(next.start);
                next.start
            }
            (Uppercase, _) => len,

            // Other graphemes can be repeated to form a word
            (_, (_, Some((next, _)))) => {
                self.graphemes.set_index(next.start);
                next.start
            }
            _ => len,
        };

        Some((grapheme, class.into()))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[cfg(test)]
mod word_cursor_tests {
    use super::*;
    use ropey::Rope;
    use WordClass::*;

    #[test]
    fn prev_and_next() {
        let data = [
            (
                "0HTTPProxyOfTheDeath23MORE123\r\n",
                vec![
                    ("0", Word),
                    ("HTTP", Word),
                    ("Proxy", Word),
                    ("Of", Word),
                    ("The", Word),
                    ("Death", Word),
                    ("23", Word),
                    ("MORE", Word),
                    ("123", Word),
                    ("\r\n", Whitespace),
                ],
            ),
            (
                "hello {((((world()))))}, 'salut'",
                vec![
                    ("hello", Word),
                    (" ", Whitespace),
                    ("{", Punctuation('{')),
                    ("((((", Punctuation('(')),
                    ("world", Word),
                    ("(", Punctuation('(')),
                    (")))))", Punctuation(')')),
                    ("}", Punctuation('}')),
                    (",", Punctuation(',')),
                    (" ", Whitespace),
                    ("'", Punctuation('\'')),
                    ("salut", Word),
                    ("'", Punctuation('\'')),
                ],
            ),
            (
                "🦠 🦠virus🦠🦀rust🦠 🦠🦀",
                vec![
                    ("🦠", Word),
                    (" ", Whitespace),
                    ("🦠", Word),
                    ("virus", Word),
                    ("🦠🦀", Word),
                    ("rust", Word),
                    ("🦠", Word),
                    (" ", Whitespace),
                    ("🦠🦀", Word),
                ],
            ),
            (
                "Hello!   -- the-world _ hello- __world the--w",
                vec![
                    ("Hello", Word),
                    ("!", Punctuation('!')),
                    ("   ", Whitespace),
                    ("--", Punctuation('-')),
                    (" ", Whitespace),
                    ("the", Word),
                    ("-", Punctuation('-')),
                    ("world", Word),
                    (" ", Whitespace),
                    ("_", Punctuation('_')),
                    (" ", Whitespace),
                    ("hello", Word),
                    ("-", Punctuation('-')),
                    (" ", Whitespace),
                    ("__", Punctuation('_')),
                    ("world", Word),
                    (" ", Whitespace),
                    ("the", Word),
                    ("--", Punctuation('-')),
                    ("w", Word),
                ],
            ),
        ];

        for (str, words) in data {
            let rope = Rope::from(str);

            // With `index == 0`
            {
                // Prev
                let mut collected = vec![];
                let mut classes = WordCursor::new(rope.slice(..), str.len());
                while let Some((range, class)) = classes.prev() {
                    assert!(classes.graphemes.index() == range.start);
                    collected.push((&str[range], class));
                }

                assert!(collected == words.iter().copied().rev().collect::<Vec<_>>());

                // Next
                let mut collected = vec![];
                let mut classes = WordCursor::new(rope.slice(..), 0);
                while let Some((range, class)) = classes.next() {
                    assert!(classes.graphemes.index() == range.end);
                    collected.push((&str[range], class));
                }

                assert!(collected == words);
            }

            // With index (simplified with flatmap by chars)
            {
                // Prev
                let mut chars = words
                    .iter()
                    .flat_map(|(str, class)| str.chars().map(|char| (char, *class)))
                    .rev()
                    .collect::<Vec<_>>();

                for (i, char) in str.char_indices().rev() {
                    let mut collected = vec![];
                    let mut classes = WordCursor::new(rope.slice(..), i + char.len_utf8());
                    while let Some((range, class)) = classes.prev() {
                        for char in str[range].chars().rev() {
                            collected.push((char, class));
                        }
                    }

                    assert!(collected == chars);
                    chars.remove(0);
                }

                assert!(WordCursor::new(rope.slice(..), 0).prev().is_none());

                // Next
                let mut chars = words
                    .iter()
                    .flat_map(|(str, class)| str.chars().map(|char| (char, *class)))
                    .collect::<Vec<_>>();

                for (i, _) in str.char_indices() {
                    let mut collected_next = vec![];
                    let mut classes = WordCursor::new(rope.slice(..), i);
                    while let Some((range, class)) = classes.next() {
                        for char in str[range].chars() {
                            collected_next.push((char, class));
                        }
                    }

                    assert!(collected_next == chars);
                    chars.remove(0);
                }

                assert!(WordCursor::new(rope.slice(..), str.len()).next().is_none());
            }
        }
    }
}
