use crate::{
    color::Rgba,
    text::{Font, FontFamilyKey, FontKey, FontSize, FontStyle, FontWeight, Fonts},
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
    pub offset: u32,
    /// Glyph advance.
    pub advance: u32,
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
        advance: u32,
    ) -> Shaper<'a> {
        let font_size = size;
        let font_advance = advance;
        let emoji_advance = 2 * font_advance;
        let emoji_size = FontSize::new(fonts.emoji().size_for_advance(emoji_advance as f32));

        Shaper {
            fonts,
            shape,
            family,
            font_size,
            emoji_size,
            font_advance,
            emoji_advance,
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
    pub fn advance(&self) -> u32 {
        self.glyphs
            .last()
            .map(|glyph| glyph.offset + glyph.advance)
            .unwrap_or_default()
    }

    /// Returns an iterator of background color ranges.
    pub fn backgrounds<'a>(&'a self) -> impl 'a + Iterator<Item = (Range<u32>, Rgba)> {
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
    font_size: FontSize,
    emoji_size: FontSize,
    font_advance: u32,
    emoji_advance: u32,
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
        let font_charmap = font.as_ref().charmap();
        let emoji_charmap = emoji.as_ref().charmap();

        let mut current_key = font.key();
        let mut cluster = CharCluster::default();
        let mut shaper = Self::build(&mut self.shape, (font, self.font_size));
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
                Self::flush(
                    &mut self.glyphs,
                    shaper,
                    current_key,
                    match () {
                        _ if current_key == font.key() => (self.font_size, self.font_advance),
                        _ if current_key == emoji.key() => (self.emoji_size, self.emoji_advance),
                        _ => unreachable!(),
                    },
                    styles,
                );

                current_key = selected_key;
                shaper = Self::build(
                    &mut self.shape,
                    match () {
                        _ if current_key == font.key() => (font, self.font_size),
                        _ if current_key == emoji.key() => (emoji, self.emoji_size),
                        _ => unreachable!(),
                    },
                );
            }

            shaper.add_cluster(&cluster);
        }

        Self::flush(
            &mut self.glyphs,
            shaper,
            current_key,
            match () {
                _ if current_key == font.key() => (self.font_size, self.font_advance),
                _ if current_key == emoji.key() => (self.emoji_size, self.emoji_advance),
                _ => unreachable!(),
            },
            styles,
        );

        self.column += str.len() as u32;
        self
    }

    /// Resets the shaper and returns the [`Glyphs`].
    pub fn glyphs(&mut self) -> Glyphs {
        self.column = 0;
        std::mem::take(&mut self.glyphs)
    }
}

/// Private.
impl<'a> Shaper<'a> {
    fn build<'b>(
        shape: &'b mut ShapeContext,
        (font, size): (&'b Font, FontSize),
    ) -> SwashShaper<'b> {
        shape
            .builder(font.as_ref())
            .script(SCRIPT)
            .size(size.as_f32())
            .features(FEATURES)
            .build()
    }

    fn flush(
        glyphs: &mut Glyphs,
        shaper: SwashShaper,
        font: FontKey,
        (size, advance): (FontSize, u32),
        styles: Styles,
    ) {
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                glyphs.glyphs.push(Glyph {
                    font,
                    size,
                    id: glyph.id,
                    offset: glyphs.advance(),
                    advance,
                    start: cluster.source.start,
                    end: cluster.source.end,
                    styles,
                });
            }
        });
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
            .size(glyph.size.as_f32())
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
