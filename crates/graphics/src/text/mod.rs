//! Text shaping and scaling.
//!
//! ### Areas of improvement
//!
//! [`LineShaper::push`] cannot produce ligatures across calls. There must be a way...
//!
//! It would be nice to crop what we shape/scale of a line on the horizontal axis.
//!
//! Maybe we could carry more style info than an [`Rbg`](virus_common::Rgb) in a [`Glyph`].

mod context;
mod font;
mod line;

pub use context::*;
pub use font::*;
pub use line::*;

use crate::colors::Rgba;
use swash::{text::cluster::SourceRange, GlyphId};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontSize                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Font size unit (`u8`).
pub type FontSize = u8;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            LineHeight                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// LineHeight unit (`u32`).
pub type LineHeight = u32;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Advance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Advance unit (`f32`).
pub type Advance = f32;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Glyph                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A shaped glyph.
#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    /// Glyph id.
    pub id: GlyphId,
    /// Glyph advance offset.
    pub offset: Advance,
    /// Glyph advance.
    pub advance: Advance,
    /// Range in the underlying string.
    pub range: SourceRange,
    /// Glyph styles.
    pub styles: Styles,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Styles                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Glyph styles.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Styles {
    pub font: FontKey,
    pub foreground: Rgba,
    pub background: Rgba,
    pub underline: bool,
    pub strike: bool,
}
