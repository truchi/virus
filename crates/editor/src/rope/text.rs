use crate::rope::{Grapheme, GraphemeCategory, GraphemesBackward};
use ropey::{Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Inner                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Eq, Debug)]
pub(super) enum Inner {
    String(String), // TODO SmolStr?
    Rope(Rope),
}

impl PartialEq for Inner {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a == b,
            (Self::String(a), Self::Rope(b)) => a == b,
            (Self::Rope(a), Self::String(b)) => a == b,
            (Self::Rope(a), Self::Rope(b)) => a == b,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Text                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Eq, PartialEq)]
pub struct Text {
    pub(super) inner: Inner,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            inner: Inner::String(String::new()),
        }
    }
}

impl<'a> From<&'a str> for Text {
    fn from(str: &'a str) -> Self {
        Self {
            inner: Inner::String(str.into()),
        }
    }
}

impl<'a> From<String> for Text {
    fn from(string: String) -> Self {
        Self {
            inner: Inner::String(string),
        }
    }
}

impl<'a> From<RopeSlice<'a>> for Text {
    fn from(slice: RopeSlice<'a>) -> Self {
        Self {
            inner: if slice.len_bytes() <= Self::BREAKPOINT {
                Inner::String(slice.into())
            } else {
                Inner::Rope(slice.into())
            },
        }
    }
}

impl<'a> From<Rope> for Text {
    fn from(rope: Rope) -> Self {
        Self {
            inner: if rope.len_bytes() <= Self::BREAKPOINT {
                Inner::String(rope.into())
            } else {
                Inner::Rope(rope)
            },
        }
    }
}

impl ToString for Text {
    fn to_string(&self) -> String {
        match &self.inner {
            Inner::String(string) => string.to_string(),
            Inner::Rope(rope) => rope.to_string(),
        }
    }
}

impl Text {
    /// Ropes greater than this breakpoint will be stored as is.
    /// Other ropes will be converted into strings.
    ///
    /// @see [`Rope::try_insert()`] comments.
    pub const BREAKPOINT: usize = 6 * 984; // 6 * ropey::MAX_BYTES

    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::String(string) => string.len(),
            Inner::Rope(rope) => rope.len_bytes(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn trailing_line_break(&mut self, bool: bool) {
        let trailing_line_break_len = match &self.inner {
            Inner::String(string) => string
                .as_str()
                .graphemes(true)
                .rev()
                .next()
                .map(|grapheme| Grapheme::Str(grapheme)),
            Inner::Rope(rope) => GraphemesBackward::new(rope.slice(..)).next(),
        }
        .map(|grapheme| (GraphemeCategory::from(grapheme.as_str()), grapheme));

        if let Some((GraphemeCategory::Break, grapheme)) = trailing_line_break_len {
            if !bool {
                let bytes = grapheme.as_str().len();
                let chars = grapheme.as_str().chars().count();

                match &mut self.inner {
                    Inner::String(string) => string.truncate(string.len() - bytes),
                    Inner::Rope(rope) => rope.remove(rope.len_chars() - chars..rope.len_chars()),
                }
            }
        } else {
            if bool {
                match &mut self.inner {
                    Inner::String(string) => string.push_str("\n"),
                    Inner::Rope(rope) => rope.insert(rope.len_chars(), "\n"),
                }
            }
        }
    }
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\"",
            match &self.inner {
                Inner::String(string) => string.clone(),
                Inner::Rope(rope) => rope.to_string(),
            },
        )
    }
}
