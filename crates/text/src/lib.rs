#![allow(unused)]

pub mod buffer;
pub mod builder;
pub mod child;
pub mod info;
pub mod internal;
pub mod text;
pub mod utils;

use buffer::*;
use internal::*;

use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Node {
    Internal(Internal),
    Leaf(Leaf),
}

#[derive(Clone, Debug)]
pub struct Leaf {
    buffer: Buffer,
}

impl Leaf {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }
}
