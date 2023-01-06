#[derive(Copy, Clone, Default, Debug)]
pub struct Cursor {
    index: usize,
    line: usize,
    column: usize,
}

impl Cursor {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}
