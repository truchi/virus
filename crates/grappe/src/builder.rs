use std::sync::Arc;

use crate::{eol::Eol, page::Page, segment::meta::SegmentMeta, text::Text};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            TextBuilder                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct TextBuilder {
    pages: Vec<Page>,
    page: PageBuilder,
    bytes: usize,
    lines: usize,
}

impl TextBuilder {
    pub fn new() -> Self {
        Self {
            pages: Default::default(),
            page: PageBuilder::new(),
            bytes: 0,
            lines: 0,
        }
    }

    pub fn push_str(&mut self, mut str: &str) {
        loop {
            str = self.page.push_str(str);

            if str.is_empty() {
                return;
            } else {
                let page = self.page.finish();

                self.bytes += page.bytes as usize;
                self.lines += page.bytes as usize;
                self.pages.push(page);
            }
        }
    }

    pub fn finish(mut self) -> Text {
        if !self.page.is_empty() {
            let page = self.page.finish();

            self.bytes += page.bytes as usize;
            self.lines += page.bytes as usize;
            self.pages.push(page);
        }

        Text {
            pages: Arc::new(self.pages),
            bytes: self.bytes,
            lines: self.lines,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            PageBuilder                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct PageBuilder {
    data: [u8; Page::LEN],
    offset: usize,
    meta: SegmentMeta,
    len: u16,
    sol: u16,
    bytes: u16,
    lines: u16,
}

impl PageBuilder {
    fn new() -> Self {
        Self {
            data: [0; Page::LEN],
            offset: 0,
            meta: Default::default(),
            len: 0,
            sol: 0,
            bytes: 0,
            lines: 0,
        }
    }

    fn len(&self) -> u16 {
        self.len + self.meta.len() as u16
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn push_str<'str>(&mut self, mut str: &'str str) -> &'str str {
        loop {
            let segment = SegmentBuilder::new(&mut self.data.as_mut()[self.offset..], self.meta);

            if let Some(mut segment) = segment {
                str = segment.push_str(str);

                if str.is_empty() {
                    self.meta = segment.meta;
                    return str;
                } else {
                    segment.finish();

                    self.offset += segment.meta.size();
                    self.len += segment.meta.size() as u16;
                    self.bytes += segment.meta.len() as u16;

                    if segment.meta.eol.is_some() {
                        self.lines += 1;

                        if self.sol == 0 {
                            self.sol = self.offset as u16;
                        }
                    }

                    self.meta = Default::default();
                }
            } else {
                return str;
            }
        }
    }

    fn finish(&mut self) -> Page {
        if let Some(mut segment) =
            SegmentBuilder::new(&mut self.data.as_mut()[self.offset..], self.meta)
        {
            segment.finish();

            if segment.meta.eol.is_some() {
                self.lines += 1;

                if self.sol == 0 {
                    self.sol = self.offset as u16;
                }
            }

            self.offset += segment.meta.size();
            self.len += segment.meta.size() as u16;
            self.bytes += segment.meta.len() as u16;

            self.meta = Default::default();
        }

        let page = Page {
            data: Arc::new(self.data),
            len: self.len,
            sol: self.sol,
            bytes: self.bytes,
            lines: self.lines,
            offset_bytes: 0,
            offset_lines: 0,
        };

        *self = Self::new();

        page
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          SegmentBuilder                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct SegmentBuilder<'a> {
    data: &'a mut [u8],
    meta: SegmentMeta,
}

impl<'a> SegmentBuilder<'a> {
    fn new(data: &'a mut [u8], meta: SegmentMeta) -> Option<Self> {
        if data.len() >= SegmentMeta::SIZE {
            Some(Self { data, meta })
        } else {
            None
        }
    }

    fn push_str<'str>(&mut self, mut str: &'str str) -> &'str str {
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

        str
    }

    fn finish(&mut self) {
        self.meta.encode(self.data);
    }

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
    fn segment_builder() {
        let mut data = [0; 1_000];
        let mut segment_mut = SegmentBuilder::new(&mut data, Default::default()).unwrap();

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
