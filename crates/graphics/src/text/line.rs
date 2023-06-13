use crate::text::*;
use std::ops::Range;
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
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
    /// Font size.
    size: FontSize,
}

impl Line {
    /// Returns a `LineShaper` at `size` with `context`.
    pub fn shaper<'a>(context: &'a mut Context, size: FontSize) -> LineShaper<'a> {
        LineShaper::new(context, size)
    }

    /// Returns a `LineScaler` of this `line` with `context`.
    pub fn scaler<'a>(&'a self, context: &'a mut Context) -> LineScaler<'a> {
        LineScaler::new(context, self)
    }

    /// Returns the glyphs of this `Line`.
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    /// Returns the `FontSize` of this `Line`.
    pub fn size(&self) -> FontSize {
        self.size
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
            },
            bytes: 0,
            advance: 0.,
        }
    }

    /// Feeds `str` to the `LineShaper` with font `styles`.
    ///
    /// Not able to produce ligature across calls to this function.
    pub fn push(&mut self, str: &str, styles: Styles) {
        let font = self
            .fonts
            .get(styles.font)
            .expect("Font not found")
            .as_ref();
        let emoji = self.fonts.emoji().as_ref();
        let font_key = font.key;
        let emoji_key = emoji.key;

        let mut key = font_key;
        let mut font_or_emoji = FontOrEmoji::Font;
        let mut cluster = CharCluster::default();
        let mut shaper = Self::build(self.shape, font, self.line.size);
        let mut parser = Parser::new(SCRIPT, str.char_indices().map(Self::token(self.bytes)));

        while parser.next(&mut cluster) {
            shaper = match (Self::select(font, emoji, &mut cluster), font_or_emoji) {
                (FontOrEmoji::Font, FontOrEmoji::Font) => shaper,
                (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => shaper,
                (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
                    Self::flush(&mut self.line, &mut self.advance, shaper, key, styles);
                    (key, font_or_emoji) = (font_key, FontOrEmoji::Font);
                    Self::build(self.shape, font, self.line.size)
                }
                (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                    Self::flush(&mut self.line, &mut self.advance, shaper, key, styles);
                    (key, font_or_emoji) = (emoji_key, FontOrEmoji::Emoji);
                    Self::build(self.shape, emoji, self.line.size)
                }
            };

            shaper.add_cluster(&cluster);

            let range = cluster.range();
            self.bytes += range.end - range.start;
        }

        Self::flush(&mut self.line, &mut self.advance, shaper, key, styles);
    }

    /// Returns the shaped `Line`.
    pub fn line(self) -> Line {
        self.line
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

    fn flush(line: &mut Line, advance: &mut Advance, shaper: Shaper, key: FontKey, styles: Styles) {
        let mut styles = styles;
        styles.font = key;

        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                line.glyphs.push(Glyph {
                    id: glyph.id,
                    offset: *advance,
                    advance: glyph.advance,
                    range: cluster.source,
                    styles,
                });
                *advance += glyph.advance;
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
    advance: Advance,
    render: Render<'static>,
}

impl<'a> LineScaler<'a> {
    const HINT: bool = true;
    const SOURCES: &[Source] = &[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
        Source::Bitmap(StrikeWith::BestFit),
    ];

    /// Creates a new `LineScaler` of `line` with `context`.
    pub fn new(context: &'a mut Context, line: &'a Line) -> Self {
        let (fonts, cache, _, scale) = context.as_muts();

        Self {
            fonts,
            cache,
            scale,
            glyphs: line.glyphs.iter(),
            size: line.size,
            advance: 0.,
            render: Render::new(Self::SOURCES),
        }
    }

    /// Returns the next glyph, along with its advance and image.
    pub fn next<'b>(&'b mut self) -> Option<(Advance, Glyph, Option<&'b Image>)> {
        let advance = self.advance;
        let glyph = self.glyphs.next()?;
        let key = glyph.styles.font;
        let font = self.fonts.get(key).expect("font").as_ref();
        let image = self.cache.get_or_insert((key, glyph.id, self.size), || {
            self.render.render(
                &mut self
                    .scale
                    .builder(font)
                    .size(self.size as f32)
                    .hint(Self::HINT)
                    .build(),
                glyph.id,
            )
        });

        self.advance += glyph.advance;
        Some((advance, *glyph, image.as_ref()))
    }
}
