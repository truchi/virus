use crate::text::*;
use lru::LruCache;
use std::{collections::HashMap, num::NonZeroUsize};
use swash::{
    scale::{image::Image, ScaleContext},
    shape::ShapeContext,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Glyphs                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Glyph cache.
pub type Glyphs = LruCache<(FontKey, GlyphId, FontSize), Option<Image>>;

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
    /// Advance cache.
    advances: HashMap<(FontKey, FontSize), Option<Advance>>,
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
            advances: Default::default(),
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

    /// Returns the advance of the character `'a'` for `font` at `size`,
    /// or `None` if `font` does not exists or advance is `0.0`.
    pub fn advance(&mut self, font: FontKey, size: FontSize) -> Option<Advance> {
        const STR: &str = "a";

        if let Some(advance) = self.advances.get(&(font, size)) {
            return *advance;
        }

        let font = self.fonts.get(font)?;

        let mut advance = 0.0;
        let mut shaper = self
            .shape
            .builder(font.as_ref())
            .script(SCRIPT)
            .size(size as f32)
            .features(FEATURES)
            .build();

        shaper.add_str(STR);
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                advance += glyph.advance;
            }
        });

        let advance = if advance == 0.0 { None } else { Some(advance) };
        self.advances.insert((font.key(), size), advance);

        advance
    }
}
