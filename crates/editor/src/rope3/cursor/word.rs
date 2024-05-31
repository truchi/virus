use crate::rope3::GraphemeCursor;
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

            // FIXME if uppercase, we have to check if there is a lowercase on the right...

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

        let len = self.graphemes.slice().len_bytes();
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
