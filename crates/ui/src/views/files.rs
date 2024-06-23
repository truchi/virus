#![allow(unused)]

use std::{
    ops::{Range, RangeBounds},
    usize,
};
use virus_graphics::{
    text::{
        Context, FontFamilyKey, FontKey, FontSize, FontStyle, FontWeight, Line, LineHeight, Styles,
    },
    types::{Position, Rectangle, Rgba},
    wgpu::{Draw, Layer},
};

const MIN_WIDTH: f32 = 0.5;

fn min_width(width: u32) -> u32 {
    (width as f32 * MIN_WIDTH).round() as u32
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FilesView                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct FilesView {
    needle: String,
    haystack: Vec<(String, Vec<Range<usize>>)>,
    selected: usize,
    family: FontFamilyKey,
    font_size: FontSize,
    line_height: LineHeight,
    background: Rgba,
}

impl FilesView {
    pub fn new(
        family: FontFamilyKey,
        font_size: FontSize,
        line_height: LineHeight,
        background: Rgba,
    ) -> Self {
        Self {
            needle: Default::default(),
            haystack: Default::default(),
            selected: 0,
            family,
            font_size,
            line_height,
            background,
        }
    }

    pub fn render<'a>(
        &mut self,
        context: &'a mut Context,
        layer: Layer<'a>,
        needle: &'a str,
        haystacks: &'a [(String, isize, Vec<Range<usize>>)],
        selected: usize,
    ) {
        Renderer::new(
            context,
            layer,
            self.background,
            self.family,
            self.font_size,
            self.line_height,
            needle,
            haystacks,
            selected,
        )
        .render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a> {
    context: &'a mut Context,
    layer: Layer<'a>,
    background: Rgba,
    family: FontFamilyKey,
    font_size: FontSize,
    line_height: LineHeight,
    advance: f32,
    needle: &'a str,
    haystacks: &'a [(String, isize, Vec<Range<usize>>)],
    selected: usize,
}

impl<'a> Renderer<'a> {
    fn new(
        context: &'a mut Context,
        layer: Layer<'a>,
        background: Rgba,
        family: FontFamilyKey,
        font_size: FontSize,
        line_height: LineHeight,
        needle: &'a str,
        haystacks: &'a [(String, isize, Vec<Range<usize>>)],
        selected: usize,
    ) -> Self {
        let advance = context
            .fonts()
            .get(
                context
                    .fonts()
                    .get(family)
                    .unwrap()
                    .best_match_regular_normal()
                    .unwrap(),
            )
            .unwrap()
            .advance_for_size(font_size);

        Self {
            context,
            layer,
            background,
            family,
            font_size,
            line_height,
            advance,
            needle,
            haystacks,
            selected,
        }
    }

    fn render(&mut self) {
        self.render_background();
        self.render_needle();
        self.render_haystacks();
    }

    fn render_background(&mut self) {
        self.layer.draw(None, 0).rectangle(None, self.background);
    }

    fn render_needle(&mut self) {
        let line = Line::shaper(
            &self.needle,
            0,
            Styles {
                weight: Default::default(),
                style: Default::default(),
                foreground: Rgba::BLACK,
                background: Default::default(),
                underline: false,
                strike: false,
            },
        )
        .shape(self.context, self.family, self.font_size, None, None);

        self.layer
            .draw(None, 0)
            .glyphs(self.context, Position::default(), &line, self.line_height);
    }

    fn render_haystacks(&mut self) {
        if self.haystacks.is_empty() {
            return;
        }

        let region = Rectangle {
            top: self.line_height as i32,
            left: 0,
            width: self.layer.size().width,
            height: self.layer.size().height - self.line_height,
        };
        let range = {
            let region_height_in_lines = (region.height / self.line_height) as usize;

            if self.selected < region_height_in_lines {
                0..region_height_in_lines.min(self.haystacks.len())
            } else {
                self.selected + 1 - region_height_in_lines..self.selected + 1
            }
        };
        let mut position = Position::default();

        for (index, (haystack, _, indices)) in self.haystacks[range.clone()].iter().enumerate() {
            let index = index + range.start;

            let weight = if index == self.selected {
                FontWeight::Bold
            } else {
                Default::default()
            };
            let mut shaper = Line::shaper(
                haystack,
                0,
                Styles {
                    weight,
                    style: Default::default(),
                    foreground: Rgba::BLACK,
                    background: Default::default(),
                    underline: false,
                    strike: false,
                },
            );

            let mut clusters = shaper.clusters_mut();

            for range in indices {
                let mut start = 0;

                for (i, cluster) in clusters
                    .iter_mut()
                    .enumerate()
                    .skip_while(|(_, cluster)| !cluster.range().contains(&range.start))
                    .take_while(|(_, cluster)| !cluster.range().contains(&range.end))
                {
                    *cluster.styles_mut() = Styles {
                        weight,
                        style: Default::default(),
                        foreground: Rgba::RED,
                        background: Default::default(),
                        underline: false,
                        strike: false,
                    };
                    start = i;
                }

                clusters = &mut clusters[start..];
            }

            let line = shaper.shape(self.context, self.family, self.font_size, None, None);
            self.layer
                .draw(region, 0)
                .glyphs(self.context, position, &line, self.line_height);

            position.top += self.line_height as i32;
        }
    }
}
