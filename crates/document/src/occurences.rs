use crate::{
    cursor::Cursor,
    segmentation::{Boundaries, Segmentation},
};
use ropey::Rope;
use std::collections::HashMap;

/// Occurences of [`Boundaries::LONG_WORD`]s in a [`Rope`].
#[derive(Debug)]
pub struct Occurences {
    items: HashMap<String, usize>,
    segmentation: Segmentation,
}

impl Occurences {
    /// Creates a new [`Occurences`] from `rope`.
    pub fn new(rope: Rope) -> Self {
        let mut completions = Self {
            items: HashMap::default(),
            segmentation: Segmentation::new(rope, 0, 0, 0),
        };

        completions.scan(Cursor::default(), None, Self::add);
        completions
    }

    /// Returns the occurences.
    pub fn items(&self) -> &HashMap<String, usize> {
        debug_assert!(self.items.values().all(|n| *n != 0));

        &self.items
    }

    /// Updates the occurences after editing from `start` to `removed_end`/`inserted_end`
    /// giving a new `rope`.
    pub fn update(&mut self, rope: Rope, start: Cursor, removed_end: Cursor, inserted_end: Cursor) {
        const LINE_FIRST: Boundaries = Boundaries::LINE_FIRST;
        const LINE_LAST: Boundaries = Boundaries::LINE_LAST;
        const LONG_WORD_START: Boundaries = Boundaries::LONG_WORD_START;
        const LONG_WORD_END: Boundaries = Boundaries::LONG_WORD_END;
        const PUNCTUATION: Boundaries = Boundaries::PUNCTUATION;

        let start = {
            self.segmentation.to_cursor(start);
            self.segmentation
                .prev(LINE_FIRST | LONG_WORD_START | PUNCTUATION);
            self.segmentation.cursor()
        };

        let end = {
            self.segmentation.to_cursor(removed_end);
            self.segmentation
                .next(LINE_LAST | LONG_WORD_END | PUNCTUATION);
            self.segmentation.cursor().index
        };

        let (removed_end, inserted_end) = (end, inserted_end.index + end - removed_end.index);

        self.scan(start, Some(removed_end), Self::remove);
        self.segmentation.update(rope, start);
        self.scan(start, Some(inserted_end), Self::add);
    }
}

/// Private.
impl Occurences {
    /// Applies `f` to words in `start.index..end` or `start.index..`.
    ///
    /// Includes words starting before `end` (exclusive) and ending before `end` (inclusive).
    /// Panics if a word overlaps the range.
    fn scan(
        &mut self,
        start: Cursor,
        end: Option<usize>,
        f: impl Fn(&mut HashMap<String, usize>, &str),
    ) {
        self.segmentation.to_cursor(start);

        let mut cursor = self
            .segmentation
            .is_boundaries(Boundaries::LONG_WORD_START)
            .then_some(start);

        while self.segmentation.next(Boundaries::LONG_WORD) {
            cursor = if let Some((word_start, word_end)) =
                cursor.map(|cursor| (cursor, self.segmentation.cursor()))
            {
                debug_assert_eq!(word_start.line, word_end.line);
                debug_assert!(self.segmentation.is_boundaries(Boundaries::LONG_WORD_END));
                debug_assert!(!matches!(end, Some(end) if end < word_end.index));

                f(
                    &mut self.items,
                    &self.segmentation.current_line()[word_start.column..word_end.column],
                );

                None
            } else {
                debug_assert!(self.segmentation.is_boundaries(Boundaries::LONG_WORD_START));

                if matches!(end, Some(end) if end <= self.segmentation.cursor().index) {
                    break;
                }

                Some(self.segmentation.cursor())
            };
        }
    }

    fn add(items: &mut HashMap<String, usize>, str: &str) {
        if let Some(count) = items.get_mut(str) {
            debug_assert_ne!(*count, 0);

            *count += 1;
        } else {
            items.insert(str.to_owned(), 1);
        }
    }

    fn remove(items: &mut HashMap<String, usize>, str: &str) {
        if let Some(count) = items.get_mut(str) {
            debug_assert_ne!(*count, 0);

            if *count == 1 {
                items.remove(str);
            } else {
                *count -= 1;
            }
        } else {
            items.insert(str.to_owned(), 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{edit::Edit, selection::Selection};

    fn sort<'a>(items: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
        let mut items = items.into_iter().collect::<Vec<_>>();
        items.sort();
        items
    }

    #[test]
    fn new() {
        for (rope, items) in [
            ("hello", vec!["hello"]),
            (" world", vec!["world"]),
            (
                "pub struct Foo {{\n    bar: Bar,\n}}",
                vec!["pub", "struct", "Foo", "bar", "Bar"],
            ),
        ] {
            assert_eq!(
                sort(
                    Occurences::new(Rope::from(rope))
                        .items()
                        .keys()
                        .map(String::as_str),
                ),
                sort(items),
            );
        }
    }

    #[test]
    fn edit() {
        for (rope, inserted, before, after) in [
            ("┃hello┃", "world", vec!["hello"], vec!["world"]),
            ("he┃ll┃o", "xx", vec!["hello"], vec!["hexxo"]),
            (
                "he ┃ll┃ o",
                "xx",
                vec!["he", "ll", "o"],
                vec!["he", "xx", "o"],
            ),
            (
                "he┃ ll ┃o",
                " xx ",
                vec!["he", "ll", "o"],
                vec!["he", "xx", "o"],
            ),
            ("he┃ ll ┃o", "xx", vec!["he", "ll", "o"], vec!["hexxo"]),
            (
                "he(┃ ll ┃)o",
                "xx",
                vec!["he", "ll", "o"],
                vec!["he", "xx", "o"],
            ),
        ] {
            let (rope, selection) = Selection::extract(rope);
            let mut edited = rope.clone();
            let edit = Edit::edit(&mut edited, selection.range(), inserted.into());
            let mut occurences = Occurences::new(rope);

            assert_eq!(
                sort(occurences.items().keys().map(String::as_str)),
                sort(before),
            );

            occurences.update(
                edited,
                edit.start(),
                edit.removed_end(),
                edit.inserted_end(),
            );

            assert_eq!(
                sort(occurences.items().keys().map(String::as_str)),
                sort(after),
            );
        }
    }
}
