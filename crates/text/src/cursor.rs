use crate::{
    buffer::BufferRef,
    info::Info,
    text::{Text, TextRef},
};
use std::sync::Weak;

#[derive(Copy, Clone, Debug)]
pub struct Cursor<'text> {
    pub text: TextRef<'text>,
    pub buffer: BufferRef<'text>,
    pub offset: Info, // in the text
    pub index: Info,  // in the text
}
