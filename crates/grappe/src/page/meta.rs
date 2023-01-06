use crate::Offset;

#[derive(Copy, Clone, Debug)]
pub struct PageMeta {
    len: u16,
    bytes: u16,
    lines: u16,
    offset: Offset,
}

impl PageMeta {}
