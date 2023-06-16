use crate::text::*;
use std::ops::Range;
use swash::{
    scale::{image::Image, Render, ScaleContext},
    shape::{ShapeContext, Shaper},
    text::cluster::{CharCluster, Parser, Status, Token},
    FontRef,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Line                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A line of shaped [`Glyph`]s.
#[derive(Clone, Debug)]
pub struct Line {
    /// Glyphs.
    glyphs: Vec<Glyph>,
    /// Font family.
    family: FontFamilyKey,
    /// Font size.
    size: FontSize,
}

impl Line {
    /// Returns a `LineShaper` at `size` with `context`.
    pub fn shaper<'a>(
        context: &'a mut Context,
        family: FontFamilyKey,
        size: FontSize,
    ) -> LineShaper<'a> {
        LineShaper::new(context, family, size)
    }

    /// Returns a `LineScaler` of this `line` with `context`.
    pub fn scaler<'a>(&'a self, context: &'a mut Context) -> LineScaler<'a> {
        LineScaler::new(context, self)
    }

    /// Returns the glyphs of this `Line`.
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    /// Returns the `FontFamilyKey` of this `Line`.
    pub fn family(&self) -> FontFamilyKey {
        self.family
    }

    /// Returns the `FontSize` of this `Line`.
    pub fn size(&self) -> FontSize {
        self.size
    }

    /// Returns the width of this `Line`.
    pub fn width(&self) -> Advance {
        self.glyphs
            .last()
            .map(|glyph| glyph.offset + glyph.advance)
            .unwrap_or_default()
    }

    /// Returns an iterator of contiguous backgrounds.
    pub fn backgrounds(&self) -> impl '_ + Iterator<Item = (Range<f32>, Rgba)> {
        // NOTE: we could also compute in `LineShaper::push()` and store in `Line`

        let mut glyphs = self.glyphs().iter().copied();
        let mut previous = glyphs.next();

        std::iter::from_fn(move || {
            let prev = previous?;
            let mut end = prev.offset + prev.advance;

            loop {
                if let Some(next) = glyphs.next() {
                    if prev.styles.background == next.styles.background {
                        end = next.offset + next.advance;
                        continue;
                    } else {
                        previous = Some(next);
                    }
                } else {
                    previous = None;
                }

                return Some((prev.offset..end, prev.styles.background));
            }
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LineShaper                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone)]
enum FontOrEmoji {
    Font,
    Emoji,
}

/// A [`Line`] shaper.
pub struct LineShaper<'a> {
    fonts: &'a Fonts,
    shape: &'a mut ShapeContext,
    line: Line,
    bytes: u32,
    advance: Advance,
    emoji_size: FontSize,
}

impl<'a> LineShaper<'a> {
    /// Creates a new `LineShaper` at `size` with `context`.
    pub fn new(context: &'a mut Context, family: FontFamilyKey, size: FontSize) -> Self {
        let mono = context
            .advance(
                context
                    .fonts()
                    .get((family, FontWeight::Regular, FontStyle::Normal))
                    .unwrap()
                    .key(),
                size,
            )
            .unwrap();
        let emoji_size = context.fonts().emoji().size_for_advance(2.0 * mono).round() as u8;
        let (fonts, _, shape, _) = context.as_muts();

        Self {
            fonts,
            shape,
            line: Line {
                glyphs: Vec::new(),
                family,
                size,
            },
            bytes: 0,
            advance: 0.,
            emoji_size,
        }
    }

    /// Feeds `str` to the `LineShaper` with font `styles`.
    ///
    /// Not able to produce ligature across calls to this function.
    pub fn push(&mut self, str: &str, styles: Styles) {
        let font = self
            .fonts
            .get((self.line.family, styles.weight, styles.style))
            .unwrap()
            .as_ref();
        let emoji = self.fonts.emoji().as_ref();
        let font_key = font.key;
        let emoji_key = emoji.key;
        let font_size = self.line.size;
        let emoji_size = self.emoji_size;

        let mut key = font_key;
        let mut size = font_size;
        let mut font_or_emoji = FontOrEmoji::Font;
        let mut cluster = CharCluster::default();
        let mut shaper = Self::build(self.shape, font, self.line.size);
        let mut parser = Parser::new(SCRIPT, str.char_indices().map(Self::token(self.bytes)));

        while parser.next(&mut cluster) {
            shaper = match (Self::select(font, emoji, &mut cluster), font_or_emoji) {
                (FontOrEmoji::Font, FontOrEmoji::Font) => shaper,
                (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => shaper,
                (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
                    Self::flush(&mut self.line, &mut self.advance, shaper, key, size, styles);
                    (key, size, font_or_emoji) = (font_key, font_size, FontOrEmoji::Font);
                    Self::build(self.shape, font, self.line.size)
                }
                (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                    Self::flush(&mut self.line, &mut self.advance, shaper, key, size, styles);
                    (key, size, font_or_emoji) = (emoji_key, emoji_size, FontOrEmoji::Emoji);
                    Self::build(self.shape, emoji, self.emoji_size)
                }
            };

            shaper.add_cluster(&cluster);

            let range = cluster.range();
            self.bytes += range.end - range.start;
        }

        Self::flush(&mut self.line, &mut self.advance, shaper, key, size, styles);
    }

    /// Returns the shaped `Line`.
    pub fn line(self) -> Line {
        self.line
    }
}

/// Private.
impl<'a> LineShaper<'a> {
    fn token(offset: u32) -> impl Clone + Fn((usize, char)) -> Token {
        move |(i, ch)| Token {
            ch,
            offset: offset + i as u32,
            len: ch.len_utf8() as u8,
            info: ch.into(),
            data: 0,
        }
    }

    fn build<'b>(shape: &'b mut ShapeContext, font: FontRef<'b>, size: FontSize) -> Shaper<'b> {
        shape
            .builder(font)
            .script(SCRIPT)
            .size(size as f32)
            .features(FEATURES)
            .build()
    }

    fn select(font: FontRef, emoji: FontRef, cluster: &mut CharCluster) -> FontOrEmoji {
        match cluster.map(|ch| font.charmap().map(ch)) {
            Status::Discard => {
                // Make sure to map cluster with correct font
                cluster.map(|ch| emoji.charmap().map(ch));
                FontOrEmoji::Emoji
            }
            Status::Complete => FontOrEmoji::Font,
            Status::Keep => match cluster.map(|ch| emoji.charmap().map(ch)) {
                Status::Discard => {
                    // Make sure to map cluster with correct font
                    cluster.map(|ch| font.charmap().map(ch));
                    FontOrEmoji::Font
                }
                Status::Complete => FontOrEmoji::Emoji,
                Status::Keep => FontOrEmoji::Emoji,
            },
        }
    }

    fn flush(
        line: &mut Line,
        offset: &mut Advance,
        shaper: Shaper,
        font: FontKey,
        size: FontSize,
        styles: Styles,
    ) {
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                line.glyphs.push(Glyph {
                    font,
                    size,
                    id: glyph.id,
                    offset: *offset,
                    advance: glyph.advance,
                    range: cluster.source,
                    styles,
                });
                *offset += glyph.advance;
            }
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LineScaler                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`Line`] scaler.
pub struct LineScaler<'a> {
    fonts: &'a Fonts,
    cache: &'a mut Glyphs,
    scale: &'a mut ScaleContext,
    glyphs: std::slice::Iter<'a, Glyph>,
    size: FontSize,
    render: Render<'static>,
}

impl<'a> LineScaler<'a> {
    /// Creates a new `LineScaler` of `line` with `context`.
    pub fn new(context: &'a mut Context, line: &'a Line) -> Self {
        let (fonts, cache, _, scale) = context.as_muts();

        Self {
            fonts,
            cache,
            scale,
            glyphs: line.glyphs.iter(),
            size: line.size,
            render: Render::new(SOURCES),
        }
    }

    /// Returns the next glyph and its image.
    pub fn next<'b>(&'b mut self) -> Option<(Glyph, Option<&'b Image>)> {
        let glyph = self.glyphs.next()?;
        let font = self.fonts.get(glyph.font).expect("font").as_ref();
        let image = self
            .cache
            .get_or_insert((glyph.font, glyph.id, glyph.size), || {
                self.render.render(
                    &mut self
                        .scale
                        .builder(font)
                        .size(self.size as f32)
                        .hint(HINT)
                        .build(),
                    glyph.id,
                )
            });

        Some((*glyph, image.as_ref()))
    }
}
