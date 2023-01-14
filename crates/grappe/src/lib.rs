#![allow(unused)]

pub mod eol;
pub mod page;
pub mod segment;
pub mod text;

pub mod builder;

// meta:
// - leading spaces: 6b 0..64
// - byte length   : 8b 0..255
// - end of line   : 2b LF,CR,CRLF,None
//
// prefix, |aaaaaaaa|bbcccccc|:
// - a: len
// - b: eol
// - c: spaces
//
// suffix, |aaaaaaaa|:
// - a: len
//
// encoding:
// [ ( prefix+bytes+suffix )* ]

const unsafe fn utf8(v: &[u8]) -> &str {
    debug_assert!(std::str::from_utf8(v).is_ok());
    std::str::from_utf8_unchecked(v)
}

fn split(str: &str, mut at: usize) -> (&str, &str) {
    if at >= str.len() {
        (str, "")
    } else {
        while !str.is_char_boundary(at) {
            at -= 1;
        }

        str.split_at(at)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Offset {
    bytes: usize,
    lines: usize,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Index {
    byte: usize,
    line: usize,
    column: usize,
}

pub enum State<T, U> {
    Incomplete(T),
    Complete(U),
}
