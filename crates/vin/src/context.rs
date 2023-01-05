use crate::{fonts::Fonts, glyph::Glyph, line::Line, FontSize};
use lru::LruCache;
use std::num::NonZeroUsize;
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    shape::ShapeContext,
    CacheKey, GlyphId,
};

const GLYPH_CACHE_CAPACITY: usize = 1_024;

pub type GlyphCache = LruCache<(CacheKey, GlyphId, FontSize), Option<Image>>;

pub struct Context {
    pub fonts: Fonts,
    pub cache: GlyphCache,
    pub shape: ShapeContext,
    pub scale: ScaleContext,
}

impl Context {
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            cache: GlyphCache::new(NonZeroUsize::new(GLYPH_CACHE_CAPACITY).unwrap()),
            shape: Default::default(),
            scale: Default::default(),
        }
    }

    pub fn scale<'a, F>(&'a mut self, line: &'a Line, mut f: F)
    where
        F: FnMut(Glyph, Option<&Image>) -> bool,
    {
        const HINT: bool = true;
        const SOURCES: &[Source] = &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
            Source::Bitmap(StrikeWith::BestFit),
        ];

        let render = Render::new(SOURCES);

        for glyph in line.glyphs() {
            let font = self.fonts.get(glyph.key).expect("font");
            let scaler = &mut self
                .scale
                .builder(font.as_ref())
                .size(line.size() as f32)
                .hint(HINT)
                .build();

            let image = self
                .cache
                .get_or_insert((glyph.key, glyph.id, line.size()), || {
                    render.render(scaler, glyph.id)
                });

            if !f(*glyph, image.as_ref()) {
                return;
            }
        }
    }
}
