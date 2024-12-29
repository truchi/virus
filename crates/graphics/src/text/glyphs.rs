use crate::{
    color::Rgba,
    text::{FontFamilyKey, FontKey, FontSize, FontStyle, FontWeight, Fonts},
};
use std::ops::Range;
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    shape::{ShapeContext, Shaper as SwashShaper},
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
    GlyphId,
};

const SCRIPT: Script = Script::Unknown;
const FEATURES: &'static [(&'static str, u16)] = &[("dlig", 1), ("calt", 1)];
const HINT: bool = true;
const SOURCES: &[Source] = &[
    Source::ColorOutline(0),
    Source::ColorBitmap(StrikeWith::BestFit),
    Source::Outline,
    Source::Bitmap(StrikeWith::BestFit),
];

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Glyphs                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A line of [`Glyph`]s.
#[derive(Clone, Default, Debug)]
pub struct Glyphs {
    glyphs: Vec<Glyph>,
}

impl Glyphs {
    /// Returns a [`Shaper`] for `family` at `size`.
    pub fn shaper<'a>(
        fonts: &'a Fonts,
        shape: &'a mut ShapeContext,
        family: FontFamilyKey,
        size: FontSize,
    ) -> Shaper<'a> {
        Shaper {
            fonts,
            shape,
            family,
            size,
            glyphs: Self { glyphs: Vec::new() },
            column: 0,
        }
    }

    /// Returns a [`Scaler`].
    pub fn scaler<'a>(fonts: &'a Fonts, scale: &'a mut ScaleContext) -> Scaler<'a> {
        Scaler {
            fonts,
            scale,
            render: Render::new(SOURCES),
        }
    }

    /// Returns the [`Glyph`]s.
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    /// Returns the [`Glyph`]s mutably.
    pub fn glyphs_mut(&mut self) -> &mut [Glyph] {
        &mut self.glyphs
    }

    /// Returns the full advance.
    pub fn advance(&self) -> f32 {
        self.glyphs
            .last()
            .map(|glyph| glyph.offset + glyph.advance)
            .unwrap_or_default()
    }

    /// Returns an iterator of background color ranges.
    pub fn backgrounds<'a>(&'a self) -> impl 'a + Iterator<Item = (Range<f32>, Rgba)> {
        let mut glyphs = self.glyphs.iter().peekable();

        std::iter::from_fn(move || {
            std::iter::repeat(())
                .map_while(|()| glyphs.next_if(|glyph| !glyph.styles.background.is_visible()))
                .count();

            let start = glyphs.next()?;
            let background = start.styles.background;
            let end = std::iter::repeat(())
                .map_while(|_| glyphs.next_if(|glyph| glyph.styles.background == background))
                .last()
                .unwrap_or(start);

            debug_assert!(background.is_visible());
            return Some((start.offset..end.offset + end.advance, background));
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Shaper                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Glyphs`] shaper.
pub struct Shaper<'a> {
    fonts: &'a Fonts,
    shape: &'a mut ShapeContext,
    family: FontFamilyKey,
    size: FontSize,
    glyphs: Glyphs,
    column: u32,
}

impl<'a> Shaper<'a> {
    /// Pushes `str` to the shaper with `weight`, `style` and `styles`.
    ///
    /// `str` MUST NOT contain line breaks. Cannot shape ligatures across `str`s.
    pub fn push(&mut self, str: &str, styles: Styles) -> &mut Self {
        debug_assert!(!str.contains(['\r', '\n']));

        let font = self
            .fonts
            .get((self.family, styles.weight, styles.style))
            .expect("Font not found in font cache");
        let emoji = self.fonts.emoji();
        let font_size = self.size;
        let emoji_size = self
            .fonts
            .emoji()
            .size_for_advance(2.0 * font.advance_for_size(font_size));
        let font_charmap = font.as_ref().charmap();
        let emoji_charmap = emoji.as_ref().charmap();

        let mut current_key = font.key();
        let mut cluster = CharCluster::default();
        let mut parser = Parser::new(
            SCRIPT,
            str.char_indices().map({
                let column = self.column;

                move |(i, ch)| Token {
                    ch,
                    offset: column + i as u32,
                    len: ch.len_utf8() as u8,
                    info: ch.into(),
                    data: 0,
                }
            }),
        );
        let mut shaper = self
            .shape
            .builder(font.as_ref())
            .script(SCRIPT)
            .size(font_size as f32)
            .features(FEATURES)
            .build();
        let mut flush = |shaper: SwashShaper, font, size| {
            shaper.shape_with(|cluster| {
                for glyph in cluster.glyphs {
                    self.glyphs.glyphs.push(Glyph {
                        font,
                        size,
                        id: glyph.id,
                        offset: self.glyphs.advance(),
                        advance: glyph.advance,
                        start: cluster.source.start,
                        end: cluster.source.end,
                        styles,
                    });
                }
            });
        };

        while parser.next(&mut cluster) {
            let selected_key = match cluster.map(|char| font_charmap.map(char)) {
                Status::Discard => {
                    cluster.map(|char| emoji_charmap.map(char));
                    emoji.key()
                }
                Status::Keep => match cluster.map(|char| emoji_charmap.map(char)) {
                    Status::Discard => {
                        cluster.map(|char| font_charmap.map(char));
                        font.key()
                    }
                    Status::Keep => emoji.key(),
                    Status::Complete => emoji.key(),
                },
                Status::Complete => font.key(),
            };

            if current_key != selected_key {
                flush(
                    shaper,
                    current_key,
                    match () {
                        _ if current_key == font.key() => font_size,
                        _ if current_key == emoji.key() => emoji_size,
                        _ => unreachable!(),
                    },
                );

                shaper = self
                    .shape
                    .builder(match () {
                        _ if selected_key == font.key() => font.as_ref(),
                        _ if selected_key == emoji.key() => emoji.as_ref(),
                        _ => unreachable!(),
                    })
                    .script(SCRIPT)
                    .size(match () {
                        _ if selected_key == font.key() => font_size as f32,
                        _ if selected_key == emoji.key() => emoji_size as f32,
                        _ => unreachable!(),
                    })
                    .features(FEATURES)
                    .build();

                current_key = selected_key;
            }

            shaper.add_cluster(&cluster);
            self.column += cluster.range().to_range().len() as u32;
        }

        flush(
            shaper,
            current_key,
            match () {
                _ if current_key == font.key() => font_size,
                _ if current_key == emoji.key() => emoji_size,
                _ => unreachable!(),
            },
        );

        self
    }

    /// Returns the [`Glyphs`] and resets the shaper.
    pub fn glyphs(&mut self) -> Glyphs {
        self.column = 0;
        std::mem::take(&mut self.glyphs)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Scaler                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO
// - reuse scaler (iterator api)
// - subpixel

/// [`Glyphs`] scaler.
pub struct Scaler<'a> {
    fonts: &'a Fonts,
    scale: &'a mut ScaleContext,
    render: Render<'static>,
}

impl<'a> Scaler<'a> {
    /// Renders `glyph`.
    pub fn render(&mut self, glyph: &Glyph) -> Image {
        let font = self
            .fonts
            .get(glyph.font)
            .expect("Font not found in font cache");
        let scaler = &mut self
            .scale
            .builder(font.as_ref())
            .size(glyph.size as f32)
            .hint(HINT)
            .build();

        if let Some(image) = self.render.render(scaler, glyph.id) {
            image
        } else {
            debug_assert!(false, "No image for glyph");
            Image::default()
        }
    }
}
