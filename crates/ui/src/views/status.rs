use crate::{theme::UiTheme, Context};
use swash::{scale::ScaleContext, shape::ShapeContext};
use virus_editor::{editor::Editor, mode::Mode};
use virus_graphics::{
    geom::{Position, Rectangle},
    gpu::Gpu,
    text::{FontWeight, Fonts, Glyphs, Styles},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           StatusView                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct StatusView {}

impl StatusView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<'a>(
        &mut self,
        context: &'a mut Context,
        region: Rectangle,
        editor: &Editor,
        keybindings: impl Iterator<Item = String>,
        file_name: Option<(String, bool)>,
    ) {
        Renderer {
            fonts: &context.fonts,
            shape: &mut context.shape,
            scale: &mut context.scale,
            theme: &context.theme,
            gpu: &mut context.gpu,
            region,
            mode: editor.mode(),
            keybindings,
            file_name,
        }
        .render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a, T> {
    fonts: &'a Fonts,
    shape: &'a mut ShapeContext,
    scale: &'a mut ScaleContext,
    theme: &'a UiTheme,
    gpu: &'a mut Gpu,
    region: Rectangle,
    mode: Mode,
    keybindings: T,
    file_name: Option<(String, bool)>,
}

impl<'a, T: Iterator<Item = String>> Renderer<'a, T> {
    fn render(&mut self) {
        let mut shaper = Glyphs::shaper(
            self.fonts,
            self.shape,
            self.theme.family,
            self.theme.font_size,
            self.theme.advance,
        );

        let (string, background) = match self.mode {
            Mode::Normal { .. } => (format!(" NORMAL "), self.theme.normal_mode_color),
            Mode::Insert { .. } => (format!(" INSERT "), self.theme.insert_mode_color),
            Mode::Files => (format!(" FILES "), virus_graphics::color::Rgb::RED), // TODO
        };

        shaper.push(
            &string,
            Styles {
                foreground: self.theme.status_mode_foreground_color.transparent(255),
                background: background.transparent(255),
                weight: FontWeight::Bold,
                ..self.theme.syntax.default
            },
        );

        for (i, key) in (&mut self.keybindings).enumerate() {
            let space = (i == 0).then_some(" ").unwrap_or_default();

            shaper.push(
                &format!("{space}{key} "),
                Styles {
                    foreground: self.theme.status_mode_foreground_color.transparent(255),
                    background: background.transparent(255 / 2),
                    ..self.theme.syntax.default
                },
            );
        }

        if let Some((file_name, is_dirty)) = &self.file_name {
            shaper.push(
                &format!(" {file_name} "),
                Styles {
                    foreground: self.theme.status_file_foreground_color.transparent(255),
                    weight: is_dirty.then_some(FontWeight::Bold).unwrap_or_default(),
                    ..Default::default()
                },
            );
        }

        self.gpu
            .draw(self.region)
            .fill(self.theme.status_background_color.transparent(255))
            .glyphs(
                self.fonts,
                self.scale,
                Position::default(),
                self.theme.line_height,
                &shaper.glyphs(),
            );
    }
}
