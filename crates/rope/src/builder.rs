use crate::{
    buffer::{BufferMut, Bytes, CAPACITY},
    page::Page,
    text::Text,
    utils::split_at,
};
use std::sync::Arc;

/// A [`Text`] builder.
#[derive(Clone, Debug)]
pub struct Builder {
    /// List of pages.
    pages: Vec<Page>,
    /// Raw bytes (next page).
    buffer: Bytes,
    /// Byte count (in `buffer`).
    bytes: u16,
    /// Feed count (in `buffer`).
    feeds: u16,
    /// Byte offset (of `buffer`).
    byte: usize,
    /// Feed offset (of `buffer`).
    feed: usize,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new empty [`Builder`].
    pub fn new() -> Self {
        Self {
            pages: Vec::default(),
            buffer: [0; CAPACITY],
            bytes: 0,
            feeds: 0,
            byte: 0,
            feed: 0,
        }
    }

    /// Pushes `str` to this [`Builder`]'s [`Text`].
    pub fn push_str(&mut self, mut str: &str) {
        loop {
            str = self.buffer_mut().push_str(str);

            if str.is_empty() {
                return;
            } else {
                self.flush();
            }
        }
    }

    /// Pushes `char` to this [`Builder`]'s [`Text`].
    pub fn push_char(&mut self, char: char) {
        if self.buffer_mut().push_char(char).is_some() {
            self.flush();
            self.buffer_mut().push_char(char);
        }
    }

    /// Returns the built [`Text`].
    pub fn build(mut self) -> Text {
        if self.bytes != 0 {
            self.flush();
        }

        Text {
            pages: Arc::new(self.pages),
            bytes: self.byte,
            feeds: self.feed,
        }
    }
}

impl Builder {
    fn buffer_mut(&mut self) -> BufferMut {
        BufferMut {
            buffer: &mut self.buffer,
            bytes: &mut self.bytes,
            feeds: &mut self.feeds,
        }
    }

    fn flush(&mut self) {
        debug_assert!(self.bytes != 0);

        let page = Page {
            buffer: Arc::new(std::mem::replace(&mut self.buffer, [0; CAPACITY])),
            bytes: self.bytes,
            feeds: self.feeds,
            byte: self.byte,
            feed: self.feed,
        };

        self.bytes = 0;
        self.feeds = 0;
        self.byte += page.bytes as usize;
        self.feed += page.feeds as usize;
        self.pages.push(page);
    }
}
