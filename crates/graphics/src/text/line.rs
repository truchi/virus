use crate::text::*;
use std::ops::{Range, RangeInclusive};
use swash::{
    scale::{image::Image, Render},
    shape::{ShapeContext, Shaper},
    text::cluster::{CharCluster, Parser, Status, Token},
    Charmap,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Line                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A line of shaped [`Glyph`]s.
#[derive(Clone, Debug)]
pub struct Line {
    /// The shaped glyphs in this line.
    glyphs: Vec<Glyph>,
    /// The font size of this line.
    font_size: FontSize,
    /// The advance of this line.
    advance: Advance,
}

impl Line {
    /// Returns a `LineShaper` at `font_size`.
    pub fn shaper<'a>(
        context: &'a mut Context,
        font_size: FontSize,
        unligature1: Option<RangeInclusive<usize>>,
        unligature2: Option<RangeInclusive<usize>>,
    ) -> LineShaper<'a> {
        LineShaper::new(context, font_size, unligature1, unligature2)
    }

    /// Returns a `LineScaler` for this `Line`.
    pub fn scaler<'a>(&'a self, context: &'a mut Context) -> LineScaler<'a> {
        LineScaler::new(context, self)
    }

    /// Returns the `Glyph`s of this `Line`.
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    /// Returns the `FontSize` of this `Line`.
    pub fn font_size(&self) -> FontSize {
        self.font_size
    }

    /// Returns the `Advance` of this `Line`.
    pub fn advance(&self) -> Advance {
        self.advance
    }

    /// Returns an iterator of contiguous `key(glyph)`.
    pub fn segments<'a, T, F>(
        &'a self,
        key: F,
    ) -> impl 'a + Iterator<Item = (Range<Advance>, &'a [Glyph], T)>
    where
        T: 'a + PartialEq,
        F: 'a + FnMut(&Glyph) -> T,
    {
        LineSegments {
            glyphs: &self.glyphs,
            iter: self.glyphs.iter(),
            current: None,
            key,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          LineSegments                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
struct LineSegments<'a, T, F> {
    glyphs: &'a [Glyph],
    iter: std::slice::Iter<'a, Glyph>,
    current: Option<(Range<Advance>, Range<usize>, T)>,
    key: F,
}

// TODO tests
impl<'a, T, F> Iterator for LineSegments<'a, T, F>
where
    T: PartialEq,
    F: FnMut(&Glyph) -> T,
{
    type Item = (Range<Advance>, &'a [Glyph], T);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.current.take(), self.iter.next()) {
            // Line/Glyphs is empty
            (None, None) => None,
            // First glyph
            (None, Some(glyph)) => {
                self.current = Some((
                    glyph.offset..glyph.offset + glyph.advance,
                    0..1,
                    (self.key)(glyph),
                ));
                self.next()
            }
            // No more glyphs
            (Some((advance, index, key)), None) => {
                self.current = None; // To stop the iteration (input iter must be fused)
                Some((advance, &self.glyphs[index], key))
            }
            // Same key
            (Some((advance, index, key)), Some(glyph)) if key == (self.key)(glyph) => {
                self.current = Some((
                    advance.start..advance.end + glyph.advance,
                    index.start..index.end + 1,
                    key,
                ));
                self.next()
            }
            // New key
            (Some((advance, index, key)), Some(glyph)) => {
                self.current = Some((
                    glyph.offset..glyph.offset + glyph.advance,
                    index.end..index.end + 1,
                    (self.key)(glyph),
                ));
                Some((advance, &self.glyphs[index], key))
            }
        }
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

// ────────────────────────────────────────────────────────────────────────────────────────────── //

/// A [`Line`] shaper.
pub struct LineShaper<'a> {
    fonts: &'a Fonts,
    shape: &'a mut ShapeContext,
    line: Line,
    bytes: u32,
    unligature1: Option<RangeInclusive<usize>>,
    unligature2: Option<RangeInclusive<usize>>,
}

impl<'a> LineShaper<'a> {
    /// Creates a new `LineShaper` at `size` with `context`.
    pub fn new(
        context: &'a mut Context,
        font_size: FontSize,
        unligature1: Option<RangeInclusive<usize>>,
        unligature2: Option<RangeInclusive<usize>>,
    ) -> Self {
        let (fonts, shape, _) = context.as_muts();

        Self {
            fonts,
            shape,
            line: Line {
                glyphs: Vec::new(),
                font_size,
                advance: 0.,
            },
            bytes: 0,
            unligature1,
            unligature2,
        }
    }

    /// Feeds `str` to the `LineShaper` with `family` and `styles`.
    pub fn push(&mut self, str: &str, family: FontFamilyKey, styles: Styles) {
        let font = self
            .fonts
            .get((family, styles.weight, styles.style))
            .expect("Font not found in font cache");
        let emoji = self.fonts.emoji();
        let font_key = font.key();
        let emoji_key = emoji.key();
        let font_size = self.line.font_size;
        let emoji_size = self
            .fonts
            .emoji()
            .size_for_advance(2.0 * font.advance_for_size(font_size));
        let font_charmap = font.as_ref().charmap();
        let emoji_charmap = emoji.as_ref().charmap();

        let line = &mut self.line;
        let mut font_or_emoji = FontOrEmoji::Font;
        let mut cluster = CharCluster::default();
        let mut shaper = Self::build(self.shape, font, font_size);
        let mut parser = Parser::new(SCRIPT, str.char_indices().map(Self::token(self.bytes)));

        while parser.next(&mut cluster) {
            match (
                font_or_emoji,
                Self::select(&mut cluster, font_charmap, emoji_charmap),
            ) {
                (FontOrEmoji::Font, FontOrEmoji::Font) => {}
                (FontOrEmoji::Emoji, FontOrEmoji::Emoji) => {}
                (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                    Self::flush(line, shaper, emoji_key, emoji_size, styles);
                    shaper = Self::build(self.shape, font, font_size);
                    font_or_emoji = FontOrEmoji::Font;
                }
                (FontOrEmoji::Font, FontOrEmoji::Emoji) => {
                    Self::flush(line, shaper, font_key, font_size, styles);
                    shaper = Self::build(self.shape, emoji, emoji_size);
                    font_or_emoji = FontOrEmoji::Emoji;
                }
            };

            let range = cluster.range().to_range();
            debug_assert!(range.start < range.end);
            let (flush_before_1, flush_after_1) =
                Self::flush_to_unligature(range.clone(), self.unligature1.clone());
            let (flush_before_2, flush_after_2) =
                Self::flush_to_unligature(range.clone(), self.unligature2.clone());

            if flush_before_1 || flush_before_2 {
                match font_or_emoji {
                    FontOrEmoji::Font => {
                        Self::flush(line, shaper, font_key, font_size, styles);
                        shaper = Self::build(self.shape, font, font_size);
                    }
                    FontOrEmoji::Emoji => {
                        Self::flush(line, shaper, emoji_key, emoji_size, styles);
                        shaper = Self::build(self.shape, emoji, emoji_size);
                    }
                }
            }

            shaper.add_cluster(&cluster);

            if flush_after_1 || flush_after_2 {
                match font_or_emoji {
                    FontOrEmoji::Font => {
                        Self::flush(line, shaper, font_key, font_size, styles);
                        shaper = Self::build(self.shape, font, font_size);
                    }
                    FontOrEmoji::Emoji => {
                        Self::flush(line, shaper, emoji_key, emoji_size, styles);
                        shaper = Self::build(self.shape, emoji, emoji_size);
                    }
                }
            }

            self.bytes += range.len() as u32;
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
            .builder(font.as_ref())
            .script(SCRIPT)
            .size(size as f32)
            .features(FEATURES)
            .build()
    }

    fn select(cluster: &mut CharCluster, font: Charmap, emoji: Charmap) -> FontOrEmoji {
        match cluster.map(|char| font.map(char)) {
            Status::Discard => {
                cluster.map(|char| emoji.map(char));
                FontOrEmoji::Emoji
            }
            Status::Keep => match cluster.map(|char| emoji.map(char)) {
                Status::Discard => {
                    cluster.map(|char| font.map(char));
                    FontOrEmoji::Font
                }
                Status::Keep => FontOrEmoji::Emoji,
                Status::Complete => FontOrEmoji::Emoji,
            },
            Status::Complete => FontOrEmoji::Font,
        }
    }

    fn flush_to_unligature(
        cluster: Range<usize>,
        unligature: Option<RangeInclusive<usize>>,
    ) -> (bool, bool) {
        let Some(unligature) = unligature else {
            return (false, false);
        };
        let start = *unligature.start();
        let end = *unligature.end();

        // We want to flush before all clusters in the range and after the last one
        if start == end {
            if cluster.start <= start && start < cluster.end {
                (true, true)
            } else {
                (false, false)
            }
        } else {
            if cluster.start < start {
                if cluster.end <= start {
                    (false, false)
                } else if cluster.end < end {
                    (true, false)
                } else {
                    (true, true)
                }
            } else if start <= cluster.start && cluster.start < end {
                if cluster.end < end {
                    (true, false)
                } else {
                    (true, true)
                }
            } else {
                (false, false)
            }
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
    context: &'a mut Context,
    font_size: FontSize,
    render: Render<'static>,
}

impl<'a> LineScaler<'a> {
    /// Creates a new `LineScaler` of `line` with `context`.
    pub fn new(context: &'a mut Context, line: &'a Line) -> Self {
        Self {
            context,
            font_size: line.font_size,
            render: Render::new(SOURCES),
        }
    }

    /// Renders `glyph`.
    pub fn render(&mut self, glyph: &Glyph) -> Image {
        let (fonts, _, scale) = self.context.as_muts();
        let font = fonts.get(glyph.font).expect("Font not found in font cache");
        let scaler = &mut scale
            .builder(font.as_ref())
            .size(self.font_size as f32)
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
