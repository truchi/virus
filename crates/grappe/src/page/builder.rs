use crate::segment::{meta::SegmentMeta, SegmentMut};

#[derive(Debug)]
pub struct PageBuilder<T: AsMut<[u8]>> {
    buffer: T,
    offset: usize,
    meta: SegmentMeta,
}

impl<T: AsMut<[u8]>> PageBuilder<T> {
    pub fn new(mut buffer: T) -> Self {
        debug_assert!(buffer.as_mut().len() >= SegmentMeta::SIZE);

        Self {
            buffer,
            offset: 0,
            meta: Default::default(),
        }
    }

    pub fn push_str<'str>(&mut self, mut str: &'str str) -> &'str str {
        loop {
            if self.offset + SegmentMeta::SIZE > self.buffer.as_mut().len() {
                return str;
            }

            let mut segment = SegmentMut::new(&mut self.buffer.as_mut()[self.offset..], self.meta);
            str = segment.push_str(str);

            if !str.is_empty() {
                segment.write();
                self.offset += segment.size();
                self.meta = Default::default();
            } else {
                self.meta = segment.meta();
                return str;
            }
        }
    }

    pub fn finish(mut self) -> T {
        if self.offset + SegmentMeta::SIZE <= self.buffer.as_mut().len() {
            let mut segment = SegmentMut::new(&mut self.buffer.as_mut()[self.offset..], self.meta);
            segment.write();
        }

        self.buffer
    }
}
