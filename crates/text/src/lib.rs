#![allow(unused)]

pub mod buffer;
pub mod builder;
pub mod child;
pub mod info;
pub mod internal;
pub mod text;
pub mod utils;

use buffer::*;
use info::Info;
use internal::*;

use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Node {
    Internal(Internal),
    Leaf(Leaf),
}

impl Node {
    pub fn leaves<T: FnMut(Info, &Leaf)>(&self, mut f: T, info: Info) -> T {
        match self {
            Node::Internal(internal) => {
                for child in internal.children() {
                    f = child.node().leaves(f, child.info());
                }
            }
            Node::Leaf(leaf) => f(info, leaf),
        }

        f
    }
}

#[derive(Clone, Debug)]
pub struct Leaf {
    pub buffer: Buffer,
}

impl Leaf {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }
}
