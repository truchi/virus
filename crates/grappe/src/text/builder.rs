use crate::page::Page;

pub struct Builder {
    pages: Vec<Page>,
    bytes: usize,
    lines: usize,
}
