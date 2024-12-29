use crate::{theme::UiTheme, Context};
use swash::{scale::ScaleContext, shape::ShapeContext};
use virus_editor::mode::Mode;
use virus_graphics::{
    gpu::Layer,
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
        layer: Layer<'a>,
        mode: Mode,
        keybindings: &'a [String],
        file_name: Option<(String, bool)>,
    ) {
        let context = context.as_mut();
        Renderer {
            fonts: context.fonts,
            shape: context.shape,
            scale: context.scale,
            theme: context.theme,
            mode,
            layer,
            keybindings,
            file_name,
        }
        .render();
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
    mode: Mode,
    keybindings: &'a [String],
    file_name: Option<(String, bool)>,
}

impl<'a> Renderer<'a> {
    fn render(&mut self) -> &mut Self {
        let mut shaper = Glyphs::shaper(
            self.fonts,
            self.shape,
            self.theme.family,
            self.theme.font_size,
        );

        let (string, background) = match self.mode {
            Mode::Normal { .. } => (format!(" NORMAL "), self.theme.normal_mode_color),
            Mode::Insert { .. } => (format!(" INSERT "), self.theme.insert_mode_color),
            Mode::Files => (format!(" FILES "), virus_graphics::color::Rgba::RED.solid()), // TODO
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

        for (i, key) in self.keybindings.iter().enumerate() {
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

        self.layer
            .draw(None, 0)
            .rectangle(None, self.theme.status_background_color.transparent(255));

        self.layer.draw(None, 0).glyphs(
            self.fonts,
            self.scale,
            Default::default(),
            self.theme.line_height,
            &shaper.glyphs(),
        );

        self
    }
}
