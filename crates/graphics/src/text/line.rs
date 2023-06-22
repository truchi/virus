use crate::text::*;
use std::{ops::Range, time::Duration};
use swash::{
    scale::{image::Image, Render},
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

    /// Returns an iterator of contiguous `key(glyph)`.
    pub fn segments<'a, T, F>(&'a self, key: F) -> impl 'a + Iterator<Item = (Range<Advance>, T)>
    where
        T: 'a + PartialEq,
        F: 'a + FnMut(&Glyph) -> T,
    {
        LineSegments {
            glyphs: self.glyphs.iter(),
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
    glyphs: std::slice::Iter<'a, Glyph>,
    current: Option<(Range<Advance>, T)>,
    key: F,
}

// TODO tests
impl<'a, T, F> Iterator for LineSegments<'a, T, F>
where
    T: PartialEq,
    F: FnMut(&Glyph) -> T,
{
    type Item = (Range<Advance>, T);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.current.take(), self.glyphs.next()) {
            // Line/Glyphs is empty
            (None, None) => None,
            // First glyph
            (None, Some(glyph)) => {
                self.current = Some((
                    glyph.offset..glyph.offset + glyph.advance,
                    (self.key)(glyph),
                ));
                self.next()
            }
            // Last glyph
            (Some((Range { start, end }, key)), None) => {
                self.current = None; // To stop the iteration
                Some((start..end, key))
            }
            // Same key
            (Some((Range { start, end }, key)), Some(glyph)) if key == (self.key)(glyph) => {
                self.current = Some((start..end + glyph.advance, key));
                self.next()
            }
            // New key
            (Some((Range { start, end }, key)), Some(glyph)) => {
                self.current = Some((
                    glyph.offset..glyph.offset + glyph.advance,
                    (self.key)(glyph),
                ));
                Some((start..end, key))
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
        let (fonts, shape, _) = context.as_muts();

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

        let animated_glyph_id = {
            let offset = self.bytes as usize;
            let animated = self.fonts.animated();
            move |range: SourceRange| {
                let Range { start, end } = range.to_range();
                animated
                    .get_by_str(&str[start - offset..end - offset])
                    .map(|animated_glyph| animated_glyph.id())
            }
        };
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
                    Self::flush(
                        line,
                        shaper,
                        emoji_key,
                        emoji_size,
                        styles,
                        animated_glyph_id,
                    );
                    font_or_emoji = FontOrEmoji::Font;
                    shaper = Self::build(self.shape, font, font_size);
                }
                (FontOrEmoji::Emoji, FontOrEmoji::Font) => {
                    Self::flush(line, shaper, font_key, font_size, styles, |_| None);
                    font_or_emoji = FontOrEmoji::Emoji;
                    shaper = Self::build(self.shape, emoji, emoji_size);
                }
            };

            shaper.add_cluster(&cluster);
            self.bytes += cluster.range().to_range().len() as u32;
        }

        match font_or_emoji {
            FontOrEmoji::Font => Self::flush(line, shaper, font_key, font_size, styles, |_| None),
            FontOrEmoji::Emoji => Self::flush(
                line,
                shaper,
                emoji_key,
                emoji_size,
                styles,
                animated_glyph_id,
            ),
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

    // TODO cache charmaps
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

    fn flush(
        line: &mut Line,
        shaper: Shaper,
        font: FontKey,
        size: FontSize,
        styles: Styles,
        animated_glyph_id: impl Fn(SourceRange) -> Option<AnimatedGlyphId>,
    ) {
        shaper.shape_with(|cluster| {
            debug_assert!(cluster.glyphs.len() <= 1);

            for glyph in cluster.glyphs {
                line.glyphs.push(Glyph {
                    font,
                    size,
                    id: glyph.id,
                    animated_id: animated_glyph_id(cluster.source),
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
    size: FontSize,
    render: Render<'static>,
}

impl<'a> LineScaler<'a> {
    /// Creates a new `LineScaler` of `line` with `context`.
    pub fn new(context: &'a mut Context, line: &'a Line) -> Self {
        Self {
            context,
            size: line.size,
            render: Render::new(SOURCES),
        }
    }

    /// Returns the index of the frame for animated `glyph` at `time`.
    pub fn frame(&self, glyph: &Glyph, time: Duration) -> Option<FrameIndex> {
        Some(
            self.context
                .fonts()
                .animated()
                .get_by_id(glyph.animated_id?)
                .unwrap()
                .frame(time),
        )
    }

    /// Renders `glyph`.
    pub fn render(&mut self, glyph: &Glyph) -> Option<Image> {
        if glyph.animated_id.is_some() {
            debug_assert!(false);
            return None;
        }

        let (fonts, _, scale) = self.context.as_muts();
        let font = fonts.get(glyph.font).expect("font");
        let scaler = &mut scale
            .builder(font)
            .size(self.size as f32)
            .hint(HINT)
            .build();

        self.render.render(scaler, glyph.id)
    }

    /// Renders animated `glyph`.
    pub fn render_animated(&mut self, glyph: &Glyph) -> Option<Vec<Image>> {
        let id = if let Some(id) = glyph.animated_id {
            id
        } else {
            debug_assert!(false);
            return None;
        };

        self.context
            .fonts()
            .animated()
            .get_by_id(id)?
            .render(glyph.advance.floor() as u32)
            .ok()
    }
}
