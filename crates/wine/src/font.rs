use std::path::Path;
use swash::{CacheKey, FontRef};

pub struct Font {
    pub data: Vec<u8>,
    pub key: CacheKey,
}

impl Font {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        let font = FontRef::from_index(&data, 0)?;
        let key = font.key;

        Some(Self { data, key })
    }

    pub fn as_ref(&self) -> FontRef {
        FontRef {
            data: &self.data,
            offset: 0,
            key: self.key,
        }
    }
}
