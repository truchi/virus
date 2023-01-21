use crate::text::*;
use lru::LruCache;
use std::num::NonZeroUsize;
use swash::{
    scale::{image::Image, ScaleContext},
    shape::ShapeContext,
    CacheKey, GlyphId,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Glyphs                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Glyph cache.
pub type Glyphs = LruCache<(CacheKey, GlyphId, FontSize), Option<Image>>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Context                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Context for font shaping and scaling.
pub struct Context {
    /// Font cache.
    fonts: Fonts,
    /// Glyph cache.
    glyphs: Glyphs,
    /// Shape context.
    shape: ShapeContext,
    /// Scale context.
    scale: ScaleContext,
}

impl Context {
    /// Capacity of the glyph cache.
    const GLYPHS_CAPACITY: usize = 1_024;

    /// Creates a new `Context` with `fonts`.
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            glyphs: Glyphs::new(NonZeroUsize::new(Self::GLYPHS_CAPACITY).unwrap()),
            shape: Default::default(),
            scale: Default::default(),
        }
    }

    /// Returns the font cache.
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// Returns the glyph cache.
    pub fn glyphs(&self) -> &Glyphs {
        &self.glyphs
    }

    /// Returns a tuple of mutable references.
    pub fn as_muts(
        &mut self,
    ) -> (
        &mut Fonts,
        &mut Glyphs,
        &mut ShapeContext,
        &mut ScaleContext,
    ) {
        (
            &mut self.fonts,
            &mut self.glyphs,
            &mut self.shape,
            &mut self.scale,
        )
    }
}
