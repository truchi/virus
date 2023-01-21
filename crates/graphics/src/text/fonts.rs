use crate::text::*;
use std::collections::HashMap;
use swash::{CacheKey, FontRef};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Fonts                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Font`] collection.
///
/// Contains multiple [`Font`]s and an emoji fallback.
pub struct Fonts {
    /// Fonts in the collection.
    fonts: HashMap<CacheKey, Font>,
    /// The emoji fallback font.
    emoji: Font,
}

impl Fonts {
    /// Returns a new `Fonts` from `fonts` and `emoji` fallback.
    pub fn new<I: IntoIterator<Item = Font>>(fonts: I, emoji: Font) -> Self {
        Self {
            fonts: fonts.into_iter().map(|font| (font.key(), font)).collect(),
            emoji,
        }
    }

    /// Returns a `FontRef` from a `key` (including the emoji font).
    pub fn get(&self, key: CacheKey) -> Option<FontRef> {
        if key == self.emoji.key() {
            Some(self.emoji.as_ref())
        } else {
            self.fonts.get(&key).map(|font| font.as_ref())
        }
    }

    /// Returns a `FontRef` to the emoji font.
    pub fn emoji(&self) -> FontRef {
        self.emoji.as_ref()
    }
}
