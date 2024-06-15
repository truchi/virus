use crate::rope::GraphemeCursor;
use ropey::RopeSlice;
use std::ops::Range;

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

/// Cursor in a `RopeSlice`'s words.
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

    pub fn set_index(&mut self, index: usize) {
        self.graphemes.set_index(index);
    }

    pub fn prev(&mut self) -> Option<(Range<usize>, WordClass)> {
        use CharClass::*;

        let skip = |graphemes: &mut GraphemeCursor, initial: CharClass| loop {
            loop {
                match graphemes.prev().map(Self::map_to_char_class) {
                    Some((grapheme, class)) if class != initial => return Some((grapheme, class)),
                    Some(_) => {}
                    None => return None,
                }
            }
        };

        let (mut grapheme, class) = self.graphemes.prev().map(Self::map_to_char_class)?;

        // Uppercase with lowercase on the right (`HelloW|orld`)
        if class == Uppercase {
            let next = {
                self.graphemes.next();
                self.graphemes.next().map(Self::map_to_char_class)
            };
            self.set_index(grapheme.start);

            if let Some((_, Lowercase)) = next {
                return Some((grapheme, class.into()));
            }
        }

        grapheme.start = match (class, skip(&mut self.graphemes, class)) {
            // Lowercases then uppercase (`Hello|`)
            (Lowercase, Some((prev, Uppercase))) => prev.start,

            // Other graphemes can be repeated to form a word
            (_, Some((prev, _))) => {
                self.set_index(prev.end);
                prev.end
            }
            _ => 0,
        };

        debug_assert!(self.graphemes.index() == grapheme.start);
        Some((grapheme, class.into()))
    }

    pub fn next(&mut self) -> Option<(Range<usize>, WordClass)> {
        use CharClass::*;

        let skip = |graphemes: &mut GraphemeCursor, initial: CharClass| loop {
            let mut prev = None;

            loop {
                match graphemes.next().map(Self::map_to_char_class) {
                    Some((grapheme, class)) if class != initial => {
                        return (prev, Some((grapheme, class)))
                    }
                    Some((grapheme, _)) => prev = Some(grapheme),
                    None => return (prev, None),
                }
            }
        };

        let len = self.graphemes.slice().len_bytes();
        let (mut grapheme, class) = self.graphemes.next().map(Self::map_to_char_class)?;
        grapheme.end = match (class, skip(&mut self.graphemes, class)) {
            // Uppercase then lowercase (`|Hello`)
            (Uppercase, (None, Some((_, Lowercase)))) => {
                match skip(&mut self.graphemes, Lowercase).1 {
                    Some((next, _)) => {
                        self.set_index(next.start);
                        next.start
                    }
                    _ => len,
                }
            }

            // Uppercases then lowercase (`|HELLOWorld`)
            (Uppercase, (Some(prev), Some((_, Lowercase)))) => {
                self.set_index(prev.start);
                prev.start
            }

            // Other graphemes can be repeated to form a word
            (_, (_, Some((next, _)))) => {
                self.set_index(next.start);
                next.start
            }
            _ => len,
        };

        debug_assert!(self.graphemes.index() == grapheme.end);
        Some((grapheme, class.into()))
    }
}

/// Private.
impl<'rope> WordCursor<'rope> {
    fn map_to_char_class<'a>(
        (grapheme, chunks): (Range<usize>, impl Iterator<Item = (usize, &'a str)>),
    ) -> (Range<usize>, CharClass) {
        (
            grapheme,
            CharClass::from(
                chunks
                    .flat_map(|(_, chunk)| chunk.chars())
                    .next()
                    .expect("Empty grapheme"),
            ),
        )
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
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
            let slice = rope.slice(..);

            // Before start
            assert!(WordCursor::new(slice, 0).prev().is_none());

            // After end
            assert!(WordCursor::new(slice, str.len()).next().is_none());

            // Start to end
            let mut collected = vec![];
            let mut classes = WordCursor::new(slice, 0);
            while let Some((range, class)) = classes.next() {
                assert!(classes.graphemes.index() == range.end);
                collected.push((&str[range], class));
            }

            assert!(collected == words);

            // End to start
            let mut collected = vec![];
            let mut classes = WordCursor::new(slice, str.len());
            while let Some((range, class)) = classes.prev() {
                assert!(classes.graphemes.index() == range.start);
                collected.push((&str[range], class));
            }

            assert!(collected == words.iter().copied().rev().collect::<Vec<_>>());

            // Next/Prev at
            let mut offset = 0;
            for &(word, expected) in &words {
                for (i, char) in word.char_indices() {
                    // Next at
                    let (range, class) = WordCursor::new(slice, offset + i).next().unwrap();

                    assert!(&str[range] == &word[i..]);
                    assert!(class == expected);

                    // Prev at
                    let (range, class) = WordCursor::new(slice, offset + i + char.len_utf8())
                        .prev()
                        .unwrap();

                    assert!(&str[range] == &word[..i + char.len_utf8()]);
                    assert!(class == expected);
                }

                offset += word.len();
            }
        }
    }
}
