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
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Chunk                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A chunk of a [`Text`].
#[derive(Copy, Clone, Debug)]
pub struct Chunk<'text> {
    pub str: &'text str,
    pub lines: usize,
    pub index: Index,
}

impl<'text> AsRef<str> for Chunk<'text> {
    fn as_ref(&self) -> &str {
        self.str
    }
}
