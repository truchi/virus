use crate::eol::Eol;
use crate::meta::Meta;

#[derive(Debug)]
pub struct PageBuilder<T: AsMut<[u8]>> {
    buffer: T,
    offset: usize,
    meta: Meta,
}

impl<T: AsMut<[u8]>> PageBuilder<T> {
    pub fn new(mut buffer: T) -> Self {
        debug_assert!(buffer.as_mut().len() >= Meta::SIZE);

        Self {
            buffer,
            offset: 0,
            meta: Default::default(),
        }
    }

    pub fn push_str<'str>(&mut self, mut str: &'str str) -> &'str str {
        loop {
            let segment = SegmentMut::new(&mut self.buffer.as_mut()[self.offset..], self.meta);

            if let Some(mut segment) = segment {
                str = segment.push_str(str);

                if !str.is_empty() {
                    segment.write();
                    self.offset += segment.size();
                    self.meta = Default::default();
                } else {
                    self.meta = segment.meta;
                    return str;
                }
            } else {
                return str;
            };
        }
    }

    pub fn finish(mut self) -> T {
        if let Some(mut segment) =
            SegmentMut::new(&mut self.buffer.as_mut()[self.offset..], self.meta)
        {
            segment.write();
        }

        self.buffer
    }
}

pub struct SegmentMut<'a> {
    buffer: &'a mut [u8],
    meta: Meta,
}

impl<'a> SegmentMut<'a> {
    pub fn new(buffer: &'a mut [u8], meta: Meta) -> Option<Self> {
        if buffer.len() >= Meta::SIZE {
            Some(Self { buffer, meta })
        } else {
            None
        }
    }

    pub fn new_unchecked(buffer: &'a mut [u8], meta: Meta) -> Self {
        debug_assert!(buffer.len() >= Meta::SIZE);

        Self { buffer, meta }
    }

    pub fn size(&self) -> usize {
        Meta::SIZE + self.meta.len as usize
    }

    pub fn push_str<'str>(&mut self, mut str: &'str str) -> &'str str {
        // If there is only spaces, try to push more spaces
        if self.meta.len == 0 && self.meta.eol.is_none() {
            str = self.consume_spaces(str);
        }

        // If there is available bytes, push until potential eol
        let available = self.available();
        if available > 0 && self.meta.eol.is_none() {
            let (before, eol, after) = Eol::split_before(str, available);
            self.copy(before);
            self.meta.eol = eol;
            str = after;
        }

        // If there is an eol, try to merge
        if self.meta.eol == Some(Eol::Cr) {
            if let Some((Eol::Lf, after)) = Eol::leading(str) {
                self.meta.eol = Some(Eol::Crlf);
                str = after;
            }
        }

        // Return the rest of the str
        str
    }

    pub fn write(&mut self) {
        self.meta.encode(self.buffer);
    }
}

impl<'a> SegmentMut<'a> {
    fn available(&self) -> usize {
        (self.buffer.len() - Meta::SIZE - self.meta.len as usize).min(Meta::MAX_LEN as usize)
    }

    fn consume_spaces<'str>(&mut self, str: &'str str) -> &'str str {
        let spaces = self.meta.spaces;
        let bytes = str.as_bytes();
        let bytes = bytes
            .get(..(Meta::MAX_SPACES - spaces) as usize)
            .unwrap_or(bytes);

        for byte in bytes {
            if *byte == b' ' {
                self.meta.spaces += 1;
            } else {
                break;
            }
        }

        let spaces = self.meta.spaces - spaces;
        &str[spaces as usize..]
    }

    fn copy(&mut self, str: &str) {
        let start = Meta::PREFIX_SIZE + self.meta.len as usize;
        let end = start + str.len();

        debug_assert!(!str.contains('\r'));
        debug_assert!(!str.contains('\n'));
        debug_assert!(end + Meta::SUFFIX_SIZE <= self.buffer.len());
        debug_assert!(self.meta.len as usize + str.len() <= Meta::MAX_LEN as usize);

        self.buffer[start..end].copy_from_slice(str.as_bytes());
        self.meta.len += str.len() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_mut() {
        let mut buffer = [0; 1_000];
        let mut segment_mut = SegmentMut::new(&mut buffer, Default::default()).unwrap();

        assert!(segment_mut.push_str("   ").is_empty());
        assert!(segment_mut.meta.spaces == 3);
        assert!(segment_mut.meta.len == 0);
        assert!(segment_mut.meta.eol == None);

        assert!(segment_mut.push_str("   abc").is_empty());
        assert!(segment_mut.meta.spaces == 6);
        assert!(segment_mut.meta.len == 3);
        assert!(segment_mut.meta.eol == None);

        assert!(segment_mut.push_str("   def\r").is_empty());
        assert!(segment_mut.meta.spaces == 6);
        assert!(segment_mut.meta.len == 9);
        assert!(segment_mut.meta.eol == Some(Eol::Cr));

        assert!(segment_mut.push_str("\nxyz") == "xyz");
        assert!(segment_mut.meta.spaces == 6);
        assert!(segment_mut.meta.len == 9);
        assert!(segment_mut.meta.eol == Some(Eol::Crlf));

        use std::str::from_utf8;
        assert!(from_utf8(&buffer[Meta::PREFIX_SIZE..Meta::PREFIX_SIZE + 9]) == Ok("abc   def"));
    }
}
