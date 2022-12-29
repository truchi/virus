use crate::{context::Context, font::Font, glyph::Glyph, Buffer, FontSize, Rgb};
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

#[derive(Clone, Debug)]
pub struct Line {
    glyphs: Vec<Glyph>,
    size: FontSize,
}

impl Line {
    pub fn from_iter<'a, I>(context: &mut Context, iter: I, key: CacheKey, size: FontSize) -> Self
    where
        I: IntoIterator<Item = (Rgb, &'a str)>,
    {
        const SCRIPT: Script = Script::Unknown;
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
                Status::Discard => {
                    // Making sure to map cluster with correct font
                    cluster.map(|ch| emoji.charmap().map(ch));
                    FontOrEmoji::Emoji
                }
                Status::Complete => FontOrEmoji::Font,
                Status::Keep => match cluster.map(|ch| emoji.charmap().map(ch)) {
                    Status::Discard => {
                        // Making sure to map cluster with correct font
                        cluster.map(|ch| font.charmap().map(ch));
                        FontOrEmoji::Font
                    }
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
        let font_key = font.key;
        let emoji_key = emoji.key;

        let mut glyphs = vec![];
        let mut offset = 0;
        let mut cluster = CharCluster::default();

        for (color, str) in iter {
            let mut key = font_key;
            let mut font_or_emoji = FontOrEmoji::Font;
            let mut shaper = build!(context, font, size);
            let mut parser = Parser::new(SCRIPT, str.char_indices().map(token(offset)));

            while parser.next(&mut cluster) {
                shaper = match (select(font, emoji, &mut cluster), font_or_emoji) {
                    (FontOrEmoji::Font, FontOrEmoji::Font) => shaper,
                    (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => shaper,
                    (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
                        flush(&mut glyphs, shaper, key, color);
                        key = font_key;
                        font_or_emoji = FontOrEmoji::Font;
                        build!(context, font, size)
                    }
                    (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                        flush(&mut glyphs, shaper, key, color);
                        key = emoji_key;
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

        Self { glyphs, size }
    }

    pub fn size(&self) -> FontSize {
        self.size
    }

    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }
}
