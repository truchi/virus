use crate::rope::{Cursor, Inner, Text};
use ropey::Rope;
use similar::{Algorithm, DiffOp, TextDiff};
use std::{ops::Range, time::Duration};
use tree_sitter::{InputEdit, Point};
use unicode_width::UnicodeWidthStr;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Edit                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Describes an edit in a document.
///
/// ```
///                   inserted
///                   vvvvvvvvv
/// OLD: aaabbbcccddd|XXXXXXXXX|eeefffggg
///  |               ^         ^ removed_end
///  | Edit          start
///  v               v                  v inserted_end
/// NEW: aaabbbcccddd|YYYYYYYYYYYYYYYYYY|eeefffggg
///                   ^^^^^^^^^^^^^^^^^^
///                   removed
/// ```
#[derive(Clone, Debug)]
pub struct Edit {
    start: Cursor,
    removed_end: Cursor,
    inserted_end: Cursor,
    removed: Text,
    inserted: Text,
}

impl Edit {
    pub fn edit(rope: &mut Rope, range: Range<Cursor>, inserted: Text) -> Self {
        debug_assert!(range.start <= range.end);

        let Range {
            start,
            end: removed_end,
        } = range;
        let insert = !inserted.is_empty();
        let remove = !range.is_empty();

        let (removed, inserted_end) = match (insert, remove) {
            // Replace
            (true, true) => (
                Self::replace(rope, start.index..removed_end.index, &inserted).into(),
                Cursor::build(rope.slice(..)).at_index(start.index + inserted.len()),
            ),
            // Insert
            (true, false) => (
                {
                    Self::insert(rope, start.index, &inserted);
                    Default::default()
                },
                Cursor::build(rope.slice(..)).at_index(start.index + inserted.len()),
            ),
            // Remove
            (false, true) => (
                Self::remove(rope, start.index..removed_end.index).into(),
                start,
            ),
            // Noop
            (false, false) => (Default::default(), removed_end),
        };

        Self::new(start, removed_end, inserted_end, removed, inserted)
    }

    pub fn diff<'a>(
        old: &'a str,
        new: &'a str,
        algorithm: Option<Algorithm>,
        timeout: Option<Duration>,
    ) -> impl 'a + Iterator<Item = Self> {
        fn update(mut cursor: Cursor, slices: &[&str]) -> Cursor {
            let mut len = 0;
            let mut lines = 0;
            let mut line_break = None;

            for (i, str) in slices.iter().copied().enumerate() {
                len += str.len();

                for (j, byte) in str.as_bytes().iter().copied().enumerate() {
                    if byte == b'\n' {
                        lines += 1;
                        line_break = Some((i, j));
                    }
                }
            }

            cursor.index += len;
            cursor.line += lines;

            if let Some((i, j)) = line_break {
                let last_line = &slices[i][j + 1..];
                cursor.column = last_line.len();
                cursor.width = last_line.width();
            } else {
                cursor.column += len;
                cursor.width += slices.iter().copied().map(str::width).sum::<usize>();
            }

            cursor
        }

        let diff = &mut TextDiff::configure();
        let diff = if let Some(algorithm) = algorithm {
            diff.algorithm(algorithm)
        } else {
            diff
        };
        let diff = if let Some(timeout) = timeout {
            diff.timeout(timeout)
        } else {
            diff
        };
        let diff = diff.diff_words(old, new);
        let mut index = 0;
        let mut start = Cursor::default();

        std::iter::from_fn(move || {
            let mut ops = diff.ops()[index..].iter();

            let edit = loop {
                let op = *ops.next()?;
                index += 1;

                match op {
                    DiffOp::Equal { new_index, len, .. } => {
                        start = update(start, &diff.new_slices()[new_index..][..len]);
                    }
                    DiffOp::Delete {
                        old_index, old_len, ..
                    } => {
                        let olds = &diff.old_slices()[old_index..][..old_len];
                        let removed_end = update(start, olds);

                        break Self::new(
                            start,
                            removed_end,
                            start,
                            Text::from(olds.iter().copied().collect::<String>()),
                            Text::default(),
                        );
                    }
                    DiffOp::Insert {
                        new_index, new_len, ..
                    } => {
                        let news = &diff.new_slices()[new_index..][..new_len];
                        let inserted_end = update(start, news);

                        break Self::new(
                            start,
                            start,
                            inserted_end,
                            Text::default(),
                            Text::from(news.iter().copied().collect::<String>()),
                        );
                    }
                    DiffOp::Replace {
                        old_index,
                        old_len,
                        new_index,
                        new_len,
                    } => {
                        let olds = &diff.old_slices()[old_index..][..old_len];
                        let news = &diff.new_slices()[new_index..][..new_len];
                        let removed_end = update(start, olds);
                        let inserted_end = update(start, news);

                        break Self::new(
                            start,
                            removed_end,
                            inserted_end,
                            Text::from(olds.iter().copied().collect::<String>()),
                            Text::from(news.iter().copied().collect::<String>()),
                        );
                    }
                }
            };

            start = edit.inserted_end;
            Some(edit)
        })
    }

    pub fn apply(&self, rope: &mut Rope) -> InputEdit {
        Self::apply_impl(
            rope,
            self.start,
            self.removed_end,
            self.inserted_end,
            &self.removed,
            &self.inserted,
        )
    }

    pub fn unapply(&self, rope: &mut Rope) -> InputEdit {
        Self::apply_impl(
            rope,
            self.start,
            self.inserted_end,
            self.removed_end,
            &self.inserted,
            &self.removed,
        )
    }

    /// Returns whether this edit would leave some text unchanged,
    /// or `None` if the edit is larger that `Text::BREAKPOINT`.
    pub fn is_noop(&self) -> Option<bool> {
        if self.removed.len() != self.inserted.len() {
            return Some(false);
        }

        if self.removed.len() > Text::BREAKPOINT {
            return None;
        }

        Some(self.removed == self.inserted)
    }

    pub fn start(&self) -> Cursor {
        self.start
    }

    pub fn removed_end(&self) -> Cursor {
        self.removed_end
    }

    pub fn inserted_end(&self) -> Cursor {
        self.inserted_end
    }

    pub fn removed(&self) -> &Text {
        &self.removed
    }

    pub fn inserted(&self) -> &Text {
        &self.inserted
    }

    pub fn to_input_edit_applied(&self) -> InputEdit {
        Self::to_input_edit_impl(self.start, self.removed_end, self.inserted_end)
    }

    pub fn to_input_edit_unapplied(&self) -> InputEdit {
        Self::to_input_edit_impl(self.start, self.inserted_end, self.removed_end)
    }

    pub fn into_removed_and_inserted(self) -> (Text, Text) {
        (self.removed, self.inserted)
    }
}

/// Private.
impl Edit {
    fn new(
        start: Cursor,
        removed_end: Cursor,
        inserted_end: Cursor,
        removed: Text,
        inserted: Text,
    ) -> Self {
        debug_assert!(start <= removed_end);
        debug_assert!(start <= inserted_end);
        debug_assert!(removed.len() == (start.index..removed_end.index).len());
        debug_assert!(inserted.len() == (start.index..inserted_end.index).len());

        Self {
            start,
            removed_end,
            inserted_end,
            removed,
            inserted,
        }
    }

    fn remove(rope: &mut Rope, range: Range<usize>) -> Rope {
        debug_assert!(!range.is_empty());
        debug_assert!(rope.char_to_byte(rope.byte_to_char(range.start)) == range.start);
        debug_assert!(rope.char_to_byte(rope.byte_to_char(range.end)) == range.end);

        let start = rope.byte_to_char(range.start);
        let end = rope.byte_to_char(range.end);

        let right = rope.split_off(end);
        let removed = rope.split_off(start);

        rope.append(right);
        removed
    }

    fn insert(rope: &mut Rope, index: usize, inserted: &Text) {
        debug_assert!(!inserted.is_empty());
        debug_assert!(rope.char_to_byte(rope.byte_to_char(index)) == index);

        let index = rope.byte_to_char(index);

        match &inserted.inner {
            Inner::String(inserted) => rope.insert(index, inserted),
            Inner::Rope(inserted) => {
                let right = rope.split_off(index);
                rope.append(inserted.clone());
                rope.append(right);
            }
        }
    }

    fn replace(rope: &mut Rope, range: Range<usize>, inserted: &Text) -> Rope {
        debug_assert!(!inserted.is_empty());
        debug_assert!(!range.is_empty());
        debug_assert!(rope.char_to_byte(rope.byte_to_char(range.start)) == range.start);
        debug_assert!(rope.char_to_byte(rope.byte_to_char(range.end)) == range.end);

        let start = rope.byte_to_char(range.start);
        let end = rope.byte_to_char(range.end);

        let right = rope.split_off(end);
        let removed = rope.split_off(start);

        match &inserted.inner {
            Inner::String(text) => rope.insert(start, text),
            Inner::Rope(text) => rope.append(text.clone()),
        };

        rope.append(right);
        removed
    }

    fn apply_impl(
        rope: &mut Rope,
        start: Cursor,
        removed_end: Cursor,
        inserted_end: Cursor,
        removed: &Text,
        inserted: &Text,
    ) -> InputEdit {
        let index = start.index;
        let range = index..index + removed.len();

        match (removed.is_empty(), inserted.is_empty()) {
            (true, true) => {}
            (true, false) => {
                Self::insert(rope, index, inserted);
            }
            (false, true) => {
                Self::remove(rope, range);
            }
            (false, false) => {
                Self::replace(rope, range, inserted);
            }
        }

        Self::to_input_edit_impl(start, removed_end, inserted_end)
    }

    fn to_input_edit_impl(start: Cursor, removed_end: Cursor, inserted_end: Cursor) -> InputEdit {
        InputEdit {
            start_byte: start.index,
            old_end_byte: removed_end.index,
            new_end_byte: inserted_end.index,
            start_position: Point {
                row: start.line,
                column: start.column,
            },
            old_end_position: Point {
                row: removed_end.line,
                column: removed_end.column,
            },
            new_end_position: Point {
                row: inserted_end.line,
                column: inserted_end.column,
            },
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_unapply() {
        fn cursor(index: usize, line: usize, column: usize) -> Cursor {
            Cursor {
                index,
                line,
                column,
                width: column,
            }
        }
        fn to_text_string(str: &str) -> Text {
            Text {
                inner: Inner::String(str.into()),
            }
        }
        fn to_text_rope(str: &str) -> Text {
            Text {
                inner: Inner::Rope(str.into()),
            }
        }

        for (old, new, start, removed_end, inserted_end, removed, inserted) in [
            ("", "123", 0, 0, 3, "", "123"),
            ("abcdef", "abc123def", 3, 3, 6, "", "123"),
            ("123", "", 0, 3, 0, "123", ""),
            ("abc123def", "abcdef", 3, 6, 3, "123", ""),
            ("abc123def", "abcXYZdef", 3, 6, 6, "123", "XYZ"),
        ] {
            let start = cursor(start, 0, start);
            let removed_end = cursor(removed_end, 0, removed_end);
            let inserted_end = cursor(inserted_end, 0, inserted_end);

            for to_text in [to_text_string, to_text_rope] {
                let mut rope = Rope::from(old);
                let edit = Edit::edit(&mut rope, start..removed_end, to_text(inserted));

                assert_eq!(edit.start, start);
                assert_eq!(edit.removed_end, removed_end);
                assert_eq!(edit.inserted_end, inserted_end);
                assert_eq!(edit.removed, removed.into());
                assert_eq!(edit.inserted, inserted.into());
                assert_eq!(rope, Rope::from(new));

                edit.unapply(&mut rope);
                assert_eq!(rope, Rope::from(old));

                edit.apply(&mut rope);
                assert_eq!(rope, Rope::from(new));
            }
        }
    }

    #[test]
    fn diff() {
        let line_breaks = ["\n", "\r\n"];
        let algorithms = [Algorithm::Myers, Algorithm::Patience, Algorithm::Lcs];

        for br in line_breaks {
            for algorithm in algorithms {
                for (old, new) in [
                    (format!(""), format!("b")),
                    (format!("a"), format!("")),
                    (format!("a"), format!("b")),
                    (format!("a{br}"), format!("b")),
                    (format!("a"), format!("b{br}")),
                    (format!(" *   / b ! c ? "), format!(" * a /   ! d ? ")),
                    (
                        format!(" fn add ( a : u8 , b : u8 ) -> u8 {{ }} "),
                        format!("fn sub(c: u16, d: u16) -> u16 {{}}"),
                    ),
                    (
                        format!("Hello,{br}world!{br}I love Virus"),
                        format!("Salut,{br}tout le monde !{br}J'💖 🦀"),
                    ),
                ] {
                    assert_eq!(
                        Edit::diff(old.as_str(), new.as_str(), Some(algorithm), None)
                            .into_iter()
                            .fold(Rope::from(old.as_str()), |mut rope, edit| {
                                edit.apply(&mut rope);
                                rope
                            }),
                        new,
                    );
                }
            }
        }
    }
}
