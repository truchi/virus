pub mod meta;

use meta::SegmentMeta;

use crate::eol::Eol;

const SPACES: &str = "                                                               ";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            SegmentRef                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct SegmentRef<'a> {
    str: &'a str,
    spaces: u8,
    eol: Option<Eol>,
}

impl<'a> SegmentRef<'a> {
    pub fn len(&self) -> usize {
        self.spaces as usize
            + self.str.len()
            + self.eol.map(|eol| eol.as_str().len()).unwrap_or_default()
    }

    pub fn spaces(&self) -> &'static str {
        &SPACES[..self.spaces as usize]
    }

    pub fn str(&self) -> &'a str {
        self.str
    }

    pub fn eol(&self) -> &'static str {
        self.eol.map(|eol| eol.as_str()).unwrap_or_default()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            SegmentMut                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct SegmentBuilderMut<'a> {
    data: &'a mut [u8],
    meta: SegmentMeta,
}

impl<'a> SegmentBuilderMut<'a> {
    pub fn new(data: &'a mut [u8], meta: SegmentMeta) -> Option<Self> {
        if data.len() >= SegmentMeta::SIZE {
            Some(Self { data, meta })
        } else {
            None
        }
    }

    pub fn new_unchecked(data: &'a mut [u8], meta: SegmentMeta) -> Self {
        debug_assert!(data.len() >= SegmentMeta::SIZE);

        Self { data, meta }
    }

    pub fn meta(&self) -> SegmentMeta {
        self.meta
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
        self.meta.encode(self.data);
    }
}

impl<'a> SegmentBuilderMut<'a> {
    fn available(&self) -> usize {
        (self.data.len() - SegmentMeta::SIZE - self.meta.len as usize)
            .min(SegmentMeta::MAX_LEN as usize)
    }

    fn consume_spaces<'str>(&mut self, str: &'str str) -> &'str str {
        let spaces = self.meta.spaces;
        let bytes = str.as_bytes();
        let bytes = bytes
            .get(..(SegmentMeta::MAX_SPACES - spaces) as usize)
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
        let start = SegmentMeta::PREFIX_SIZE + self.meta.len as usize;
        let end = start + str.len();

        debug_assert!(!str.contains('\r'));
        debug_assert!(!str.contains('\n'));
        debug_assert!(end + SegmentMeta::SUFFIX_SIZE <= self.data.len());
        debug_assert!(self.meta.len as usize + str.len() <= SegmentMeta::MAX_LEN as usize);

        self.data[start..end].copy_from_slice(str.as_bytes());
        self.meta.len += str.len() as u8;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Tests                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces() {
        assert!(SPACES.len() == 63)
    }

    #[test]
    fn segment_mut() {
        let mut data = [0; 1_000];
        let mut segment_mut = SegmentBuilderMut::new(&mut data, Default::default()).unwrap();

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
        assert!(
            from_utf8(&data[SegmentMeta::PREFIX_SIZE..SegmentMeta::PREFIX_SIZE + 9])
                == Ok("abc   def")
        );
    }
}
