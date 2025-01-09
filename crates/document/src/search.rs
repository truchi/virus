use ropey::RopeSlice;

/// Returns the end of the range used to find `needle` in `haystack`.
fn end(needle_len: usize, haystack_len: usize) -> usize {
    (haystack_len + 1)
        .checked_sub(needle_len)
        .unwrap_or_default()
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         SearchForward                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Iterator of forward matches of a needle rope in an haystack rope.
pub struct SearchForward<'needle, 'haystack> {
    needle: RopeSlice<'needle>,
    haystack: RopeSlice<'haystack>,
    start: usize,
    end: usize,
}

impl<'needle, 'haystack> SearchForward<'needle, 'haystack> {
    pub fn new(needle: RopeSlice<'needle>, haystack: RopeSlice<'haystack>) -> Self {
        Self {
            needle,
            haystack,
            start: 0,
            end: (needle.len_bytes() != 0)
                .then(|| end(needle.len_bytes(), haystack.len_bytes()))
                .unwrap_or_default(),
        }
    }
}

impl<'needle, 'haystack> Iterator for SearchForward<'needle, 'haystack> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for index in self.start..self.end {
            let Some(haystack) = self
                .haystack
                .get_byte_slice(index..index + self.needle.len_bytes())
            else {
                continue;
            };

            if haystack == self.needle {
                self.start = index + self.needle.len_bytes();

                debug_assert_eq!(
                    self.needle,
                    self.haystack
                        .byte_slice(index..index + self.needle.len_bytes()),
                );
                return Some(index);
            }
        }

        None
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         SearchBackward                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Iterator of backward matches of a needle rope in an haystack rope.
pub struct SearchBackward<'needle, 'haystack> {
    needle: RopeSlice<'needle>,
    haystack: RopeSlice<'haystack>,
    start: usize,
    end: usize,
}

impl<'needle, 'haystack> SearchBackward<'needle, 'haystack> {
    pub fn new(needle: RopeSlice<'needle>, haystack: RopeSlice<'haystack>) -> Self {
        Self {
            needle,
            haystack,
            start: 0,
            end: (needle.len_bytes() != 0)
                .then(|| end(needle.len_bytes(), haystack.len_bytes()))
                .unwrap_or_default(),
        }
    }
}

impl<'needle, 'haystack> Iterator for SearchBackward<'needle, 'haystack> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for index in (self.start..self.end).rev() {
            let Some(haystack) = self
                .haystack
                .get_byte_slice(index..index + self.needle.len_bytes())
            else {
                continue;
            };

            if haystack == self.needle {
                self.end = end(self.needle.len_bytes(), index);

                debug_assert_eq!(
                    self.needle,
                    self.haystack
                        .byte_slice(index..index + self.needle.len_bytes()),
                );
                return Some(index);
            }
        }

        None
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn forward() {
        for (needle, haystack, expected) in [
            ("", "", vec![]),
            ("", "?", vec![]),
            ("?", "", vec![]),
            ("???", "?", vec![]),
            ("a", "abc", vec![0]),
            ("b", "abc", vec![1]),
            ("c", "abc", vec![2]),
            ("ab", "abcabcabc", vec![0, 3, 6]),
            ("ca", "abcabcabc", vec![2, 5]),
            ("abcabcabc", "abcabcabc", vec![0]),
            ("a", "a🦀a🦀a", vec![0, 5, 10]),
        ] {
            assert_eq!(
                SearchForward::new(
                    Rope::from(needle).byte_slice(..),
                    Rope::from(haystack).byte_slice(..),
                )
                .collect::<Vec<_>>(),
                expected,
                "needle: {needle}, haystack: {haystack}",
            );
        }
    }

    #[test]
    fn backward() {
        for (needle, haystack, expected) in [
            ("", "", vec![]),
            ("", "?", vec![]),
            ("?", "", vec![]),
            ("???", "?", vec![]),
            ("a", "abc", vec![0]),
            ("b", "abc", vec![1]),
            ("c", "abc", vec![2]),
            ("ab", "abcabcabc", vec![6, 3, 0]),
            ("ca", "abcabcabc", vec![5, 2]),
            ("abcabcabc", "abcabcabc", vec![0]),
            ("a", "a🦀a🦀a", vec![10, 5, 0]),
        ] {
            assert_eq!(
                SearchBackward::new(
                    Rope::from(needle).byte_slice(..),
                    Rope::from(haystack).byte_slice(..),
                )
                .collect::<Vec<_>>(),
                expected,
                "needle: {needle}, haystack: {haystack}",
            );
        }
    }
}
