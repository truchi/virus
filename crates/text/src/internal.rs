use crate::{child::Child, info::Info, Node};
use std::sync::Arc;

#[derive(Clone, Default, Debug)]
pub struct Internal {
    children: [Option<Child>; Self::CAPACITY],
    len: usize,
    info: Info,
}

impl Internal {
    pub const CAPACITY: usize = 4;

    pub fn new() -> Self {
        Self {
            children: [None, None, None, None],
            len: 0,
            info: Info::default(),
        }
    }

    pub fn info(&self) -> Info {
        self.info
    }

    pub fn push(&mut self, child: Child) -> Option<Child> {
        if self.len == Self::CAPACITY {
            Some(child)
        } else {
            self.info += child.info();
            self.children[self.len] = Some(child);
            self.len += 1;

            None
        }
    }

    pub fn children(&self) -> impl Iterator<Item = &Child> {
        let mut i = 0;
        std::iter::from_fn(move || {
            //
            if i < self.len {
                let child = self.children[i].as_ref();
                debug_assert!(child.is_some());
                i += 1;
                child
            } else {
                None
            }
        })
    }
}
