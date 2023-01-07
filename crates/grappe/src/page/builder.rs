use super::Page;
use crate::segment::{meta::SegmentMeta, SegmentBuilderMut};

#[derive(Debug)]
pub struct PageBuilder {
    data: [u8; Page::BYTES],
    offset: usize,
    meta: SegmentMeta,
    bytes: usize,
    lines: usize,
}

pub struct PageBuilderMut<'a> {
    data: &'a mut [u8],
    offset: usize,
    meta: SegmentMeta,
    len: usize,
    sol: usize,
    bytes: usize,
    lines: usize,
}

impl<'a> PageBuilderMut<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        debug_assert!(data.as_mut().len() >= SegmentMeta::SIZE);

        Self {
            data,
            offset: 0,
            meta: Default::default(),
            len: 0,
            sol: 0,
            bytes: 0,
            lines: 0,
        }
    }

    pub fn push_str<'str>(&mut self, mut str: &'str str) -> &'str str {
        loop {
            let segment = SegmentBuilderMut::new(&mut self.data.as_mut()[self.offset..], self.meta);

            if let Some(mut segment) = segment {
                str = segment.push_str(str);

                if !str.is_empty() {
                    segment.write();

                    if segment.meta().eol.is_some() {
                        self.lines += 1;

                        if self.sol == 0 {
                            self.sol = self.offset;
                        }
                    }

                    self.offset += segment.meta().size();
                    self.len += segment.meta().size();
                    self.bytes += segment.meta().len();

                    self.meta = Default::default();
                } else {
                    self.meta = segment.meta();
                    return str;
                }
            } else {
                return str;
            };
        }
    }

    pub fn finish(&mut self) {
        if let Some(mut segment) =
            SegmentBuilderMut::new(&mut self.data.as_mut()[self.offset..], self.meta)
        {
            segment.write();

            if segment.meta().eol.is_some() {
                self.lines += 1;

                if self.sol == 0 {
                    self.sol = self.offset;
                }
            }

            self.offset += segment.meta().size();
            self.len += segment.meta().size();
            self.bytes += segment.meta().len();

            self.meta = Default::default();
        }
    }
}
