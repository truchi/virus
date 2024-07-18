use crate::{cursor::Cursor, rope::RopeExt};
use ropey::{Rope, RopeSlice};
use std::ops::Range;
use tree_sitter::{InputEdit, Point};
use virus_lsp::type_aliases::TextDocumentContentChangeEventRangeAndText;

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

impl<'a> From<String> for Text {
    fn from(string: String) -> Self {
        Self {
            inner: Inner::String(string),
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

impl<'a> From<Rope> for Text {
    fn from(rope: Rope) -> Self {
        Self {
            inner: if rope.len_bytes() <= Self::MAX_BYTES {
                Inner::String(rope.into())
            } else {
                Inner::Rope(rope)
            },
        }
    }
}

impl ToString for Text {
    fn to_string(&self) -> String {
        match &self.inner {
            Inner::String(string) => string.to_string(),
            Inner::Rope(rope) => rope.to_string(),
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

    pub fn removed(&self) -> &Text {
        &self.removed
    }

    pub fn inserted(&self) -> &Text {
        &self.inserted
    }

    pub fn to_ts_edit_applied(&self) -> InputEdit {
        Self::to_input_edit_impl(self.start, self.removed_end, self.inserted_end)
    }

    pub fn to_ts_edit_unapplied(&self) -> InputEdit {
        Self::to_input_edit_impl(self.start, self.inserted_end, self.removed_end)
    }

    pub fn to_lsp_edit_applied(&self) -> TextDocumentContentChangeEventRangeAndText {
        Self::to_lsp_edit_impl(self.start, self.removed_end, &self.inserted)
    }

    pub fn to_lsp_edit_unapplied(&self) -> TextDocumentContentChangeEventRangeAndText {
        Self::to_lsp_edit_impl(self.start, self.inserted_end, &self.removed)
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
                rope.cursor()
                    .at_index(start.index + inserted.len())
                    .cursor(rope),
            ),
            // Insert
            (true, false) => (
                {
                    Self::insert(rope, start.index, &inserted);
                    Default::default()
                },
                rope.cursor()
                    .at_index(start.index + inserted.len())
                    .cursor(rope),
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
}

/// Private.
impl Edit {
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

    fn to_lsp_edit_impl(
        start: Cursor,
        removed_end: Cursor,
        inserted: &Text,
    ) -> TextDocumentContentChangeEventRangeAndText {
        use virus_lsp::{
            structures::{Position, Range},
            UInteger,
        };

        TextDocumentContentChangeEventRangeAndText {
            range: Range {
                start: Position {
                    line: start.line as UInteger,
                    character: start.column as UInteger,
                },
                end: virus_lsp::structures::Position {
                    line: removed_end.line as UInteger,
                    character: removed_end.column as UInteger,
                },
            },
            text: inserted.to_string(),
        }
    }
}
