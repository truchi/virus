use crate::Rgb;
use swash::{text::cluster::SourceRange, CacheKey, GlyphId};

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    pub id: GlyphId,
    pub advance: f32,
    pub range: SourceRange,
    pub key: CacheKey,
    pub color: Rgb,
}
