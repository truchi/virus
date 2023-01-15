pub const unsafe fn utf8(bytes: &[u8]) -> &str {
    debug_assert!(std::str::from_utf8(bytes).is_ok());
    std::str::from_utf8_unchecked(bytes)
}

pub unsafe fn utf8_mut(bytes: &mut [u8]) -> &mut str {
    debug_assert!(std::str::from_utf8(bytes).is_ok());
    std::str::from_utf8_unchecked_mut(bytes)
}

pub fn split_at(str: &str, mut index: usize) -> (&str, &str) {
    while !str.is_char_boundary(index) {
        index -= 1;
    }

    str.split_at(index)
}
