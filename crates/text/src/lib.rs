#![allow(unused)]

pub mod buffer;
pub mod builder;
pub mod cursor;
pub mod info;
pub mod internal;
pub mod text;
pub mod utils;

use buffer::*;
use info::Info;
use internal::*;
use std::sync::Arc;

/// An index in a [`Text`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Index {
    /// Byte offset.
    pub byte: usize,
    /// Line offset.
    pub line: usize,
    /// Column offset.
    pub column: usize,
}

#[derive(Clone, Debug)]
pub enum Node {
    Internal(Internal),
    Leaf(Leaf),
}

impl Node {
    pub fn leaves<T: FnMut(Info, &Leaf)>(&self, mut f: T, info: Info) -> T {
        match self {
            Node::Internal(internal) => {
                for (_, child) in internal.children() {
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
