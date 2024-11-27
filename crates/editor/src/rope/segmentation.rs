#![allow(unused)]

use crate::rope::Cursor;
use bitflags::Flags;
use ropey::Rope;
use std::cell::Cell;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};
use unicode_width::UnicodeWidthChar;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Grapheme                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GraphemeCategory {
    Break,
    Space,
    Punctuation,
    Separator,
    Upper,
    Lower,
    Numeric,
    Symbol,
}

impl From<&str> for GraphemeCategory {
    fn from(grapheme: &str) -> Self {
        debug_assert!(!grapheme.is_empty());

        match grapheme {
            "\r" | "\n" | "\r\n" => Self::Break,
            _ if grapheme.chars().all(|char| char.is_ascii_whitespace()) => Self::Space,

            "_" | "-" => Self::Separator,
            _ if grapheme.chars().all(|char| char.is_ascii_punctuation()) => Self::Punctuation,

            _ if grapheme.chars().all(|char| char.is_ascii_uppercase()) => Self::Upper,
            _ if grapheme.chars().all(|char| char.is_ascii_lowercase()) => Self::Lower,
            _ if grapheme.chars().all(|char| char.is_ascii_digit()) => Self::Numeric,

            _ => {
                let Some(char) = grapheme.chars().next() else {
                    return Self::Space;
                };

                match char.general_category_group() {
                    GeneralCategoryGroup::Letter => char
                        .is_uppercase()
                        .then_some(Self::Upper)
                        .unwrap_or(Self::Lower),
                    GeneralCategoryGroup::Mark => Self::Space,
                    GeneralCategoryGroup::Number => Self::Numeric,
                    GeneralCategoryGroup::Punctuation => Self::Punctuation,
                    GeneralCategoryGroup::Symbol => Self::Symbol,
                    GeneralCategoryGroup::Separator => Self::Space,
                    GeneralCategoryGroup::Other => Self::Space,
                }
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Boundaries                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

bitflags::bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Boundaries: u32 {
        const LINE_START        = 0b_0000_0000_0000_0001;
        const LINE_END          = 0b_0000_0000_0000_0010;
        const LINE_FIRST        = 0b_0000_0000_0000_0100;
        const LINE_LAST         = 0b_0000_0000_0000_1000;
        const PUNCTUATION_START = 0b_0000_0000_0001_0000;
        const PUNCTUATION_END   = 0b_0000_0000_0010_0000;
        const SHORT_WORD_START  = 0b_0000_0000_0100_0000;
        const SHORT_WORD_END    = 0b_0000_0000_1000_0000;
        const LONG_WORD_START   = 0b_0000_0001_0000_0000;
        const LONG_WORD_END     = 0b_0000_0010_0000_0000;
    }
}

mod bundaries {
    use super::*;
    use GraphemeCategory::*;

    #[rustfmt::skip]
    macro_rules! whitespace { () => { None | Some(Break | Space) } }

    #[rustfmt::skip]
    macro_rules! punctuation { () => { Some(Punctuation | Separator) } }

    #[rustfmt::skip]
    macro_rules! non_alphanum { () => { whitespace!() | punctuation!() } }

    #[rustfmt::skip]
    macro_rules! alphanum { () => { Some(Upper | Lower | Numeric | Symbol) } }

    #[rustfmt::skip]
    macro_rules! boundaries {
        ($boundaries:ident, $bool:expr $(,)?) => {
            if $bool { Boundaries::$boundaries } else { Boundaries::empty() }
        };
    }

    fn graphemes(
        index: usize,
        line: &str,
    ) -> (
        impl FnMut() -> Option<GraphemeCategory> + '_,
        impl FnMut() -> Option<GraphemeCategory> + '_,
    ) {
        let (left, right) = line.split_at(index);
        let (mut left, mut right) = (left.graphemes(true).rev(), right.graphemes(true));

        (
            move || left.next().map(GraphemeCategory::from),
            move || right.next().map(GraphemeCategory::from),
        )
    }

    fn is_word(mut dir: impl FnMut() -> Option<GraphemeCategory>) -> bool {
        loop {
            match dir() {
                Some(Separator) => continue,
                alphanum!() => break true,
                non_alphanum!() => break false,
            }
        }
    }

    // NOTE
    // In the following functions `line` must be one line only (optionally ending with a line break)
    // and index must not split `\r\n`.
    impl Boundaries {
        pub(super) fn is_line_start(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            index == 0
        }

        pub(super) fn is_line_end(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let line_break_len = if line.ends_with("\r\n") {
                2
            } else if line.ends_with(['\r', '\n']) {
                1
            } else {
                0
            };

            index == line.len() - line_break_len
        }

        pub(super) fn is_line_first(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (before, after) = line.split_at(index);

            before.chars().rev().all(|char| char.is_whitespace())
                && after.starts_with(|char: char| !char.is_whitespace())
        }

        pub(super) fn is_line_last(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (before, after) = line.split_at(index);

            before.ends_with(|char: char| !char.is_whitespace())
                && after.chars().all(|char| char.is_whitespace())
        }

        pub(super) fn is_punctuation_start(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (mut left, mut right) = graphemes(index, line);

            match right() {
                Some(Punctuation) => match left() {
                    whitespace!() | alphanum!() => true,
                    Some(Punctuation) => false,
                    Some(Separator) => is_word(left),
                },
                Some(Separator) => matches!(left(), whitespace!()),
                whitespace!() | alphanum!() => false,
            }
        }

        pub(super) fn is_punctuation_end(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (mut left, mut right) = graphemes(index, line);

            match left() {
                Some(Punctuation) => match right() {
                    whitespace!() | alphanum!() => true,
                    Some(Punctuation) => false,
                    Some(Separator) => is_word(right),
                },
                Some(Separator) => matches!(right(), whitespace!()),
                whitespace!() | alphanum!() => false,
            }
        }

        pub(super) fn is_short_word_start(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (mut left, mut right) = graphemes(index, line);

            match left() {
                Some(Upper) => match right() {
                    Some(Upper) => matches!(right(), Some(Lower)),
                    non_alphanum!() | Some(Lower) => false,
                    Some(Numeric | Symbol) => true,
                },
                Some(Lower) => matches!(right(), Some(Upper | Numeric | Symbol)),
                Some(Numeric) => matches!(right(), Some(Upper | Lower | Symbol)),
                Some(Symbol) => matches!(right(), Some(Upper | Lower | Numeric)),
                non_alphanum!() => matches!(right(), alphanum!()),
            }
        }

        pub(super) fn is_short_word_end(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (mut left, mut right) = graphemes(index, line);

            match right() {
                Some(Upper) => match left() {
                    Some(Upper) => matches!(right(), Some(Lower)),
                    non_alphanum!() => false,
                    Some(Lower | Numeric | Symbol) => true,
                },
                Some(Lower) => match left() {
                    non_alphanum!() | Some(Upper | Lower) => false,
                    Some(Numeric | Symbol) => true,
                },
                Some(Numeric) => matches!(left(), Some(Upper | Lower | Symbol)),
                Some(Symbol) => matches!(left(), Some(Upper | Lower | Numeric)),
                non_alphanum!() => matches!(left(), alphanum!()),
            }
        }

        pub(super) fn is_long_word_start(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (mut left, mut right) = graphemes(index, line);

            match left() {
                alphanum!() | Some(Separator) => false,
                whitespace!() | Some(Punctuation) => is_word(right),
            }
        }

        pub(super) fn is_long_word_end(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (mut left, mut right) = graphemes(index, line);

            match right() {
                alphanum!() | Some(Separator) => false,
                whitespace!() | Some(Punctuation) => is_word(left),
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Segmentation                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct Segmentation {
    rope: Rope,
    index: usize,
    line: usize,
    column: usize,
    current_line: String,
    graphemes: GraphemeCursor,
}

impl Segmentation {
    pub fn new(rope: Rope, index: usize, line: usize, column: usize) -> Self {
        let current_line = rope.line(line).to_string();
        let graphemes = GraphemeCursor::new(column, current_line.len(), true);

        Self {
            rope,
            index,
            line,
            column,
            current_line,
            graphemes,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn is_boundaries(&self, boundaries: Boundaries) -> bool {
        // Could be cached...
        boundaries.iter().any(|boundaries| match () {
            _ if boundaries == Boundaries::LINE_START => {
                Boundaries::is_line_start(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::LINE_END => {
                Boundaries::is_line_end(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::LINE_FIRST => {
                Boundaries::is_line_first(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::LINE_LAST => {
                Boundaries::is_line_last(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::PUNCTUATION_START => {
                Boundaries::is_punctuation_start(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::PUNCTUATION_END => {
                Boundaries::is_punctuation_end(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::SHORT_WORD_START => {
                Boundaries::is_short_word_start(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::SHORT_WORD_END => {
                Boundaries::is_short_word_end(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::LONG_WORD_START => {
                Boundaries::is_long_word_start(self.column, &self.current_line)
            }
            _ if boundaries == Boundaries::LONG_WORD_END => {
                Boundaries::is_long_word_end(self.column, &self.current_line)
            }
            _ => {
                debug_assert!(false);
                false
            }
        })
    }

    pub fn prev_grapheme(&mut self) -> bool {
        debug_assert_eq!(self.column, self.graphemes.cur_cursor());

        if self.column == 0 {
            if self.line == 0 {
                return false;
            }

            let current_line = self.rope.line(self.line - 1).to_string();
            self.line -= 1;
            self.column = current_line.len();
            self.current_line = current_line;
            self.graphemes = GraphemeCursor::new(self.column, self.current_line.len(), true);
        }

        match self.graphemes.prev_boundary(&self.current_line, 0).unwrap() {
            Some(column) => {
                let str = &self.current_line[column..self.column];
                self.index -= str.len();
                self.column = column;

                true
            }
            None => unreachable!(),
        }
    }

    pub fn next_grapheme(&mut self) -> bool {
        debug_assert_eq!(self.column, self.graphemes.cur_cursor());

        match self.graphemes.next_boundary(&self.current_line, 0).unwrap() {
            Some(column) => {
                let str = &self.current_line[self.column..column];
                self.index += str.len();
                self.column = column;

                if matches!(str, "\r" | "\n" | "\r\n") {
                    debug_assert!(self.line < self.rope.len_lines() - 1);

                    self.line += 1;
                    self.column = 0;
                    self.current_line = self.rope.line(self.line).to_string();
                    self.graphemes =
                        GraphemeCursor::new(self.column, self.current_line.len(), true);
                }

                true
            }
            None => {
                debug_assert_eq!(self.line, self.rope.len_lines() - 1);
                false
            }
        }
    }

    pub fn prev(&mut self, boundaries: Boundaries) -> bool {
        if boundaries.is_empty() {
            return self.prev_grapheme();
        }

        let index = self.index;
        let line = self.line;
        let column = self.column;
        let graphemes = self.graphemes.clone();

        loop {
            if !self.prev_grapheme() {
                // Restore
                if self.index != index {
                    self.index = index;
                    self.line = line;
                    self.column = column;
                    self.current_line = self.rope.line(line).to_string();
                    self.graphemes = graphemes;
                }

                return false;
            }

            if self.is_boundaries(boundaries) {
                return true;
            }
        }
    }

    pub fn next(&mut self, boundaries: Boundaries) -> bool {
        if boundaries.is_empty() {
            return self.next_grapheme();
        }

        let index = self.index;
        let line = self.line;
        let column = self.column;
        let graphemes = self.graphemes.clone();

        loop {
            if !self.next_grapheme() {
                // Restore
                if self.index != index {
                    self.index = index;
                    self.line = line;
                    self.column = column;
                    self.current_line = self.rope.line(line).to_string();
                    self.graphemes = graphemes;
                }

                return false;
            }

            if self.is_boundaries(boundaries) {
                return true;
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
    use crate::rope::CursorRef;
    use std::collections::HashSet;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn graphemes() {
        let str = "\rHello\n  This is {(foo-bar_baz12)}\r\n  !";
        let start = Cursor::default();

        assert_eq!(
            {
                let mut segmentation = Segmentation::new(Rope::from(str), 39, 3, 3);
                let mut index = str.len();
                let mut graphemes = Vec::new();

                while segmentation.prev_grapheme() {
                    graphemes.push(&str[segmentation.index..index]);
                    index = segmentation.index;
                }

                graphemes
            },
            str.graphemes(true).rev().collect::<Vec<_>>(),
        );
        assert_eq!(
            {
                let mut segmentation = Segmentation::new(Rope::from(str), 0, 0, 0);
                let mut index = 0;
                let mut graphemes = Vec::new();

                while segmentation.next_grapheme() {
                    graphemes.push(&str[index..segmentation.index]);
                    index = segmentation.index;
                }

                graphemes
            },
            str.graphemes(true).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn boundaries() {
        const DEBUG: bool = true;

        let data = [
            (
                Boundaries::LINE_START,
                vec![Cursor::extract("┃"), Cursor::extract("┃12\r┃34\n┃5\r\n┃")],
            ),
            (
                Boundaries::LINE_END,
                vec![Cursor::extract("┃"), Cursor::extract("12┃\r34┃\n5┃\r\n┃")],
            ),
            (
                Boundaries::LINE_FIRST,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("┃12\r┃34\n┃5\r\n"),
                    Cursor::extract(" ┃12\r ┃34\n ┃5\r\n"),
                ],
            ),
            (
                Boundaries::LINE_LAST,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("12┃\r34┃\n5┃\r\n"),
                    Cursor::extract("12┃ \r34┃ \n5┃ \r\n"),
                ],
            ),
            (
                Boundaries::PUNCTUATION_START,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("┃__  ┃__--__  ┃--  ┃--__--"),
                    Cursor::extract("┃(ab┃) ┃(__ab__┃) ┃(__--ab--__┃) ┃(--ab--┃) ┃(--__ab__--┃)"),
                    Cursor::extract("ab12CDEf_-_-ij🚀🦀kl"),
                ],
            ),
            (
                Boundaries::PUNCTUATION_END,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("__┃  __--__┃  --┃  --__--┃"),
                    Cursor::extract("(┃ab)┃ (┃__ab__)┃ (┃__--ab--__)┃ (┃--ab--)┃ (┃--__ab__--)┃"),
                    Cursor::extract("ab12CDEf_-_-ij🚀🦀kl"),
                ],
            ),
            (
                Boundaries::SHORT_WORD_START,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("__  __--__  --  --__--"),
                    Cursor::extract("(┃ab) (__┃ab__) (__--┃ab--__) (--┃ab--) (--__┃ab__--)"),
                    Cursor::extract("┃ab┃12┃CD┃Ef_-_-┃ij┃🚀🦀┃kl"),
                ],
            ),
            (
                Boundaries::SHORT_WORD_END,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("__  __--__  --  --__--"),
                    Cursor::extract("(ab┃) (__ab┃__) (__--ab┃--__) (--ab┃--) (--__ab┃__--)"),
                    Cursor::extract("ab┃12┃CD┃Ef┃_-_-ij┃🚀🦀┃kl┃"),
                ],
            ),
            (
                Boundaries::LONG_WORD_START,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("__  __--__  --  --__--"),
                    Cursor::extract("(┃ab) (┃__ab__) (┃__--ab--__) (┃--ab--) (┃--__ab__--)"),
                    Cursor::extract("┃ab12CDEf_-_-ij🚀🦀kl"),
                ],
            ),
            (
                Boundaries::LONG_WORD_END,
                vec![
                    Cursor::extract(""),
                    Cursor::extract("__  __--__  --  --__--"),
                    Cursor::extract("(ab┃) (__ab__┃) (__--ab--__┃) (--ab--┃) (--__ab__--┃)"),
                    Cursor::extract("ab12CDEf_-_-ij🚀🦀kl┃"),
                ],
            ),
        ];

        for (expected, data) in data {
            println!("Expecting {expected:?}:");

            for (rope, cursors) in data {
                let str = &rope.to_string();
                let cursors = cursors
                    .into_iter()
                    .map(|cursor| cursor.index)
                    .collect::<HashSet<_>>();

                for (index, _) in str.grapheme_indices(true).chain([(str.len(), "")]) {
                    let is_expected = {
                        let cursor = CursorRef::with(rope.clone().slice(..))
                            .at_index(index)
                            .as_cursor();

                        Segmentation::new(rope.clone(), cursor.index, cursor.line, cursor.column)
                            .is_boundaries(expected)
                    };

                    println!(
                        "  ({}) {:?}",
                        if cursors.contains(&index) { "y" } else { "n" },
                        {
                            let mut str = str.clone();
                            str.insert(index, '┃');
                            str
                        },
                    );

                    if cursors.contains(&index) {
                        assert!(is_expected);
                    } else {
                        assert!(!is_expected);
                    }
                }
            }
        }
    }
}
