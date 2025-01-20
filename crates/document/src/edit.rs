use crate::{
    cursor::Cursor, graphemes::GraphemesBackward, occurences::Occurences,
    segmentation::GraphemeCategory, smol::SmolCow,
};
use ropey::{Rope, RopeSlice};
use similar::{Algorithm, DiffOp, TextDiff};
use smol_str::{SmolStr, SmolStrBuilder};
use std::{ops::Range, time::Duration};
use tree_sitter::{InputEdit, Point, Tree};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Text                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Eq)]
pub enum Text {
    Smol(SmolStr),
    Rope(Rope),
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Smol(a), Self::Smol(b)) => a == b,
            (Self::Smol(a), Self::Rope(b)) => a.as_str() == b,
            (Self::Rope(a), Self::Smol(b)) => a == b.as_str(),
            (Self::Rope(a), Self::Rope(b)) => a == b,
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::Smol(SmolStr::default())
    }
}

impl<'a> From<&'a SmolStr> for Text {
    fn from(smol: &'a SmolStr) -> Self {
        if smol.len() <= Self::BREAKPOINT {
            Self::Smol(smol.clone())
        } else {
            Self::Rope(smol.as_str().into())
        }
    }
}

impl From<SmolStr> for Text {
    fn from(smol: SmolStr) -> Self {
        if smol.len() <= Self::BREAKPOINT {
            Self::Smol(smol)
        } else {
            Self::Rope(smol.as_str().into())
        }
    }
}

impl<'a> From<RopeSlice<'a>> for Text {
    fn from(slice: RopeSlice<'a>) -> Self {
        if slice.len_bytes() <= Self::BREAKPOINT {
            Self::Smol({
                let mut smol = SmolStrBuilder::new();

                for chunk in slice.chunks() {
                    smol.push_str(chunk);
                }

                smol.finish()
            })
        } else {
            Self::Rope(slice.into())
        }
    }
}

impl<'a> From<Rope> for Text {
    fn from(rope: Rope) -> Self {
        if rope.len_bytes() <= Self::BREAKPOINT {
            Self::Smol({
                let mut smol = SmolStrBuilder::new();

                for chunk in rope.chunks() {
                    smol.push_str(chunk);
                }

                smol.finish()
            })
        } else {
            Self::Rope(rope)
        }
    }
}

impl Text {
    /// `text.len() <= Self::BREAKPOINT ? Self::Smol(text) : Self::Rope(text)`.
    ///
    /// 6 chunks of text data.
    /// We want to reduce memory overhead of ropes for "small" text
    /// while still having structural sharing for "big" ropes.
    ///
    /// Also, as this text is to be inserted in document ropes, we align to the
    /// [`Rope::try_insert()`] logic.
    /// @see [`Rope::try_insert()`] comments.
    pub const BREAKPOINT: usize = 6 * 984;

    pub fn len(&self) -> usize {
        match self {
            Self::Smol(smol) => smol.len(),
            Self::Rope(rope) => rope.len_bytes(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn trailing_line_break(&mut self, bool: bool) {
        let trailing_line_break_len = match self {
            Self::Smol(smol) => smol
                .as_str()
                .graphemes(true)
                .rev()
                .next()
                .map(|grapheme| SmolCow::Str(grapheme)),
            Self::Rope(rope) => GraphemesBackward::new(rope.slice(..)).next(),
        }
        .map(|grapheme| (GraphemeCategory::from(grapheme.as_str()), grapheme));

        if let Some((GraphemeCategory::Break, grapheme)) = trailing_line_break_len {
            if !bool {
                let bytes = grapheme.as_str().len();
                let chars = grapheme.as_str().chars().count();

                match self {
                    Self::Smol(smol) => *smol = SmolStr::new(&smol[..smol.len() - bytes]),
                    Self::Rope(rope) => rope.remove(rope.len_chars() - chars..rope.len_chars()),
                }
            }
        } else {
            if bool {
                match self {
                    Self::Smol(smol) => {
                        let mut builder = SmolStrBuilder::new();
                        builder.push_str(smol);
                        builder.push_str("\n");
                        *smol = builder.finish();
                    }
                    Self::Rope(rope) => rope.insert(rope.len_chars(), "\n"),
                }
            }
        }
    }
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\"",
            match self {
                Self::Smol(smol) => smol.to_string(),
                Self::Rope(rope) => rope.to_string(),
            },
        )
    }
}

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
        const ALGORITHM: Algorithm = Algorithm::Myers;

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
                cursor.column = slices[i][j + 1..].len();
                cursor.width = slices[i][j + 1..].width();

                for line in &slices[i + 1..] {
                    cursor.column += line.len();
                    cursor.width += line.width();
                }
            } else {
                cursor.column += len;
                cursor.width += slices.iter().copied().map(str::width).sum::<usize>();
            }

            cursor
        }

        let diff = &mut TextDiff::configure();
        let diff = diff.algorithm(algorithm.unwrap_or(ALGORITHM));
        let diff = if let Some(timeout) = timeout {
            diff.timeout(timeout)
        } else {
            diff
        };
        let diff = diff.diff_chars(old, new);
        let mut index = 0;
        let mut start = Cursor::default();

        std::iter::from_fn(move || {
            let mut ops = diff.ops()[index..].iter();

            let edit = loop {
                let op = *ops.next()?;
                index += 1;

                match op {
                    DiffOp::Equal { new_index, len, .. } => {
                        if index == diff.ops().len() {
                            return None;
                        }

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
                            Text::from(olds.iter().copied().collect::<SmolStr>()),
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
                            Text::from(news.iter().copied().collect::<SmolStr>()),
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
                            Text::from(olds.iter().copied().collect::<SmolStr>()),
                            Text::from(news.iter().copied().collect::<SmolStr>()),
                        );
                    }
                }
            };

            start = edit.inserted_end;
            Some(edit)
        })
    }

    pub fn apply_rope(&self, rope: &mut Rope) {
        Self::apply_rope_impl(rope, self.start, &self.removed, &self.inserted);
    }

    pub fn unapply_rope(&self, rope: &mut Rope) {
        Self::apply_rope_impl(rope, self.start, &self.inserted, &self.removed);
    }

    pub fn apply_cursor(&self, cursor: Cursor) -> Option<Cursor> {
        Self::apply_cursor_impl(cursor, self.start, self.removed_end, self.inserted_end)
    }

    pub fn unapply_cursor(&self, cursor: Cursor) -> Option<Cursor> {
        Self::apply_cursor_impl(cursor, self.start, self.inserted_end, self.removed_end)
    }

    pub fn apply_tree(&self, tree: &mut Tree) {
        Self::apply_tree_impl(tree, self.start, self.removed_end, self.inserted_end);
    }

    pub fn unapply_tree(&self, tree: &mut Tree) {
        Self::apply_tree_impl(tree, self.start, self.inserted_end, self.removed_end);
    }

    pub fn apply_occurences(&self, occurences: &mut Occurences, rope: Rope) {
        occurences.update(rope, self.start, self.removed_end, self.inserted_end);
    }

    pub fn unapply_occurences(&self, occurences: &mut Occurences, rope: Rope) {
        occurences.update(rope, self.start, self.inserted_end, self.removed_end)
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

        match inserted {
            Text::Smol(inserted) => rope.insert(index, inserted),
            Text::Rope(inserted) => {
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

        match inserted {
            Text::Smol(inserted) => rope.insert(start, inserted),
            Text::Rope(inserted) => rope.append(inserted.clone()),
        };

        rope.append(right);
        removed
    }

    fn apply_rope_impl(rope: &mut Rope, start: Cursor, removed: &Text, inserted: &Text) {
        let index = start.index;
        let range = index..index + removed.len();

        match (removed.is_empty(), inserted.is_empty()) {
            (true, true) => {}
            (true, false) => Self::insert(rope, index, inserted),
            (false, true) => drop(Self::remove(rope, range)),
            (false, false) => drop(Self::replace(rope, range, inserted)),
        }
    }

    fn apply_cursor_impl(
        cursor: Cursor,
        start: Cursor,
        removed_end: Cursor,
        inserted_end: Cursor,
    ) -> Option<Cursor> {
        // Before edit
        if cursor.index <= start.index {
            return Some(cursor);
        }

        // Inside edit
        if start.index < cursor.index && cursor.index < removed_end.index {
            return None;
        }

        debug_assert!(removed_end.line <= cursor.line);

        let (index, line) = (
            cursor.index - (removed_end.index - start.index) + (inserted_end.index - start.index),
            cursor.line - (removed_end.line - start.line) + (inserted_end.line - start.line),
        );

        // Below edit
        if removed_end.line < cursor.line {
            return Some(Cursor {
                index,
                line,
                column: cursor.column,
                width: cursor.width,
            });
        }

        debug_assert!(removed_end.line == cursor.line);

        let (column, width) = (
            cursor.column - removed_end.column + inserted_end.column,
            cursor.width - removed_end.width + inserted_end.width,
        );

        // On edit's last line
        Some(Cursor {
            index,
            line,
            column,
            width,
        })
    }

    fn apply_tree_impl(tree: &mut Tree, start: Cursor, removed_end: Cursor, inserted_end: Cursor) {
        tree.edit(&InputEdit {
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
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::Selection;

    #[test]
    fn rope() {
        fn cursor(index: usize, line: usize, column: usize) -> Cursor {
            Cursor {
                index,
                line,
                column,
                width: column,
            }
        }
        fn to_text_smol(str: &str) -> Text {
            Text::Smol(str.into())
        }
        fn to_text_rope(str: &str) -> Text {
            Text::Rope(str.into())
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

            for to_text in [to_text_smol, to_text_rope] {
                let mut rope = Rope::from(old);
                let edit = Edit::edit(&mut rope, start..removed_end, to_text(inserted));

                assert_eq!(edit.start, start);
                assert_eq!(edit.removed_end, removed_end);
                assert_eq!(edit.inserted_end, inserted_end);
                assert_eq!(edit.removed, SmolStr::new(removed).into());
                assert_eq!(edit.inserted, SmolStr::new(inserted).into());
                assert_eq!(rope, Rope::from(new));

                edit.unapply_rope(&mut rope);
                assert_eq!(rope, Rope::from(old));

                edit.apply_rope(&mut rope);
                assert_eq!(rope, Rope::from(new));
            }
        }
    }

    #[test]
    fn cursor() {
        for (removed, inserted, data) in [
            (
                "a┃bc┃d",
                "a┃123┃d",
                vec![
                    ("┃abcd", "┃a123d"),
                    ("a┃bcd", "a┃123d"),
                    ("ab┃cd", "a123d"),
                    ("abc┃d", "a123┃d"),
                    ("abcd┃", "a123d┃"),
                ],
            ),
            (
                "1┃234\n567┃8\n90",
                "1┃ab\ncd┃8\n90",
                vec![
                    ("┃1234\n5678\n90", "┃1ab\ncd8\n90"),
                    ("1┃234\n5678\n90", "1┃ab\ncd8\n90"),
                    ("12┃34\n5678\n90", "1ab\ncd8\n90"),
                    ("123┃4\n5678\n90", "1ab\ncd8\n90"),
                    ("1234┃\n5678\n90", "1ab\ncd8\n90"),
                    ("1234\n┃5678\n90", "1ab\ncd8\n90"),
                    ("1234\n5┃678\n90", "1ab\ncd8\n90"),
                    ("1234\n56┃78\n90", "1ab\ncd8\n90"),
                    ("1234\n567┃8\n90", "1ab\ncd┃8\n90"),
                    ("1234\n5678┃\n90", "1ab\ncd8┃\n90"),
                    ("1234\n5678\n┃90", "1ab\ncd8\n┃90"),
                    ("1234\n5678\n9┃0", "1ab\ncd8\n9┃0"),
                    ("1234\n5678\n90┃", "1ab\ncd8\n90┃"),
                ],
            ),
        ] {
            for (old, new) in data {
                let (old_rope, old_cursor) = Cursor::extract(old);
                let (new_rope, new_cursor) = Cursor::extract_optional(new);
                let (removed_rope, removed_selection) = Selection::extract(removed);
                let (inserted_rope, inserted_selection) = Selection::extract(inserted);

                // Assert test data is valid
                assert_eq!(old_rope, removed_rope);
                assert_eq!(removed_selection.anchor, inserted_selection.anchor);
                assert_eq!(inserted_rope, new_rope);

                let applied_cursor = Edit::apply_cursor_impl(
                    old_cursor,
                    removed_selection.anchor,
                    removed_selection.head,
                    inserted_selection.head,
                );

                assert_eq!(applied_cursor, new_cursor);

                if let Some(applied_cursor) = applied_cursor {
                    let unapplied_cursor = Edit::apply_cursor_impl(
                        applied_cursor,
                        removed_selection.anchor,
                        inserted_selection.head,
                        removed_selection.head,
                    );

                    assert_eq!(unapplied_cursor, Some(old_cursor));
                }
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
                                edit.apply_rope(&mut rope);
                                rope
                            }),
                        new,
                    );
                }
            }
        }
    }
}
