use crate::{
    tween::{Tween, Tweened},
    views::{DocumentView, FilesView},
};
use std::{sync::Arc, time::Duration};
use virus_common::{Rectangle, Rgba};
use virus_editor::{
    document::{Document, Selection},
    mode::SelectMode,
    theme::Theme,
};
use virus_graphics::{
    text::{Context, Font, FontSize, FontStyle, FontWeight, Fonts, LineHeight},
    wgpu::Graphics,
};
use winit::window::Window;

const SCROLL_DURATION: Duration = Duration::from_millis(500);
const SCROLL_TWEEN: Tween = Tween::ExpoOut;

const FONT_SIZE: FontSize = 24;
const LINE_HEIGHT: LineHeight = 30;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                 Ui                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Ui {
    window: Arc<Window>,
    graphics: Graphics,
    context: Context,
    document_view: DocumentView,
    scroll_top: Tweened<u32>,
    scrollbar_alpha: Tweened<u8>,
    _files_view: FilesView,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let graphics = Graphics::new(&window);
        let context = Context::new(fonts());
        let family = context.fonts().get("Victor").unwrap();

        let document_view = DocumentView::new(
            family.key(),
            Theme::catppuccin_latte(),
            FONT_SIZE,
            LINE_HEIGHT,
        );
        let _files_view = FilesView::new(family.key(), FONT_SIZE, LINE_HEIGHT, Rgba::BLACK);

        Self {
            window,
            graphics,
            context,
            document_view,
            scroll_top: Tweened::new(0),
            scrollbar_alpha: Tweened::new(0),
            _files_view,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn is_animating(&self) -> bool {
        self.scroll_top.is_animating() || self.scrollbar_alpha.is_animating()
    }

    pub fn scroll_up(&mut self) {
        let scroll = self.screen_height_in_lines() / 2 * self.document_view.line_height();
        self.scroll_to(self.scroll_top.end().saturating_sub(scroll))
    }

    pub fn scroll_down(&mut self) {
        let line_height = self.document_view.line_height();
        let rope_lines = self.document_view.rope().len_lines() as u32 - 1;
        let screen_height_in_lines = self.screen_height_in_lines();

        if rope_lines > screen_height_in_lines {
            let end = self.scroll_top.end() + screen_height_in_lines / 2 * line_height;
            self.scroll_to(end.min((rope_lines - screen_height_in_lines) * line_height));
        }
    }

    pub fn ensure_visibility(&mut self, selection: Selection) {
        let line_height = self.document_view.line_height();
        let screen_height_in_lines = self.screen_height_in_lines();
        let line = selection.range().start.line as u32;
        let start = self.scroll_top.end() / line_height;
        let end = start + screen_height_in_lines;

        if line < start {
            self.scroll_to(line * line_height);
        } else if line >= end {
            self.scroll_to((line - screen_height_in_lines + 1) * line_height);
        }
    }

    pub fn resize(&mut self) {
        self.graphics.resize(&self.window);
    }

    pub fn update(&mut self, delta: Duration) {
        self.scroll_top.step(delta);
        self.scrollbar_alpha.step(delta);
    }

    pub fn render(&mut self, document: &Document, select_mode: Option<SelectMode>) {
        let region = self.region();

        self.document_view.render(
            &mut self.context,
            &mut self.graphics.layer(0, region),
            document,
            select_mode,
            self.scroll_top.current(),
            self.scrollbar_alpha.current(),
        );
        // self.files_view
        //     .render(&mut self.context, &mut self.graphics.layer(1, region));

        self.graphics.render();
    }
}

/// Private.
impl Ui {
    fn screen_height_in_lines(&self) -> u32 {
        self.window.inner_size().height / self.document_view.line_height()
    }

    fn region(&self) -> Rectangle {
        let size = self.window.inner_size();
        let height = self.screen_height_in_lines() * self.document_view.line_height();

        Rectangle {
            top: (size.height - height) as i32 / 2,
            left: 0,
            width: size.width,
            height,
        }
    }

    fn scroll_to(&mut self, scroll_top: u32) {
        self.scroll_top
            .to(scroll_top, SCROLL_DURATION, SCROLL_TWEEN);
        self.scrollbar_alpha = Tweened::with_animation(255, 0, SCROLL_DURATION, SCROLL_TWEEN);
    }
}

fn fonts() -> Fonts {
    use virus_graphics::text::{FontStyle::*, FontWeight::*};

    const EMOJI: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";
    const FOLDER: &str = "/Users/romain/Library/Fonts/";
    const FONTS: &[(&str, &[(&str, FontWeight, FontStyle)])] = &[
        //
        (
            "Victor",
            &[
                //
                ("VictorMono-Thin.ttf", Thin, Normal),
                ("VictorMono-ExtraLight.ttf", ExtraLight, Normal),
                ("VictorMono-Light.ttf", Light, Normal),
                ("VictorMono-Regular.ttf", Regular, Normal),
                ("VictorMono-Medium.ttf", Medium, Normal),
                ("VictorMono-SemiBold.ttf", SemiBold, Normal),
                ("VictorMono-Bold.ttf", Bold, Normal),
                //
                ("VictorMono-ThinItalic.ttf", Thin, Italic),
                ("VictorMono-ExtraLightItalic.ttf", ExtraLight, Italic),
                ("VictorMono-LightItalic.ttf", Light, Italic),
                ("VictorMono-Italic.ttf", Regular, Italic),
                ("VictorMono-MediumItalic.ttf", Medium, Italic),
                ("VictorMono-SemiBoldItalic.ttf", SemiBold, Italic),
                ("VictorMono-BoldItalic.ttf", Bold, Italic),
                //
                ("VictorMono-ThinOblique.ttf", Thin, Oblique),
                ("VictorMono-ExtraLightOblique.ttf", ExtraLight, Oblique),
                ("VictorMono-LightOblique.ttf", Light, Oblique),
                ("VictorMono-Oblique.ttf", Regular, Oblique),
                ("VictorMono-MediumOblique.ttf", Medium, Oblique),
                ("VictorMono-SemiBoldOblique.ttf", SemiBold, Oblique),
                ("VictorMono-BoldOblique.ttf", Bold, Oblique),
            ],
        ),
    ];

    let mut fonts = Fonts::new(Font::from_file(EMOJI).unwrap());

    for (family, faces) in FONTS {
        let family = fonts.set(String::from(*family)).unwrap();

        for (font, font_weight, font_style) in *faces {
            let font = fonts
                .set(Font::from_file(String::from(FOLDER) + font).unwrap())
                .unwrap();

            fonts
                .set((family, *font_weight, *font_style, font))
                .unwrap();
        }
    }

    fonts
}
