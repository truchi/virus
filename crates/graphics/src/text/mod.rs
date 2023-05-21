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

/// Font size unit (`u8`).
pub type FontSize = u8;

/// LineHeight unit (`u32`).
pub type LineHeight = u32;

/// Advance unit (`f32`).
pub type Advance = f32;
