use std::sync::Arc;

use crate::{
    buffer::{Buffer, BufferMut},
    child::Child,
    info::Info,
    text::Text,
    Internal, Leaf, Node,
};

#[derive(Debug)]
pub struct Builder {
    internal: Internal,
    buffer: Buffer,
    info: Info,
}

impl Builder {
    /// Creates a new empty [`Builder`].
    pub fn new() -> Self {
        Self {
            buffer: Buffer::default(),
            internal: Internal::default(),
            info: Info::default(),
        }
    }

    /// Pushes `str` to this [`Builder`]'s [`Text`].
    pub fn push_str(&mut self, mut str: &str) {
        loop {
            let (info, rest) = unsafe { self.buffer_mut() }.push_str(str);
            self.info += info;

            if rest.is_empty() {
                return;
            } else {
                self.flush_buffer();
                str = rest;
            }
        }
    }

    /// Pushes `char` to this [`Builder`]'s [`Text`].
    pub fn push_char(&mut self, char: char) {
        let (info, char) = unsafe { self.buffer_mut() }.push_char(char);
        self.info += info;

        if let Some(char) = char {
            self.flush_buffer();

            let (info, char) = unsafe { self.buffer_mut() }.push_char(char);
            self.info += info;

            debug_assert!(char.is_none());
        }
    }

    pub fn build(mut self) -> Text {
        self.flush_buffer();
        let info = self.internal.info();
        Text {
            node: Arc::new(Node::Internal(self.internal)),
            info,
        }
    }
}

impl Builder {
    unsafe fn buffer_mut(&mut self) -> BufferMut {
        BufferMut::from_buffer(&mut self.buffer, self.info.bytes, self.info.feeds)
    }

    fn flush_buffer(&mut self) {
        let buffer = std::mem::take(&mut self.buffer);
        let info = std::mem::take(&mut self.info);
        let child = Child::new(Arc::new(Node::Leaf(Leaf::new(buffer))), info);

        if let Some(child) = self.internal.push(child) {
            self.flush_internal();
            self.internal.push(child);
        }
    }

    fn flush_internal(&mut self) {
        let internal = std::mem::take(&mut self.internal);
        let info = internal.info();
        let child = Child::new(Arc::new(Node::Internal(internal)), info);

        self.internal.push(child);
    }
}
