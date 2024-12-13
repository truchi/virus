use crate::{panes::Panes, theme::UiTheme, views::DocumentView, Context};
use swash::{scale::ScaleContext, shape::ShapeContext};
use virus_editor::fuzzy::Search;
use virus_graphics::{
    text::{FontWeight, Fonts, Glyphs, Styles},
    types::{Position, Rectangle},
    wgpu::Layer,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FilesView                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct FilesView {}

impl FilesView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(
        &mut self,
        context: &mut Context,
        layer: Layer,
        selected: usize,
        needle: &str,
        search: &Search,
    ) {
        let context = context.as_mut();
        let region = {
            let columns = 2 + Panes::ACTIVE_COLUMNS + DocumentView::GUTTER_COLUMNS;
            let width = (columns as f32 * context.theme.advance).ceil() as u32;
            let left = (layer.size().width.saturating_sub(width) / 2) as i32;

            Rectangle {
                top: 0,
                left,
                width,
                height: layer.size().height,
            }
        };

        Renderer {
            fonts: context.fonts,
            shape: context.shape,
            scale: context.scale,
            theme: context.theme,
            layer,
            selected,
            needle,
            search,
            region,
        }
        .background()
        .needle()
        .matches();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a> {
    fonts: &'a Fonts,
    shape: &'a mut ShapeContext,
    scale: &'a mut ScaleContext,
    theme: &'a UiTheme,
    layer: Layer<'a>,
    selected: usize,
    needle: &'a str,
    search: &'a Search,
    region: Rectangle,
}

impl<'a> Renderer<'a> {
    fn background(&mut self) -> &mut Self {
        self.layer
            .draw(None, 0)
            .rectangle(None, self.theme.inactive_foreground_color);
        self.layer
            .draw(self.region, 0)
            .rectangle(None, self.theme.background_color.transparent(255));

        self
    }

    fn needle(&mut self) -> &mut Self {
        let glyphs = Glyphs::shaper(
            self.fonts,
            self.shape,
            self.theme.family,
            self.theme.font_size,
        )
        .push(
            &self.needle,
            Styles {
                weight: FontWeight::Bold,
                ..self.theme.syntax.default
            },
        )
        .glyphs();

        self.layer
            .draw(
                {
                    let mut region = self.region;
                    region.top += self.theme.line_height as i32;
                    region.left += self.theme.advance.ceil() as i32;
                    region
                },
                0,
            )
            .glyphs(
                self.fonts,
                self.scale,
                Position::default(),
                self.theme.line_height,
                &glyphs,
            );

        self.layer
            .draw(
                {
                    let mut region = self.region;
                    region.top += self.theme.line_height as i32;
                    region
                },
                0,
            )
            .rectangle(
                Rectangle {
                    top: 0,
                    left: (self.theme.advance + glyphs.advance()).ceil() as i32
                        - self.theme.caret_width as i32 / 2,
                    width: self.theme.caret_width,
                    height: self.theme.line_height,
                },
                self.theme.insert_mode_color.transparent(255),
            );

        self
    }

    fn matches(&mut self) -> &mut Self {
        if self.search.matches().is_empty() {
            return self;
        }

        let region = {
            let mut region = self.region;
            region.top += 3 * self.theme.line_height as i32;
            region.left += self.theme.advance.ceil() as i32;
            region
        };
        let range = {
            let region_height_in_lines = (region.height / self.theme.line_height) as usize;

            if self.selected < region_height_in_lines {
                0..region_height_in_lines.min(self.search.matches().len())
            } else {
                self.selected + 1 - region_height_in_lines..self.selected + 1
            }
        };
        let mut position = Position::default();

        for (index, m) in self.search.matches()[range.clone()].iter().enumerate() {
            let str = self.search.haystack()[m.index].as_str();
            let weight = (index + range.start == self.selected)
                .then_some(FontWeight::Bold)
                .unwrap_or_default();
            let styles0 = Styles {
                weight,
                ..self.theme.syntax.default
            };
            let styles1 = Styles {
                weight,
                foreground: self.theme.insert_mode_color.transparent(255),
                ..self.theme.syntax.default
            };
            let glyphs = {
                let mut index = 0;
                let mut shaper = Glyphs::shaper(
                    self.fonts,
                    self.shape,
                    self.theme.family,
                    self.theme.font_size,
                );

                for range in &m.indices {
                    if index < range.start {
                        shaper.push(&str[index..range.start], styles0);
                    }

                    shaper.push(&str[range.clone()], styles1);
                    index = range.end;
                }

                if index < str.len() {
                    shaper.push(&str[index..], styles0);
                }

                shaper.glyphs()
            };

            self.layer.draw(region, 0).glyphs(
                self.fonts,
                self.scale,
                position,
                self.theme.line_height,
                &glyphs,
            );

            position.top += self.theme.line_height as i32;
        }

        self
    }
}
