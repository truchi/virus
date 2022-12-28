use crate::{Buffer, Rgb};
use std::{collections::HashMap, path::Path};
use swash::{
    scale::{image::Image, Render, ScaleContext, Scaler, Source, StrikeWith},
    shape::{ShapeContext, Shaper},
    text::{
        cluster::{CharCluster, Parser, SourceRange, Status, Token},
        Script,
    },
    CacheKey, FontRef, GlyphId,
};
use utils::lru::Lru;

pub type FontSize = u8;

pub struct Fonts {
    fonts: HashMap<CacheKey, Font>,
    emoji: Font,
}

impl Fonts {
    pub fn new(emoji: Font) -> Self {
        Self {
            fonts: Default::default(),
            emoji,
        }
    }

    pub fn insert(&mut self, font: Font) {
        self.fonts.insert(font.key, font);
    }

    pub fn get(&self, key: CacheKey) -> Option<&Font> {
        self.fonts.get(&key)
    }

    pub fn get_mut(&mut self, key: CacheKey) -> Option<&mut Font> {
        self.fonts.get_mut(&key)
    }

    pub fn emoji(&self) -> FontRef {
        self.emoji.as_ref()
    }
}

pub struct Font {
    pub data: Vec<u8>,
    pub key: CacheKey,
}

impl Font {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        let font = FontRef::from_index(&data, 0)?;
        let key = font.key;

        Some(Self { data, key })
    }

    pub fn as_ref(&self) -> FontRef {
        FontRef {
            data: &self.data,
            offset: 0,
            key: self.key,
        }
    }
}

pub struct Context {
    fonts: Fonts,
    cache: Lru<(CacheKey, GlyphId, FontSize), Option<Image>>,
    shape: ShapeContext,
    scale: ScaleContext,
}

impl Context {
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            cache: Lru::new(1_024),
            shape: Default::default(),
            scale: Default::default(),
        }
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn scale<'a, F>(&'a mut self, line: &'a Line, mut f: F)
    where
        F: FnMut(Glyph, Option<&Image>) -> bool,
    {
        const HINT: bool = false;
        const SOURCES: &[Source] = &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
            Source::Bitmap(StrikeWith::BestFit),
        ];

        let key = line.key;
        let size = line.size;
        let font = self.fonts.get(key).expect("font");
        let render = Render::new(SOURCES);
        let scaler = &mut self
            .scale
            .builder(font.as_ref())
            .size(size as f32)
            .hint(HINT)
            .build();

        for glyph in line.glyphs() {
            let image = self.cache.get_or_set((font.key, glyph.id, size), || {
                render.render(scaler, glyph.id)
            });

            if !f(*glyph, image.as_ref()) {
                return;
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    pub id: GlyphId,
    pub advance: f32,
    pub range: SourceRange,
    pub key: CacheKey,
    pub color: Rgb,
}

pub struct Line {
    glyphs: Vec<Glyph>,
    key: CacheKey,
    size: FontSize,
}

impl Line {
    pub fn from_iter<'a, I>(context: &mut Context, iter: I, key: CacheKey, size: FontSize) -> Self
    where
        I: IntoIterator<Item = (Rgb, &'a str)>,
    {
        const SCRIPT: Script = Script::Latin;
        const FEATURES: &[(&str, u16)] = &[("dlig", 1), ("calt", 1)];

        #[derive(Copy, Clone, Debug)]
        enum FontOrEmoji {
            Font,
            Emoji,
        }

        fn token(offset: u32) -> impl Clone + Fn((usize, char)) -> Token {
            move |(i, ch)| Token {
                ch,
                offset: offset + i as u32,
                len: ch.len_utf8() as u8,
                info: ch.into(),
                data: 0,
            }
        }

        macro_rules! build {
            ($context:ident, $font:ident, $size:ident) => {
                $context
                    .shape
                    .builder($font)
                    .script(SCRIPT)
                    .size($size as f32)
                    .features(FEATURES)
                    .build()
            };
        }

        fn select(font: FontRef, emoji: FontRef, cluster: &mut CharCluster) -> FontOrEmoji {
            match cluster.map(|ch| font.charmap().map(ch)) {
                Status::Discard => FontOrEmoji::Emoji,
                Status::Complete => FontOrEmoji::Font,
                Status::Keep => match cluster.map(|ch| emoji.charmap().map(ch)) {
                    Status::Discard => FontOrEmoji::Font,
                    Status::Complete => FontOrEmoji::Emoji,
                    Status::Keep => FontOrEmoji::Emoji,
                },
            }
        }

        fn flush(glyphs: &mut Vec<Glyph>, shaper: Shaper, key: CacheKey, color: Rgb) {
            shaper.shape_with(|cluster| {
                for glyph in cluster.glyphs {
                    glyphs.push(Glyph {
                        id: glyph.id,
                        advance: glyph.advance,
                        range: cluster.source,
                        key,
                        color,
                    })
                }
            });
        }

        let font = context.fonts.get(key).expect("Font not found").as_ref();
        let emoji = context.fonts.emoji();

        let mut glyphs = vec![];
        let mut offset = 0;
        let mut cluster = CharCluster::default();

        for (color, str) in iter {
            let mut font_or_emoji = FontOrEmoji::Font;
            let mut shaper = build!(context, font, size);
            let mut parser = Parser::new(SCRIPT, str.char_indices().map(token(offset)));

            while parser.next(&mut cluster) {
                shaper = match (select(font, emoji, &mut cluster), font_or_emoji) {
                    (FontOrEmoji::Font, FontOrEmoji::Font) => shaper,
                    (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => shaper,
                    (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
                        flush(&mut glyphs, shaper, key, color);
                        font_or_emoji = FontOrEmoji::Font;
                        build!(context, font, size)
                    }
                    (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                        flush(&mut glyphs, shaper, key, color);
                        font_or_emoji = FontOrEmoji::Emoji;
                        build!(context, emoji, size)
                    }
                };

                shaper.add_cluster(&cluster);

                let range = cluster.range();
                offset += range.end - range.start;
            }

            flush(&mut glyphs, shaper, key, color);
        }

        Self { glyphs, key, size }
    }

    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }
}
