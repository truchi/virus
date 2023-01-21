#![allow(unused)]

mod buffer;
mod builder;
mod cursor;
mod page;
mod selection;
mod text;
mod utils;

pub use builder::Builder;
pub use cursor::Cursor;
pub use cursor::CursorMut;
pub use selection::Chunks;
pub use selection::Selection;
pub use text::CursorIndex;
pub use text::SelectionRange;
pub use text::Text;
pub use text::TextRef;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Index                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An index in a [`Text`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Index {
    /// Byte offset.
    pub byte: usize,
    /// Line offset.
    pub line: usize,
    /// Column offset.
    pub column: usize,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Chunk                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A chunk of a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct Chunk<'text> {
    /// Content of this [`Chunk`].
    pub str: &'text str,
    /// Feed (`'\n'`) count.
    pub feeds: usize,
    /// Byte offset.
    pub byte: usize,
    /// Line offset.
    pub line: usize,
}

impl<'text> Chunk<'text> {
    /// Returns the count of lines in this [`Chunk`] (`self.feeds + 1`).
    pub fn lines(&self) -> usize {
        self.feeds + 1
    }

    /// Returns the content of this [`Chunk`].
    fn as_str(&self) -> &str {
        self.str
    }
}

impl<'text> AsRef<str> for Chunk<'text> {
    fn as_ref(&self) -> &str {
        self.str
    }
}
