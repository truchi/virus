//! Text shaping and scaling.

mod font;
mod glyphs;

pub use font::*;
pub use glyphs::*;

use crate::types::Rgba;
use swash::{scale::ScaleContext, shape::ShapeContext, GlyphId};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontSize                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Font size unit (`u8`).
pub type FontSize = u8;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            LineHeight                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Line height unit (`u32`).
pub type LineHeight = u32;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Advance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Advance unit (`f32`).
pub type Advance = f32;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            GlyphKey                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Glyph`] key.
pub type GlyphKey = (FontKey, FontSize, GlyphId);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Styles                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Glyph`] styles.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Styles {
    pub weight: FontWeight,
    pub style: FontStyle,
    pub foreground: Rgba,
    pub background: Rgba,
    pub underline: bool,
    pub strike: bool,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Glyph                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A glyph.
#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    /// Font key.
    pub font: FontKey,
    /// Font size.
    pub size: FontSize,
    /// Glyph id.
    pub id: GlyphId,
    /// Glyph advance offset.
    pub offset: Advance,
    /// Glyph advance.
    pub advance: Advance,
    /// Start index in the underlying string.
    pub start: u32,
    /// End index in the underlying string.
    pub end: u32,
    /// Glyph styles.
    pub styles: Styles,
}

impl Glyph {
    /// Returns the [`GlyphKey`].
    pub fn key(&self) -> GlyphKey {
        (self.font, self.size, self.id)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Context                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Context for font shaping and scaling.
pub struct Context {
    /// Font cache.
    fonts: Fonts,
    /// Shape context.
    shape: ShapeContext,
    /// Scale context.
    scale: ScaleContext,
}

impl Context {
    /// Creates a new `Context` with `fonts`.
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            shape: Default::default(),
            scale: Default::default(),
        }
    }

    /// Returns the font cache.
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// Returns a tuple of mutable references.
    pub fn as_muts(&mut self) -> (&mut Fonts, &mut ShapeContext, &mut ScaleContext) {
        (&mut self.fonts, &mut self.shape, &mut self.scale)
    }
}
