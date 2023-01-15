#![allow(unused)]

mod chunk;
mod cursor;
mod page;
mod selection;
mod text;
mod utils;

pub use cursor::Cursor;
pub use selection::Chunks;
pub use selection::EndMut;
pub use selection::Selection;
pub use selection::StartMut;
pub use text::CursorIndex;
pub use text::SelectionRange;
pub use text::Text;
pub use text::TextRef;

pub type Byte = usize;
pub type Line = usize;
pub type Column = usize;
pub type LineColumn = (usize, usize);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Index {
    byte: usize,
    line: usize,
    column: usize,
}

impl Index {
    pub fn byte(&self) -> Byte {
        self.byte
    }

    pub fn line(&self) -> Line {
        self.line
    }

    pub fn column(&self) -> Column {
        self.column
    }

    pub fn line_column(&self) -> LineColumn {
        (self.line, self.column)
    }
}
