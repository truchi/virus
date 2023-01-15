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
    text: Vec<Page>,
    text_bytes: usize,
    text_lines: usize,
    buffer: Bytes,
    buffer_bytes: u16,
    buffer_lines: u16,
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
            text: Vec::default(),
            text_bytes: 0,
            text_lines: 0,
            buffer: [0; CAPACITY],
            buffer_bytes: 0,
            buffer_lines: 0,
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
        if self.buffer_bytes != 0 {
            self.flush();
        }

        Text {
            pages: Arc::new(self.text),
            bytes: self.text_bytes,
            lines: self.text_lines,
        }
    }
}

impl Builder {
    fn buffer_mut(&mut self) -> BufferMut {
        BufferMut {
            buffer: &mut self.buffer,
            bytes: &mut self.buffer_bytes,
            lines: &mut self.buffer_lines,
        }
    }

    fn flush(&mut self) {
        debug_assert!(self.buffer_bytes != 0);

        let page = Page {
            buffer: Arc::new(std::mem::replace(&mut self.buffer, [0; CAPACITY])),
            bytes: self.buffer_bytes,
            lines: self.buffer_lines,
            byte: self.text_bytes,
            line: self.text_lines,
        };

        self.buffer_bytes = 0;
        self.buffer_lines = 0;
        self.text_bytes += page.bytes as usize;
        self.text_lines += page.lines as usize;
        self.text.push(page);
    }
}
