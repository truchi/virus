use crate::rope::Cursor;
use ropey::Rope;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

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

// TODO OPENING_PAIR_START OPENING_PAIR_END QUOTE_START QUOTE_END
bitflags::bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Boundaries: u16 {
        const LINE_START        = 1 << 1;
        const LINE_END          = 1 << 2;
        const LINE_FIRST        = 1 << 3;
        const LINE_LAST         = 1 << 4;
        const PUNCTUATION_START = 1 << 5;
        const PUNCTUATION_END   = 1 << 6;
        const SHORT_WORD_START  = 1 << 7;
        const SHORT_WORD_END    = 1 << 8;
        const LONG_WORD_START   = 1 << 9;
        const LONG_WORD_END     = 1 << 10;

        const GRAPHEME          = 0;
        const PUNCTUATION       = Self::PUNCTUATION_START.bits() | Self::PUNCTUATION_END.bits();
        const SHORT_WORD        = Self::SHORT_WORD_START.bits()  | Self::SHORT_WORD_END.bits();
        const LONG_WORD         = Self::LONG_WORD_START.bits()   | Self::LONG_WORD_END.bits();
        const WORD              = Self::SHORT_WORD.bits()        | Self::LONG_WORD.bits();
    }
}

impl Boundaries {
    pub fn from_bools(
        punctuation_start: bool,
        punctuation_end: bool,
        short_word_start: bool,
        short_word_end: bool,
        long_word_start: bool,
        long_word_end: bool,
    ) -> Self {
        let mut boundaries = Boundaries::empty();

        punctuation_start.then(|| boundaries.insert(Self::PUNCTUATION_START));
        punctuation_end.then(|| boundaries.insert(Self::PUNCTUATION_END));
        short_word_start.then(|| boundaries.insert(Self::SHORT_WORD_START));
        short_word_end.then(|| boundaries.insert(Self::SHORT_WORD_END));
        long_word_start.then(|| boundaries.insert(Self::LONG_WORD_START));
        long_word_end.then(|| boundaries.insert(Self::LONG_WORD_END));

        boundaries
    }
}

#[allow(unreachable_patterns)]
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

            let (mut left, right) = graphemes(index, line);

            match left() {
                alphanum!() | Some(Separator) => false,
                whitespace!() | Some(Punctuation) => is_word(right),
            }
        }

        pub(super) fn is_long_word_end(index: usize, line: &str) -> bool {
            debug_assert!(index <= line.len());

            let (left, mut right) = graphemes(index, line);

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

    pub fn cursor(&self) -> Cursor {
        Cursor {
            index: self.index,
            line: self.line,
            column: self.column,
            width: self.current_line[..self.column].width(),
        }
    }

    pub fn current_line(&self) -> &str {
        &self.current_line
    }

    pub fn is_boundaries(&self, boundaries: Boundaries) -> bool {
        boundaries.iter().any(|boundaries| match () {
            _ if boundaries == Boundaries::GRAPHEME => true,
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

    pub fn to_start(&mut self) {
        self.to_cursor(Cursor::builder(self.rope.slice(..)).at_start())
    }

    pub fn to_end(&mut self) {
        self.to_cursor(Cursor::builder(self.rope.slice(..)).at_end())
    }

    pub fn to_line_start(&mut self) {
        self.to_column(0);
    }

    pub fn to_line_end(&mut self) {
        self.to_column(
            self.current_line.len()
                - self
                    .current_line
                    .graphemes(true)
                    .rev()
                    .map(|grapheme| (grapheme, GraphemeCategory::from(grapheme)))
                    .take_while(|(_, category)| matches!(category, GraphemeCategory::Break))
                    .map(|(grapheme, _)| grapheme.len())
                    .sum::<usize>(),
        );
    }

    pub fn to_line_first(&mut self) {
        self.to_column(
            self.current_line
                .graphemes(true)
                .map(|grapheme| (grapheme, GraphemeCategory::from(grapheme)))
                .take_while(|(_, category)| matches!(category, GraphemeCategory::Space))
                .map(|(grapheme, _)| grapheme.len())
                .sum(),
        );
    }

    pub fn to_line_last(&mut self) {
        self.to_column(
            self.current_line.len()
                - self
                    .current_line
                    .graphemes(true)
                    .rev()
                    .map(|grapheme| (grapheme, GraphemeCategory::from(grapheme)))
                    .take_while(|(_, category)| {
                        matches!(category, GraphemeCategory::Break | GraphemeCategory::Space)
                    })
                    .map(|(grapheme, _)| grapheme.len())
                    .sum::<usize>(),
        );
    }

    pub fn to_cursor(&mut self, cursor: Cursor) {
        if self.index == cursor.index {
            return;
        }

        if self.line != cursor.line {
            self.current_line = self.rope.line(cursor.line).to_string();
            self.graphemes = GraphemeCursor::new(cursor.column, self.current_line.len(), true);
        } else if self.column != cursor.column {
            self.graphemes = GraphemeCursor::new(cursor.column, self.current_line.len(), true);
        }

        (self.index, self.line, self.column) = (cursor.index, cursor.line, cursor.column);
    }

    pub fn update(&mut self, rope: Rope, cursor: Cursor) {
        if Rope::is_instance(&self.rope, &rope) {
            self.to_cursor(cursor);
        } else {
            *self = Self::new(rope, cursor.index, cursor.line, cursor.column);
        }
    }

    pub fn prev(&mut self, boundaries: Boundaries) -> bool {
        if boundaries == Boundaries::GRAPHEME {
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
        if boundaries == Boundaries::GRAPHEME {
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

/// Private.
impl Segmentation {
    fn to_column(&mut self, column: usize) {
        if self.column == column {
            return;
        }

        self.index = self.index - self.column + column;
        self.column = column;
        self.graphemes = GraphemeCursor::new(column, self.current_line.len(), true);
    }

    fn prev_grapheme(&mut self) -> bool {
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

    fn next_grapheme(&mut self) -> bool {
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
        let data = [
            (
                Boundaries::LINE_START,
                vec![
                    //
                    Cursor::extract("┃"),
                    Cursor::extract("┃12\r┃34\n┃5\r\n┃"),
                    // Cursor::extract("12\r  34\n5  \r\n  6\n"), // TODO test cases like this
                ],
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
