use crate::cursor::Cursor;
use ropey::{iter::Chunks, Rope, RopeSlice};
use std::ops::Range;
use tree_sitter::{InputEdit, Point};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Text                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
enum Inner {
    String(String),
    Rope(Rope),
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Clone, Debug)]
pub struct Text {
    inner: Inner,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            inner: Inner::String(String::new()),
        }
    }
}

impl<'a> From<&'a str> for Text {
    fn from(str: &'a str) -> Self {
        Self {
            inner: Inner::String(str.into()),
        }
    }
}

impl<'a> From<RopeSlice<'a>> for Text {
    fn from(slice: RopeSlice<'a>) -> Self {
        Self {
            inner: if slice.len_bytes() <= Self::MAX_BYTES {
                Inner::String(slice.into())
            } else {
                Inner::Rope(slice.into())
            },
        }
    }
}

impl Text {
    /// @see [`Rope::try_insert()`] comments.
    pub const MAX_BYTES: usize = 6 * 984; // 6 * ropey::MAX_BYTES

    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::String(string) => string.len(),
            Inner::Rope(rope) => rope.len_bytes(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn chunks(&self) -> impl Iterator<Item = &str> {
        enum Iter<'a> {
            String(Option<&'a str>),
            Rope(Chunks<'a>),
        }

        let mut iter = match &self.inner {
            Inner::String(string) => Iter::String(Some(&string)),
            Inner::Rope(rope) => Iter::Rope(rope.chunks()),
        };

        std::iter::from_fn(move || match &mut iter {
            Iter::String(string) => string.take(),
            Iter::Rope(chunks) => chunks.next(),
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Edit                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Describes an edit in a document.
#[derive(Clone, Debug)]
pub struct Edit {
    start: Cursor,
    removed_end: Cursor,
    inserted_end: Cursor,
    removed: Text,
    inserted: Text,
}

impl Edit {
    pub fn new(
        start: Cursor,
        removed_end: Cursor,
        inserted_end: Cursor,
        removed: Text,
        inserted: Text,
    ) -> Self {
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

    pub fn start(&self) -> Cursor {
        self.start
    }

    pub fn removed_end(&self) -> Cursor {
        self.removed_end
    }

    pub fn inserted_end(&self) -> Cursor {
        self.inserted_end
    }

    pub fn input_edit(&self) -> InputEdit {
        Self::input_edit_impl(self.start, self.removed_end, self.inserted_end)
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

    pub fn insert(rope: &mut Rope, index: usize, inserted: &Text) {
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

    pub fn remove(rope: &mut Rope, range: Range<usize>) -> Rope {
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

    pub fn replace(rope: &mut Rope, range: Range<usize>, inserted: &Text) -> Rope {
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
}

/// Private.
impl Edit {
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
            (true, false) => Edit::insert(rope, index, inserted),
            (false, true) => {
                Edit::remove(rope, range);
            }
            (false, false) => {
                Edit::replace(rope, range, inserted);
            }
        }

        Self::input_edit_impl(start, removed_end, inserted_end)
    }

    fn input_edit_impl(start: Cursor, removed_end: Cursor, inserted_end: Cursor) -> InputEdit {
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
