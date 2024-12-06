use crate::{panes::Panes, theme::UiTheme, views::DocumentView};
use std::{cell::RefCell, rc::Weak, usize};
use virus_editor::fuzzy::Search;
use virus_graphics::{
    text::{Context, FontWeight, Line, Styles},
    types::{Position, Rectangle},
    wgpu::Layer,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FilesView                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct FilesView {
    theme: Weak<RefCell<UiTheme>>,
}

impl FilesView {
    pub fn new(theme: Weak<RefCell<UiTheme>>) -> Self {
        Self { theme }
    }

    pub fn render<'a>(
        &mut self,
        context: &'a mut Context,
        layer: Layer<'a>,
        selected: usize,
        needle: &'a str,
        search: &'a Search,
    ) {
        let theme = *self.theme.upgrade().unwrap().borrow();

        Renderer::new(context, layer, theme, selected, needle, search).render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a> {
    context: &'a mut Context,
    layer: Layer<'a>,
    theme: UiTheme,
    selected: usize,
    needle: &'a str,
    search: &'a Search,
    region: Rectangle,
}

impl<'a> Renderer<'a> {
    fn new(
        context: &'a mut Context,
        layer: Layer<'a>,
        theme: UiTheme,
        selected: usize,
        needle: &'a str,
        search: &'a Search,
    ) -> Self {
        let region = {
            let columns = 2 + Panes::ACTIVE_COLUMNS + DocumentView::GUTTER_COLUMNS;
            let width = (columns as f32 * theme.advance).ceil() as u32;
            let left = (layer.size().width.saturating_sub(width) / 2) as i32;

            Rectangle {
                top: 0,
                left,
                width,
                height: layer.size().height,
            }
        };

        Self {
            context,
            layer,
            theme,
            selected,
            needle,
            search,
            region,
        }
    }

    fn render(&mut self) {
        self.render_background();
        self.render_needle();
        self.render_matches();
    }

    fn render_background(&mut self) {
        self.layer
            .draw(None, 0)
            .rectangle(None, self.theme.inactive_foreground_color);
        self.layer
            .draw(self.region, 0)
            .rectangle(None, self.theme.background_color.transparent(255));
    }

    fn render_needle(&mut self) {
        let line = Line::shaper(
            &self.needle,
            0,
            Styles {
                weight: FontWeight::Bold,
                ..self.theme.syntax.default
            },
        )
        .shape(self.context, self.theme.family, self.theme.font_size);

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
                self.context,
                Position::default(),
                &line,
                self.theme.line_height,
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
                    left: (self.theme.advance + line.advance()).ceil() as i32
                        - self.theme.caret_width as i32 / 2,
                    width: self.theme.caret_width,
                    height: self.theme.line_height,
                },
                self.theme.insert_mode_color.transparent(255),
            );
    }

    fn render_matches(&mut self) {
        if self.search.matches().is_empty() {
            return;
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
            let index = index + range.start;

            let weight = if index == self.selected {
                FontWeight::Bold
            } else {
                Default::default()
            };
            let mut shaper = Line::shaper(
                &self.search.haystack()[m.index],
                0,
                Styles {
                    weight,
                    ..self.theme.syntax.default
                },
            );

            let mut clusters = shaper.clusters_mut();

            for range in &m.indices {
                let mut start = 0;

                for (i, cluster) in clusters
                    .iter_mut()
                    .enumerate()
                    .skip_while(|(_, cluster)| !cluster.range().contains(&range.start))
                    .take_while(|(_, cluster)| !cluster.range().contains(&range.end))
                {
                    *cluster.styles_mut() = Styles {
                        weight,
                        foreground: self.theme.insert_mode_color.transparent(255),
                        ..Default::default()
                    };
                    start = i;
                }

                clusters = &mut clusters[start..];
            }

            let line = shaper.shape(self.context, self.theme.family, self.theme.font_size);
            self.layer.draw(region, 0).glyphs(
                self.context,
                position,
                &line,
                self.theme.line_height,
            );

            position.top += self.theme.line_height as i32;
        }
    }
}
