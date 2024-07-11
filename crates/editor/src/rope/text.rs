use ropey::{iter::Chunks, Rope, RopeSlice};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Text                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
enum Inner {
    String(String),
    Rope(Rope),
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Clone, Debug)]
pub struct Text {
    inner: Inner,
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

impl<'a> From<RopeSlice<'a>> for Text {
    fn from(slice: RopeSlice<'a>) -> Self {
        Self {
            inner: if slice.len_bytes() <= 2 * 1024 {
                Inner::String(slice.into())
            } else {
                Inner::Rope(slice.into())
            },
        }
    }
}

impl Text {
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::String(string) => string.len(),
            Inner::Rope(rope) => rope.len_bytes(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn chunks(&self) -> impl Iterator<Item = &str> {
        enum Iter<'a> {
            String(Option<&'a str>),
            Rope(Chunks<'a>),
        }

        let mut iter = match &self.inner {
            Inner::String(string) => Iter::String(Some(&string)),
            Inner::Rope(rope) => Iter::Rope(rope.chunks()),
        };

        std::iter::from_fn(move || match &mut iter {
            Iter::String(string) => string.take(),
            Iter::Rope(chunks) => chunks.next(),
        })
    }
}
