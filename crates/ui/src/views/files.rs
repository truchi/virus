use crate::{panes::Panes, theme::UiTheme, views::DocumentView, Context};
use swash::{scale::ScaleContext, shape::ShapeContext};
use virus_editor::{editor::Editor, files::EditorFiles, mode::Mode};
use virus_graphics::{
    geom::{Position, Rectangle, Size},
    gpu::Gpu,
    text::{FontWeight, Fonts, Glyphs, Styles},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FilesView                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct FilesView {}

impl FilesView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self, context: &mut Context, region: Rectangle, editor: &Editor) {
        if editor.mode() != Mode::Files {
            return;
        }

        Renderer {
            fonts: &context.fonts,
            shape: &mut context.shape,
            scale: &mut context.scale,
            theme: &context.theme,
            gpu: &mut context.gpu,
            region,
            files: editor.files(),
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
    gpu: &'a mut Gpu,
    region: Rectangle,
    files: EditorFiles<'a>,
}

impl<'a> Renderer<'a> {
    fn centered(&self, margin: u32) -> Rectangle {
        self.region.centered(Size::new(
            (margin + Panes::ACTIVE_COLUMNS + DocumentView::GUTTER_COLUMNS) * self.theme.advance,
            self.region.height,
        ))
    }

    fn background(&mut self) -> &mut Self {
        self.gpu
            .draw(self.region)
            .fill(self.theme.inactive_foreground_color);
        self.gpu
            .draw(self.centered(2))
            .fill(self.theme.background_color.transparent(255));

        self
    }

    fn needle(&mut self) -> &mut Self {
        let glyphs = Glyphs::shaper(
            self.fonts,
            self.shape,
            self.theme.family,
            self.theme.font_size,
            self.theme.advance,
        )
        .push(
            self.files.needle(),
            Styles {
                weight: FontWeight::Bold,
                ..self.theme.syntax.default
            },
        )
        .glyphs();

        // Needle
        self.gpu.draw(self.centered(0)).glyphs(
            self.fonts,
            self.scale,
            Position::new(self.theme.line_height as i32, 0),
            self.theme.line_height,
            &glyphs,
        );

        // Caret
        let left = glyphs.advance() as i32 - self.theme.caret_width as i32 / 2;

        if left <= self.centered(0).width() as i32 {
            self.gpu.draw(self.centered(2)).rectangle(
                Rectangle::new(
                    self.theme.line_height as i32,
                    self.theme.advance as i32 + left,
                    self.theme.caret_width,
                    self.theme.line_height,
                ),
                self.theme.insert_mode_color.transparent(255),
            );
        }

        self
    }

    fn matches(&mut self) -> &mut Self {
        let selected = self.files.selected_index();
        let region = {
            let mut region = self.centered(0);
            region.top += 3 * self.theme.line_height as i32;
            region.height -= 3 * self.theme.line_height;
            region
        };
        let range = {
            let region_height_in_lines = (region.height / self.theme.line_height) as usize;

            if selected < region_height_in_lines {
                0..region_height_in_lines.min(self.files.matches().len())
            } else {
                selected + 1 - region_height_in_lines..selected + 1
            }
        };
        let mut position = Position::default();

        for (index, m) in self.files.matches()[range.clone()].iter().enumerate() {
            let str = self.files.haystack()[m.index].as_str();
            let weight = (index + range.start == selected)
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
                    self.theme.advance,
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

            self.gpu.draw(region).glyphs(
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
