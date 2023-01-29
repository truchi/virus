use crate::{info::Info, text::Text, Node};
use std::sync::Arc;

#[derive(Clone, Default, Debug)]
pub struct Internal {
    pub children: [Option<Text>; Self::CAPACITY],
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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn info(&self) -> Info {
        self.info
    }

    pub fn push(&mut self, child: Text) -> Option<Text> {
        if self.len == Self::CAPACITY {
            Some(child)
        } else {
            self.info += child.info();
            self.children[self.len] = Some(child);
            self.len += 1;

            None
        }
    }

    pub fn children(&self) -> impl Iterator<Item = (Info, &Text)> {
        let mut iter = self.children[..self.len].iter();
        let mut offset = Info::default();

        std::iter::from_fn(move || {
            let offset_ = offset;
            let child = iter.next()?.as_ref();
            debug_assert!(child.is_some());

            if let Some(child) = child {
                offset += child.info;
                Some((offset_, child))
            } else {
                debug_assert!(false);
                None
            }
        })

        // let mut i = 0;
        // std::iter::from_fn(move || {
        //     if i < self.len {
        //         let child = self.children[i].as_ref();
        //         debug_assert!(child.is_some());
        //         i += 1;
        //         child
        //     } else {
        //         None
        //     }
        // })
    }
}
