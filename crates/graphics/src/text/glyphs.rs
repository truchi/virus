use super::{
    Advance, Context, FontFamilyKey, FontKey, FontSize, FontStyle, FontWeight, Fonts, GlyphKey,
    Styles, FEATURES, HINT, SCRIPT, SOURCES,
};
use crate::types::Rgba;
use std::ops::Range;
use swash::{
    scale::{image::Image, Render, ScaleContext},
    shape::{ShapeContext, Shaper as SwashShaper},
    text::cluster::{CharCluster, Parser, Status, Token},
    GlyphId,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Glyph                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A shaped glyph.
#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    /// Font key.
    pub font: FontKey,
    /// Font size.
    pub size: FontSize,
    /// Glyph id.
    pub id: GlyphId,
    /// Glyph advance offset.
    pub offset: Advance,
    /// Glyph advance.
    pub advance: Advance,
    /// Start index in the underlying string.
    pub start: u32,
    /// End index in the underlying string.
    pub end: u32,
    /// Styles tag.
    pub styles: u16,
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

/// Lines of [`Glyph`]s.
#[derive(Clone, Debug)]
pub struct Glyphs {
    lines: Vec<Vec<Glyph>>,
}

impl Glyphs {
    /// Returns a [`Shaper`] for `family` at `size`.
    pub fn shaper<'context>(
        context: &'context mut Context,
        family: FontFamilyKey,
        size: FontSize,
    ) -> Shaper<'context> {
        let (fonts, shape, _) = context.as_muts();

        Shaper {
            fonts,
            shape,
            family,
            size,
            glyphs: Self { lines: Vec::new() },
            line: 0,
            column: 0,
        }
    }

    /// Returns a [`Scaler`].
    pub fn scaler<'context>(context: &'context mut Context) -> Scaler<'context> {
        let (fonts, _, scale) = context.as_muts();

        Scaler {
            fonts,
            scale,
            render: Render::new(SOURCES),
        }
    }

    /// Returns the lines of [`Glyph`]s.
    pub fn lines(&self) -> &Vec<Vec<Glyph>> {
        &self.lines
    }

    /// Returns an iterator of background color ranges.
    pub fn backgrounds<'a>(
        line: &'a [Glyph],
        styles: impl 'a + Fn(u16) -> Styles,
    ) -> impl 'a + Iterator<Item = (Range<Advance>, Rgba)> {
        let background = move |glyph: &Glyph| styles(glyph.styles).background;
        let mut glyphs = line.iter().peekable();

        std::iter::from_fn(move || {
            std::iter::repeat(())
                .flat_map(|()| glyphs.next_if(|glyph| !background(glyph).is_visible()))
                .count();

            let start = glyphs.next()?;
            let color = background(start);
            let end = std::iter::repeat(())
                .flat_map(|_| glyphs.next_if(|glyph| background(glyph) == color))
                .last()
                .unwrap_or(start);

            debug_assert!(color.is_visible());
            return Some((start.offset..end.offset + end.advance, color));
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Shaper                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Glyph`]'s shaper.
pub struct Shaper<'context> {
    fonts: &'context Fonts,
    shape: &'context mut ShapeContext,
    family: FontFamilyKey,
    size: FontSize,
    glyphs: Glyphs,
    line: usize,
    column: u32,
}

impl<'context> Shaper<'context> {
    /// Pushes `str` to the shaper with `weight`, `style` and `styles`.
    ///
    /// `"\r\n"` MUST NOT be split across `str`s.
    pub fn push(
        &mut self,
        str: impl AsRef<str>,
        weight: FontWeight,
        style: FontStyle,
        styles: u16,
    ) -> &mut Self {
        let (str, next_line) = {
            let str = str.as_ref();

            if str.is_empty() {
                return self;
            }

            match str.find(['\r', '\n']) {
                Some(index) => {
                    let (left, right) = str.split_at(index);

                    match right.as_bytes()[0] {
                        b'\r' => {
                            if right.as_bytes().get(1) == Some(&b'\n') {
                                (left, Some(&right[2..]))
                            } else {
                                (left, Some(&right[1..]))
                            }
                        }
                        b'\n' => (left, Some(&right[1..])),
                        _ => unreachable!(),
                    }
                }
                None => (str, None),
            }
        };
        let line = {
            if self.glyphs.lines.get(self.line).is_none() {
                self.glyphs.lines.push(Vec::new());
            }

            &mut self.glyphs.lines[self.line]
        };

        let font = self
            .fonts
            .get((self.family, weight, style))
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
                    data: styles as u32,
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

        fn flush(line: &mut Vec<Glyph>, shaper: SwashShaper, font: FontKey, size: FontSize) {
            let mut offset = line
                .last()
                .map(|glyph| glyph.offset + glyph.advance)
                .unwrap_or_default();

            shaper.shape_with(|cluster| {
                for glyph in cluster.glyphs {
                    line.push(Glyph {
                        font,
                        size,
                        id: glyph.id,
                        offset,
                        advance: glyph.advance,
                        start: cluster.source.start,
                        end: cluster.source.end,
                        styles: glyph.data as u16,
                    });
                    offset += glyph.advance;
                }
            });
        }

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
                    line,
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
            line,
            shaper,
            current_key,
            match () {
                _ if current_key == font.key() => font_size,
                _ if current_key == emoji.key() => emoji_size,
                _ => unreachable!(),
            },
        );

        if let Some(str) = next_line {
            self.line += 1;
            self.column = 0;
            self.push(str, weight, style, styles)
        } else {
            self
        }
    }

    /// Returns the shaped [`Glyphs`].
    pub fn glyphs(self) -> Glyphs {
        self.glyphs
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Scaler                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Glyph`]'s scaler.
pub struct Scaler<'context> {
    fonts: &'context Fonts,
    scale: &'context mut ScaleContext,
    render: Render<'static>,
}

impl<'context> Scaler<'context> {
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
