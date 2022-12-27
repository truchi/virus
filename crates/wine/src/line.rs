use std::{collections::HashMap, path::Path};

use crate::{default, Rgb};
use swash::{
    scale::ScaleContext,
    shape::{ShapeContext, Shaper},
    text::{
        cluster::{CharCluster, Parser, SourceRange, Status, Token},
        Script,
    },
    CacheKey, FontRef, GlyphId,
};

pub struct Fonts {
    fonts: HashMap<CacheKey, Font>,
    emoji: Font,
}

impl Fonts {
    pub fn new(emoji: Font) -> Self {
        Self {
            fonts: default(),
            emoji,
        }
    }

    pub fn insert(&mut self, font: Font) {
        self.fonts.insert(font.key, font);
    }

    pub fn get(&self, key: CacheKey) -> Option<FontRef> {
        Some(self.fonts.get(&key)?.as_ref())
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
    shape: ShapeContext,
    scale: ScaleContext,
}

impl Context {
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            shape: default(),
            scale: default(),
        }
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }
}

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
    size: f32,
}

impl Line {
    pub fn from_iter<'a, I>(context: &mut Context, iter: I, key: CacheKey, size: f32) -> Self
    where
        I: IntoIterator<Item = (Rgb, &'a str)>,
    {
        const SCRIPT: Script = Script::Latin;
        const FEATURES: [(&str, u16); 2] = [("dlig", 1), ("calt", 1)];

        #[derive(Copy, Clone)]
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
                    .size($size)
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

        let font = context.fonts.get(key).expect("Font not found");
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

// pub struct LineBuilder<'a> {
//     context: &'a mut Context,
//     shaper: Shaper<'a>,
//     cluster: CharCluster,
//     offset: u32,
//     line: Line,
// }

// impl<'a> LineBuilder<'a> {
//     pub fn push(&mut self, font_key: FontKey, color: Rgb, str: &str) {
//         let font = self.context.fonts.get(font_key).expect("Font not found");
//         let emoji = self.context.fonts.emoji();

//         let mut parser = Parser::new(
//             Context::SCRIPT,
//             str.char_indices().map(|(i, ch)| Token {
//                 ch,
//                 offset: self.offset + i as u32,
//                 len: ch.len_utf8() as u8,
//                 info: ch.into(),
//                 data: 0,
//             }),
//         );

//         let select = |cluster: &mut CharCluster| match cluster.map(|ch| font.charmap().map(ch)) {
//             Status::Discard => FontOrEmoji::Emoji,
//             Status::Complete => FontOrEmoji::Font,
//             Status::Keep => match cluster.map(|ch| emoji.charmap().map(ch)) {
//                 Status::Discard => FontOrEmoji::Font,
//                 Status::Complete => FontOrEmoji::Emoji,
//                 Status::Keep => FontOrEmoji::Emoji,
//             },
//         };

//         #[derive(Copy, Clone)]
//         enum FontOrEmoji {
//             Font,
//             Emoji,
//         }

//         let mut offset = 0;
//         let mut font_or_emoji = FontOrEmoji::Font;
//         let mut shaper = self
//             .context
//             .shape
//             .builder(font)
//             .script(Context::SCRIPT)
//             .build();

//         while parser.next(&mut self.cluster) {
//             shaper = match (select(&mut self.cluster), font_or_emoji) {
//                 (FontOrEmoji::Font, FontOrEmoji::Font) => shaper,
//                 (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => shaper,
//                 (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
//                     font_or_emoji = FontOrEmoji::Font;
//                     self.context
//                         .shape
//                         .builder(font)
//                         .script(Context::SCRIPT)
//                         .build()
//                 }
//                 (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
//                     font_or_emoji = FontOrEmoji::Emoji;
//                     self.context
//                         .shape
//                         .builder(emoji)
//                         .script(Context::SCRIPT)
//                         .build()
//                 }
//             };

//             shaper.add_cluster(&self.cluster);

//             let range = self.cluster.range();
//             offset += range.end - range.start;
//         }

//         self.offset += offset;
//     }
// }
