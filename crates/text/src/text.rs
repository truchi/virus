use crate::{buffer::BufferRef, cursor::Cursor, info::Info, utils::count_feeds, Index, Leaf, Node};
use std::{borrow::BorrowMut, sync::Arc};

#[derive(Clone, Debug)]
pub struct Text {
    pub node: Arc<Node>,
    pub info: Info,
}

impl Text {
    pub fn new(node: Arc<Node>, info: Info) -> Self {
        Self { node, info }
    }

    pub fn info(&self) -> Info {
        self.info
    }

    pub fn node(&self) -> &Arc<Node> {
        &self.node
    }

    pub fn leaves<T: FnMut(Info, &Leaf)>(&self, f: T) {
        self.node.leaves(f, self.info);
    }

    pub fn as_ref(&self) -> TextRef {
        TextRef {
            node: self.node.as_ref(),
            info: self.info,
        }
    }

    pub fn cursor<I: CursorIndex>(&self, index: I) -> Cursor {
        index.cursor(self.as_ref())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextRef<'text> {
    node: &'text Node,
    info: Info,
}

/// Cursoring operations.
pub trait CursorIndex {
    /// Returns a [`Cursor`] at this index from a [`TextRef`].
    fn cursor(self, text: TextRef) -> Cursor;
}

impl CursorIndex for usize {
    fn cursor(self, text: TextRef) -> Cursor {
        let TextRef { mut node, mut info } = text;
        let mut global_offset = Info::default();

        loop {
            match node {
                Node::Internal(internal) => {
                    debug_assert!(internal.len() > 0);

                    (global_offset, TextRef { node, info }) = internal
                        .children()
                        .map(|(offset, child)| (global_offset + offset, child))
                        .take_while(|(offset, child)| offset.bytes < self)
                        .last()
                        .map(|(offset, child)| (offset, child.as_ref()))
                        .expect("must have children");
                }
                Node::Leaf(leaf) => {
                    let buffer = unsafe { BufferRef::from_buffer(&leaf.buffer, info) };

                    return Cursor {
                        text,
                        buffer,
                        offset: global_offset,
                        index: Info {
                            bytes: self,
                            feeds: count_feeds(
                                &buffer.as_str()[..info.bytes.min(self - global_offset.bytes)],
                            ),
                        },
                    };
                }
            };
        }
    }
}

impl CursorIndex for (usize, usize) {
    fn cursor(self, text: TextRef) -> Cursor {
        todo!()
    }
}
