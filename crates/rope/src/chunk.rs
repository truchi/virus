use crate::utils::split_at;
use std::{mem::size_of, sync::atomic::AtomicUsize};

pub const CHUNK_CAPACITY: usize = 1_024 - 2 * size_of::<AtomicUsize>();

pub type Chunk = [u8; CHUNK_CAPACITY];

pub struct ChunkMut<'a, const CAPACITY: usize> {
    chunk: &'a mut [u8; CAPACITY],
    bytes: &'a mut u16,
}

impl<'a, const CAPACITY: usize> ChunkMut<'a, CAPACITY> {
    pub fn len(&self) -> usize {
        *self.bytes as usize
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn push<'str>(&mut self, str: &'str str) -> &'str str {
        let (str, rest) = split_at(str, self.capacity() - self.len());
        let start = self.len();
        let end = start + str.len();

        self.chunk[start..end].copy_from_slice(str.as_bytes());
        *self.bytes += str.len() as u16;

        rest
    }
}
