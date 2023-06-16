use crate::text::*;
use std::ops::Range;
use swash::{
    scale::{image::Image, Render, ScaleContext},
    shape::{ShapeContext, Shaper},
    text::cluster::{CharCluster, Parser, Status, Token},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Line                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A line of shaped [`Glyph`]s.
#[derive(Clone, Debug)]
pub struct Line {
    /// Glyphs.
    glyphs: Vec<Glyph>,
    /// Font size.
    size: FontSize,
    /// Advance.
    advance: Advance,
}

impl Line {
    /// Returns a `LineShaper` for `size`.
    pub fn shaper<'a>(context: &'a mut Context, size: FontSize) -> LineShaper<'a> {
        LineShaper::new(context, size)
    }

    /// Returns a `LineScaler` of this `Line`.
    pub fn scaler<'a>(&'a self, context: &'a mut Context) -> LineScaler<'a> {
        LineScaler::new(context, self)
    }

    /// Returns the `Glyph`s of this `Line`.
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    /// Returns the `FontSize` of this `Line`.
    pub fn size(&self) -> FontSize {
        self.size
    }

    /// Returns the `Advance` of this `Line`.
    pub fn advance(&self) -> Advance {
        self.advance
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
}

impl<'a> LineShaper<'a> {
    /// Creates a new `LineShaper` at `size` with `context`.
    pub fn new(context: &'a mut Context, size: FontSize) -> Self {
        let (fonts, _, shape, _) = context.as_muts();

        Self {
            fonts,
            shape,
            line: Line {
                glyphs: Vec::new(),
                size,
                advance: 0.,
            },
            bytes: 0,
        }
    }

    /// Feeds `str` to the `LineShaper` with font `styles`.
    ///
    /// Not able to produce ligature across calls to this function.
    pub fn push(&mut self, str: &str, family: FontFamilyKey, styles: Styles) {
        let font = self
            .fonts
            .get((family, styles.weight, styles.style))
            .unwrap();
        let emoji = self.fonts.emoji();
        let font_key = font.key();
        let emoji_key = emoji.key();
        let font_size = self.line.size;
        let emoji_size = self
            .fonts
            .emoji()
            .size_for_advance(2.0 * font.advance_for_size(font_size));

        let line = &mut self.line;
        let mut font_or_emoji = FontOrEmoji::Font;
        let mut cluster = CharCluster::default();
        let mut shaper = Self::build(self.shape, font, font_size);
        let mut parser = Parser::new(SCRIPT, str.char_indices().map(Self::token(self.bytes)));

        while parser.next(&mut cluster) {
            match (Self::select(font, emoji, &mut cluster), font_or_emoji) {
                (FontOrEmoji::Font, FontOrEmoji::Font) => {}
                (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => {}
                (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
                    Self::flush(line, shaper, emoji_key, emoji_size, styles);
                    font_or_emoji = FontOrEmoji::Font;
                    shaper = Self::build(self.shape, font, font_size);
                }
                (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                    Self::flush(line, shaper, font_key, font_size, styles);
                    font_or_emoji = FontOrEmoji::Emoji;
                    shaper = Self::build(self.shape, emoji, emoji_size);
                }
            };

            shaper.add_cluster(&cluster);
            self.bytes += {
                let range = cluster.range();
                range.end - range.start
            };
        }

        match font_or_emoji {
            FontOrEmoji::Font => Self::flush(line, shaper, font_key, font_size, styles),
            FontOrEmoji::Emoji => Self::flush(line, shaper, emoji_key, emoji_size, styles),
        }
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

    fn build<'b>(shape: &'b mut ShapeContext, font: &'b Font, size: FontSize) -> Shaper<'b> {
        shape
            .builder(font)
            .script(SCRIPT)
            .size(size as f32)
            .features(FEATURES)
            .build()
    }

    fn select(font: &Font, emoji: &Font, cluster: &mut CharCluster) -> FontOrEmoji {
        match cluster.map(|ch| font.as_ref().charmap().map(ch)) {
            Status::Discard => {
                // Make sure to map cluster with correct font
                cluster.map(|ch| emoji.as_ref().charmap().map(ch));
                FontOrEmoji::Emoji
            }
            Status::Complete => FontOrEmoji::Font,
            Status::Keep => match cluster.map(|ch| emoji.as_ref().charmap().map(ch)) {
                Status::Discard => {
                    // Make sure to map cluster with correct font
                    cluster.map(|ch| font.as_ref().charmap().map(ch));
                    FontOrEmoji::Font
                }
                Status::Complete => FontOrEmoji::Emoji,
                Status::Keep => FontOrEmoji::Emoji,
            },
        }
    }

    fn flush(line: &mut Line, shaper: Shaper, font: FontKey, size: FontSize, styles: Styles) {
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                line.glyphs.push(Glyph {
                    font,
                    size,
                    id: glyph.id,
                    offset: line.advance,
                    advance: glyph.advance,
                    range: cluster.source,
                    styles,
                });
                line.advance += glyph.advance;
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
            .get_or_insert((glyph.font, glyph.size, glyph.id), || {
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
