mod atlases;
mod buffers;
mod constants;
mod init;
mod pipeline;
mod vertices;

use super::*;
use atlases::*;
use buffers::*;
use constants::*;
use init::*;
use vertices::*;

pub use pipeline::TextPipeline;

const MAX_RECTANGLES: usize = 1_000;
const MAX_SHADOWS: usize = 10_000;
const MAX_GLYPHS: usize = 10_000;
const MAX_BLURS: usize = 1_000;
const RECTANGLE_VERTEX_SIZE: usize = size_of::<RectangleVertex>();
const SHADOW_VERTEX_SIZE: usize = size_of::<ShadowVertex>();
const GLYPH_VERTEX_SIZE: usize = size_of::<GlyphVertex>();
const BLUR_VERTEX_SIZE: usize = size_of::<GlyphVertex>();
const INDEX_SIZE: usize = size_of::<Index>();
