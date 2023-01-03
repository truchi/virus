pub mod cursor;
pub mod line;
pub mod text;

mod unicode;

pub struct Index {
    offset: usize,
    row: usize,
    column: usize,
}

impl Index {
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
