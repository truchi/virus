use ropey::{iter::Chars, Rope, RopeSlice};
use std::{
    iter::{Map, Peekable},
    ops::Range,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                     GraphemeAndCharCursor                                      //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct ChunkCursor<'rope> {}

impl<'rope> ChunkCursor<'rope> {
    pub fn new(rope: &'rope Rope, index: usize) -> Self {}
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           CharCursor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct CharCursor<'rope> {
    chars: Chars<'rope>,
    index: usize,
}

impl<'rope> CharCursor<'rope> {
    pub fn new(rope: &'rope Rope, index: usize) -> Self {
        Self {
            chars: rope.chars_at(rope.byte_to_char(index)),
            index,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn prev(&mut self) -> Option<(usize, char)> {
        let char = self.chars.prev()?;

        self.index -= char.len_utf8();
        Some((self.index, char))
    }

    pub fn next(&mut self) -> Option<(usize, char)> {
        let char = self.chars.next()?;
        let index = self.index;

        self.index += char.len_utf8();
        Some((index, char))
    }

    /// Advances while `f` returns `true`,
    /// returns the last skipped item and the next item at the new position.
    pub fn next_while<F: FnMut((usize, char)) -> bool>(
        &mut self,
        mut f: F,
    ) -> (Option<(usize, char)>, Option<(usize, char)>) {
        let mut last = None;

        loop {
            let next = match self.next() {
                Some(current) if f(current) => {
                    last = Some(current);
                    continue;
                }
                Some(next) => {
                    self.prev();
                    Some(next)
                }
                None => None,
            };

            return (last, next);
        }
    }
}

// // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
// //                                          CharIndices                                           //
// // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// /// Char indices iterator, from left to right.
// pub struct CharIndicesLtr<'rope> {
//     chars: ropey::iter::Chars<'rope>,
//     index: usize,
// }

// impl<'rope> CharIndicesLtr<'rope> {
//     pub fn new(slice: RopeSlice<'rope>) -> Self {
//         Self {
//             chars: slice.chars(),
//             index: 0,
//         }
//     }
// }

// impl<'rope> Iterator for CharIndicesLtr<'rope> {
//     type Item = (usize, char);

//     fn next(&mut self) -> Option<Self::Item> {
//         let char = self.chars.next()?;
//         let index = self.index;

//         self.index += char.len_utf8();
//         Some((index, char))
//     }
// }

// // ────────────────────────────────────────────────────────────────────────────────────────────── //

// /// Char indices iterator, from right to left.
// pub struct CharIndicesRtl<'rope> {
//     chars: ropey::iter::Chars<'rope>,
//     index: usize,
// }

// impl<'rope> CharIndicesRtl<'rope> {
//     pub fn new(slice: RopeSlice<'rope>) -> Self {
//         Self {
//             chars: slice.chars_at(slice.len_chars()).reversed(),
//             index: slice.len_bytes(),
//         }
//     }
// }

// impl<'rope> Iterator for CharIndicesRtl<'rope> {
//     type Item = (usize, char);

//     fn next(&mut self) -> Option<Self::Item> {
//         let char = self.chars.next()?;

//         self.index -= char.len_utf8();
//         Some((self.index, char))
//     }
// }

// // ────────────────────────────────────────────────────────────────────────────────────────────── //

// #[cfg(test)]
// mod char_indices_tests {
//     use super::*;

//     #[test]
//     fn test() {
//         let str = "Hello, world! 🦠❤🦀";
//         let rope = Rope::from(str);

//         assert!(
//             CharIndicesLtr::new(rope.slice(..)).collect::<Vec<_>>()
//                 == str.char_indices().collect::<Vec<_>>()
//         );
//         assert!(
//             CharIndicesRtl::new(rope.slice(..)).collect::<Vec<_>>()
//                 == str.char_indices().rev().collect::<Vec<_>>()
//         );
//     }
// }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Words                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq)]
enum CharClass {
    Whitespace,
    Punctuation(char),
    Numeric,
    Lowercase,
    Uppercase,
}

impl From<char> for CharClass {
    fn from(char: char) -> Self {
        if char.is_whitespace() {
            Self::Whitespace
        } else if char.is_ascii_punctuation() {
            Self::Punctuation(char)
        } else if char.is_numeric() {
            Self::Numeric
        } else if char.is_uppercase() {
            Self::Uppercase
        } else {
            Self::Lowercase
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
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct WordClasses<'rope> {
    chars: CharCursor<'rope>,
    len: usize,
}

impl<'rope> WordClasses<'rope> {
    pub fn new(rope: &'rope Rope, index: usize) -> Self {
        Self {
            chars: CharCursor::new(rope, index),
            len: rope.len_bytes(),
        }
    }
}

impl<'rope> Iterator for WordClasses<'rope> {
    type Item = (Range<usize>, WordClass);

    fn next(&mut self) -> Option<Self::Item> {
        use CharClass::*;

        let (start, class) = {
            let (start, char) = self.chars.next()?;
            let class = CharClass::from(char);
            (start, class)
        };

        let (last, next) = self.chars.next_while(|(_, char)| class == char.into());
        let (last, next) = (
            last.map(|(index, char)| (index, CharClass::from(char))),
            next.map(|(index, char)| (index, CharClass::from(char))),
        );

        let end = match (class, last, next) {
            (Uppercase, None, Some((_, Lowercase))) => {
                let (_, next) = self.chars.next_while(|(_, char)| Lowercase == char.into());
                next.map(|(end, _)| end).unwrap_or(self.len)
            }
            (Uppercase, Some((end, _)), Some((_, Lowercase))) => {
                self.chars.prev();
                end
            }
            _ => next.map(|(end, _)| end).unwrap_or(self.len),
        };

        return Some((start..end, class.into()));
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

// pub struct WordClassesLtr<'rope> {
//     chars: Peekable<Map<CharIndicesLtr<'rope>, fn((usize, char)) -> (usize, CharClass)>>,
//     current: Option<(usize, CharClass)>,
//     len: usize,
// }

// impl<'rope> WordClassesLtr<'rope> {
//     pub fn new(rope: &'rope Rope, index: usize) -> Self {
//         let slice = rope.slice(rope.byte_to_char(index)..);
//         let mut chars = CharIndicesLtr::new(slice)
//             .map((|(index, char)| (index, CharClass::from(char))) as _)
//             .peekable();
//         let current = chars.next();

//         Self {
//             chars,
//             current,
//             len: rope.len_bytes(),
//         }
//     }
// }

// impl<'rope> Iterator for WordClassesLtr<'rope> {
//     type Item = (Range<usize>, WordClass);

//     // Devrait marcher, pas testé...
//     // Faire avec in CharIndicesCursor serai plus pratique, non?
//     fn next(&mut self) -> Option<Self::Item> {
//         let (start, mut initial) = self.current?;

//         // Cursor before an uppercase is a special case
//         if initial == CharClass::Uppercase {
//             match self.chars.peek() {
//                 // Followed by uppercases then lowercase,
//                 // return the index before the last uppercase
//                 Some((_, CharClass::Uppercase)) => {
//                     let last =
//                         std::iter::from_fn(|| self.chars.next_if(|(_, class)| *class == initial))
//                             .last()
//                             .expect("Just peeked it");

//                     if matches!(self.chars.peek(), Some((_, CharClass::Lowercase))) {
//                         self.current = Some(last);

//                         return Some((start..last.0, initial.into()));
//                     }
//                 }
//                 // Followed by a lowercase, pretend it was lowercase
//                 Some((_, CharClass::Lowercase)) => initial = CharClass::Lowercase,
//                 _ => {}
//             }
//         }

//         // Remove chars until another class
//         while let Some(_) = self.chars.next_if(|(_, class)| *class == initial) {}

//         // Prepare next iteration
//         self.current = self.chars.next();

//         // Return that index, or end
//         let end = if let Some((end, _)) = self.current {
//             end
//         } else {
//             self.len
//         };

//         Some((start..end, initial.into()))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use WordClass::*;

    #[test]
    fn test() {
        // Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
        // hello {((((world))))}, salut
        let str = "Hello, world!";
        let str = "Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123";
        let str = "ala🦠lala";

        let rope = Rope::from(str);
        let it = WordClasses::new(&rope, 0);

        dbg!(str);

        for (range, class) in it {
            dbg!((&str[range], class));
        }

        // dbg!(WordClassesLtr::new(&rope, 0).collect::<Vec<_>>());

        // assert!(
        //     WordClassesLtr::new(&rope, 0).collect::<Vec<_>>()
        //         == [
        //             (0..5, Word),
        //             (5..6, Punctuation(',')),
        //             (6..7, Whitespace),
        //             (7..12, Word),
        //             (12..13, Punctuation('!')),
        //         ]
        // );
    }
}
