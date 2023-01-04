pub mod cursor;
pub mod line;
pub mod text;

mod unicode;

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Index {
    offset: usize,
    row: usize,
    column: usize,
}

impl Index {
    pub fn new(offset: usize, row: usize, column: usize) -> Self {
        Self {
            offset,
            row,
            column,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }
}
