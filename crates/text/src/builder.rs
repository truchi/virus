use std::sync::Arc;

use crate::{
    buffer::{Buffer, BufferMut},
    child::Child,
    info::Info,
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
    pub fn push_str(&mut self, str: &str) {
        loop {
            let str = unsafe { self.buffer_mut() }.push_str(str);

            if str.is_empty() {
                return;
            } else {
                self.flush_buffer();
            }
        }
    }

    /// Pushes `char` to this [`Builder`]'s [`Text`].
    pub fn push_char(&mut self, char: char) {
        if unsafe { self.buffer_mut() }.push_char(char).is_some() {
            self.flush_buffer();
            unsafe { self.buffer_mut() }.push_char(char);
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
