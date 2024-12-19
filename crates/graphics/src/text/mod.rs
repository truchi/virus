//! Text shaping and scaling.

mod font;
mod glyphs;

pub use font::*;
pub use glyphs::*;

use crate::types::Rgba;
use swash::GlyphId;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontSize                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Font size unit (`u8`).
pub type FontSize = u8;

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
    pub offset: f32,
    /// Glyph advance.
    pub advance: f32,
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
