use std::collections::HashMap;

use swash::{CacheKey, FontRef};

use crate::font::Font;

pub struct Fonts {
    fonts: HashMap<CacheKey, Font>,
    emoji: Font,
}

impl Fonts {
    pub fn new(emoji: Font) -> Self {
        Self {
            fonts: Default::default(),
            emoji,
        }
    }

    pub fn insert(&mut self, font: Font) {
        self.fonts.insert(font.key, font);
    }

    pub fn get(&self, key: CacheKey) -> Option<&Font> {
        if key == self.emoji.key {
            Some(&self.emoji)
        } else {
            self.fonts.get(&key)
        }
    }

    pub fn get_mut(&mut self, key: CacheKey) -> Option<&mut Font> {
        self.fonts.get_mut(&key)
    }

    pub fn emoji(&self) -> FontRef {
        self.emoji.as_ref()
    }
}
