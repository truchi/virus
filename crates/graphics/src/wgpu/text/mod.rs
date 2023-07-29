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

const VERTICES_PER_QUAD: usize = 4;
const INDICES_PER_QUAD: usize = 6;
