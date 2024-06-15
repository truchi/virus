use crate::text::{
    Advance, Context, FontFamilyKey, FontKey, FontSize, Glyph, Styles, FEATURES, HINT, SCRIPT,
    SOURCES,
};
use std::{
    collections::HashMap,
    ops::{Range, RangeInclusive},
    usize,
};
use swash::{
    scale::{image::Image, Render},
    shape::Shaper,
    text::cluster::{CharCluster, Parser, Status, Token},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Line                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO rename Glyphs?
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
    /// Returns a `LineShaper` with default `pattern` and `styles`.
    pub fn shaper(line: &str, pattern: usize, styles: Styles) -> LineShaper {
        LineShaper::new(line, pattern, styles)
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
pub struct Cluster {
    cluster: CharCluster,
    pattern: usize,
    styles: Styles,
}

impl Cluster {
    pub fn range(&self) -> Range<usize> {
        self.cluster.range().to_range()
    }

    pub fn pattern(&self) -> usize {
        self.pattern
    }

    pub fn styles(&self) -> Styles {
        self.styles
    }

    pub fn pattern_mut(&mut self) -> &mut usize {
        &mut self.pattern
    }

    pub fn styles_mut(&mut self) -> &mut Styles {
        &mut self.styles
    }
}

impl std::fmt::Debug for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unused)]
        #[derive(Debug)]
        struct Cluster<'a> {
            chars: &'a [swash::text::cluster::Char],
            pattern: usize,
            styles: Styles,
        }

        std::fmt::Debug::fmt(
            &Cluster {
                chars: self.cluster.chars(),
                pattern: self.pattern,
                styles: self.styles,
            },
            f,
        )
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct LineShaper {
    clusters: Vec<Cluster>,
    styles: Styles,
}

impl LineShaper {
    /// Creates a new `LineShaper` with default `pattern` and `styles`.
    pub fn new(line: &str, pattern: usize, styles: Styles) -> Self {
        let mut clusters = Vec::new();
        let mut cluster = CharCluster::default();
        let mut parser = Parser::new(
            SCRIPT,
            line.char_indices().map(|(i, char)| Token {
                ch: char,
                offset: i as u32,
                len: char.len_utf8() as u8,
                info: char.into(),
                data: Default::default(),
            }),
        );

        while parser.next(&mut cluster) {
            clusters.push(Cluster {
                cluster,
                pattern,
                styles,
            });
        }

        Self { clusters, styles }
    }

    pub fn clusters(&self) -> &[Cluster] {
        &self.clusters
    }

    pub fn clusters_mut(&mut self) -> &mut [Cluster] {
        &mut self.clusters
    }

    /// Shapes the text with `family` and `font_size`.
    ///
    /// No ligaturing will happen in `unligature_1` and `unligature_2`.
    pub fn shape(
        mut self,
        context: &mut Context,
        family: FontFamilyKey,
        font_size: FontSize,
        unligature_1: Option<RangeInclusive<usize>>,
        unligature_2: Option<RangeInclusive<usize>>,
    ) -> Line {
        struct Prev<'context> {
            shaper: Shaper<'context>,
            font: FontKey,
            size: FontSize,
        }

        impl<'context> Prev<'context> {
            fn flush(self, line: &mut Line, styles: Styles) {
                self.shaper.shape_with(|cluster| {
                    for glyph in cluster.glyphs {
                        line.glyphs.push(Glyph {
                            font: self.font,
                            size: self.size,
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

        let (fonts, shape, _) = context.as_muts();
        let emoji = fonts.emoji();
        let emoji_charmap = emoji.as_ref().charmap();
        let (unligature_1, unligature_2) = if let Some((unligature_1, unligature_2)) =
            unligature_1.clone().zip(unligature_2.clone())
        {
            let first_contains_second_start = unligature_1.contains(unligature_2.start());
            let first_contains_second_end = unligature_1.contains(unligature_2.end());
            let second_contains_first_start = unligature_2.contains(unligature_1.start());
            let second_contains_first_end = unligature_2.contains(unligature_1.end());

            if first_contains_second_start && first_contains_second_end {
                (Some(unligature_1), None)
            } else if second_contains_first_start && second_contains_first_end {
                (Some(unligature_2), None)
            } else if first_contains_second_start {
                (Some(*unligature_1.start()..=*unligature_2.end()), None)
            } else if second_contains_first_start {
                (Some(*unligature_2.start()..=*unligature_1.end()), None)
            } else {
                (Some(unligature_1), Some(unligature_2))
            }
        } else {
            (unligature_1, unligature_2)
        };

        let mut cache = HashMap::new();
        let mut prev = Option::<Prev>::None;
        let mut line = Line {
            glyphs: Vec::new(),
            font_size,
            advance: 0.,
        };

        // Shape the clusters reusing the shaper as long as the font is the same
        // (or forcing new shaper to unligature)
        for cluster in &mut self.clusters {
            let font = fonts
                .get((family, cluster.styles.weight, cluster.styles.style))
                .expect("Font not found in font cache");
            let (font_charmap, emoji_size) = *cache.entry(font.key()).or_insert_with(|| {
                (
                    font.as_ref().charmap(),
                    emoji.size_for_advance(2.0 * font.advance_for_size(font_size)),
                )
            });
            let selected = match cluster.cluster.map(|char| font_charmap.map(char)) {
                Status::Discard => {
                    cluster.cluster.map(|char| emoji_charmap.map(char));
                    emoji.key()
                }
                Status::Keep => match cluster.cluster.map(|char| emoji_charmap.map(char)) {
                    Status::Discard => {
                        cluster.cluster.map(|char| font_charmap.map(char));
                        font.key()
                    }
                    Status::Keep => emoji.key(),
                    Status::Complete => emoji.key(),
                },
                Status::Complete => font.key(),
            };
            let force_flush = {
                // Assuming clusters align "nicely" with unligature ranges
                let force_push_1 = {
                    if let Some(unligature) = unligature_1.clone() {
                        unligature.contains(&cluster.range().start)
                    } else {
                        false
                    }
                };
                let force_push_2 = {
                    if let Some(unligature) = unligature_2.clone() {
                        unligature.contains(&cluster.range().start)
                    } else {
                        false
                    }
                };

                force_push_1 || force_push_2
            };

            prev = Some(Prev {
                shaper: {
                    let shaper = prev.take().and_then(|prev| {
                        if !force_flush && prev.font == selected {
                            Some(prev.shaper)
                        } else {
                            prev.flush(&mut line, self.styles);
                            None
                        }
                    });
                    let mut shaper = if let Some(shaper) = shaper {
                        shaper
                    } else {
                        let (font, size) = match () {
                            _ if selected == font.key() => (font, font_size),
                            _ if selected == emoji.key() => (emoji, emoji_size),
                            _ => unreachable!(),
                        };

                        shape
                            .builder(font.as_ref())
                            .script(SCRIPT)
                            .size(size as f32)
                            .features(FEATURES)
                            .build()
                    };

                    shaper.add_cluster(&cluster.cluster);
                    shaper
                },
                font: selected,
                size: match () {
                    _ if selected == font.key() => font_size,
                    _ if selected == emoji.key() => emoji_size,
                    _ => unreachable!(),
                },
            });
        }

        // Flush last shaper
        if let Some(prev) = prev {
            prev.flush(&mut line, self.styles);
        }

        // Please tell me those ranges are monotonic
        debug_assert!(self
            .clusters
            .windows(2)
            .all(|clusters| clusters[0].range().start <= clusters[1].range().start));
        debug_assert!(line
            .glyphs
            .windows(2)
            .all(|glyphs| glyphs[0].range.start <= glyphs[1].range.start));

        // Transfer styles from clusters to glyphs
        let mut glyphs = line.glyphs.iter_mut().peekable();

        for cluster in &self.clusters {
            // Clusters transfer their styles from the end of the previous one to their own end
            while let Some(glyph) =
                glyphs.next_if(|glyph| (glyph.range.start as usize) < cluster.range().end)
            {
                glyph.styles = cluster.styles;
            }
        }

        // Last cluster transfers styles until the end
        if let Some(cluster) = self.clusters.last() {
            for glyph in glyphs {
                glyph.styles = cluster.styles;
            }
        }

        line
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
