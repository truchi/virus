use crate::{
    syntax::SyntaxTheme,
    theme::UiTheme,
    views::{DocumentView, FilesView},
};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    ops::Range,
    rc::Rc,
    sync::Arc,
    time::Duration,
};
use virus_editor::document::{Document, DocumentId};
use virus_graphics::{
    text::{Context, Font, FontStyle, FontWeight, Fonts},
    types::{Rectangle, Rgba},
    wgpu::Graphics,
    Catppuccin,
};
use winit::window::Window;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                 Ui                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Ui {
    window: Arc<Window>,
    graphics: Graphics,
    context: Context,
    theme: Rc<RefCell<UiTheme>>,
    document: DocumentId,
    documents: HashMap<DocumentId, DocumentView>,
    files: FilesView,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let graphics = Graphics::new(Arc::clone(&window));
        let context = Context::new(fonts());
        let theme = Rc::new(RefCell::new({
            let catppuccin = Catppuccin::default();
            let normal_mode = catppuccin.blue;
            let select_mode = catppuccin.pink;
            let insert_mode = catppuccin.green;

            UiTheme {
                syntax: SyntaxTheme::catppuccin(),
                family: context.fonts().get("Victor").unwrap().key(),
                font_size: 20,
                line_height: 25,
                scrollbar_color: catppuccin.surface1.solid(),
                scroll_duration: Duration::from_millis(500),
                scroll_tween: crate::tween::Tween::ExpoOut,
                outline_normal_mode_colors: vec![
                    normal_mode.solid().transparent(255 / 4),
                    normal_mode.solid().transparent(255 / 6),
                    normal_mode.solid().transparent(255 / 8),
                    normal_mode.solid().transparent(255 / 10),
                ],
                outline_select_mode_colors: vec![
                    select_mode.solid().transparent(255 / 4),
                    select_mode.solid().transparent(255 / 6),
                    select_mode.solid().transparent(255 / 8),
                    select_mode.solid().transparent(255 / 10),
                ],
                outline_insert_mode_colors: vec![
                    insert_mode.solid().transparent(255 / 4),
                    insert_mode.solid().transparent(255 / 6),
                    insert_mode.solid().transparent(255 / 8),
                    insert_mode.solid().transparent(255 / 10),
                ],
                caret_normal_mode_color: normal_mode,
                caret_select_mode_color: select_mode,
                caret_insert_mode_color: insert_mode,
                caret_normal_mode_width: 2,
                caret_select_mode_width: 2,
                caret_insert_mode_width: 2,
                selection_select_mode_color: select_mode.solid().transparent(255 / 2),
                selection_insert_mode_color: insert_mode.solid().transparent(255 / 2),
            }
        }));
        let files = FilesView::new(theme.clone(), Rgba::WHITE);

        Self {
            window,
            graphics,
            context,
            theme,
            document: DocumentId::NONE,
            documents: Default::default(),
            files,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn theme(&self) -> Ref<UiTheme> {
        self.theme.borrow()
    }

    pub fn is_animating(&self) -> bool {
        self.documents.values().any(|view| view.is_animating())
    }

    pub fn screen_height_in_lines(&self) -> u32 {
        self.window.inner_size().height / self.theme().line_height
    }

    pub fn scroll_up(&mut self) {
        self.documents
            .get_mut(&self.document)
            .map(|view| view.scroll_up());
    }

    pub fn scroll_down(&mut self) {
        self.documents
            .get_mut(&self.document)
            .map(|view| view.scroll_down());
    }

    pub fn scroll_to(&mut self, top: u32) {
        self.documents
            .get_mut(&self.document)
            .map(|views| views.scroll_to(top));
    }

    pub fn ensure_visibility(&mut self, line: usize) {
        self.documents
            .get_mut(&self.document)
            .map(|views| views.ensure_visibility(line));
    }

    pub fn resize(&mut self) {
        let size = self.region().size();

        self.graphics.resize(&self.window);
        self.documents
            .values_mut()
            .for_each(|views| views.resize(size));
    }

    pub fn update(&mut self, delta: Duration) {
        self.documents
            .values_mut()
            .for_each(|views| views.update(delta));
    }

    pub fn render<'a>(
        &mut self,
        document: &Document,
        show_selection_as_lines: bool,
        outline_colors: &[Rgba],
        caret_color: Rgba,
        caret_width: u32,
        selection_color: Rgba,
        search: Option<(&'a str, &'a [(String, isize, Vec<Range<usize>>)], usize)>,
    ) {
        let region = self.region();

        self.document = document.id();
        let view = self
            .documents
            .entry(self.document)
            .or_insert_with(|| DocumentView::new(self.theme.clone()));
        view.resize(region.size());
        view.render(
            &mut self.context,
            &mut self.graphics.layer(region, 0),
            document,
            show_selection_as_lines,
            outline_colors,
            caret_color,
            caret_width,
            selection_color,
        );

        if let Some((needle, haystack, selected)) = search {
            self.files.render(
                &mut self.context,
                self.graphics.layer(region, 1),
                needle,
                haystack,
                selected,
            );
        }

        self.graphics.render();
    }
}

/// Private.
impl Ui {
    fn region(&self) -> Rectangle {
        let size = self.window.inner_size();
        let height = self.screen_height_in_lines() * self.theme().line_height;

        Rectangle {
            top: (size.height - height) as i32 / 2,
            left: 0,
            width: size.width,
            height,
        }
    }
}

fn fonts() -> Fonts {
    use virus_graphics::text::{FontStyle::*, FontWeight::*};

    const EMOJI: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";
    const FOLDER: &str = "/Users/romain/Library/Fonts/";
    const FONTS: &[(&str, &[(&str, FontWeight, FontStyle)])] = &[
        (
            "Victor",
            &[
                // Normal
                ("VictorMono-Thin.ttf", Thin, Normal),
                ("VictorMono-ExtraLight.ttf", ExtraLight, Normal),
                ("VictorMono-Light.ttf", Light, Normal),
                ("VictorMono-Regular.ttf", Regular, Normal),
                ("VictorMono-Medium.ttf", Medium, Normal),
                ("VictorMono-SemiBold.ttf", SemiBold, Normal),
                ("VictorMono-Bold.ttf", Bold, Normal),
                // Italic
                ("VictorMono-ThinItalic.ttf", Thin, Italic),
                ("VictorMono-ExtraLightItalic.ttf", ExtraLight, Italic),
                ("VictorMono-LightItalic.ttf", Light, Italic),
                ("VictorMono-Italic.ttf", Regular, Italic),
                ("VictorMono-MediumItalic.ttf", Medium, Italic),
                ("VictorMono-SemiBoldItalic.ttf", SemiBold, Italic),
                ("VictorMono-BoldItalic.ttf", Bold, Italic),
                // Oblique
                ("VictorMono-ThinOblique.ttf", Thin, Oblique),
                ("VictorMono-ExtraLightOblique.ttf", ExtraLight, Oblique),
                ("VictorMono-LightOblique.ttf", Light, Oblique),
                ("VictorMono-Oblique.ttf", Regular, Oblique),
                ("VictorMono-MediumOblique.ttf", Medium, Oblique),
                ("VictorMono-SemiBoldOblique.ttf", SemiBold, Oblique),
                ("VictorMono-BoldOblique.ttf", Bold, Oblique),
            ],
        ),
        (
            "JetBrains",
            &[
                // Normal
                ("JetBrainsMonoNerdFont-Thin.ttf", Thin, Normal),
                ("JetBrainsMonoNerdFont-ExtraLight.ttf", ExtraLight, Normal),
                ("JetBrainsMonoNerdFont-Light.ttf", Light, Normal),
                ("JetBrainsMonoNerdFont-Regular.ttf", Regular, Normal),
                ("JetBrainsMonoNerdFont-Medium.ttf", Medium, Normal),
                ("JetBrainsMonoNerdFont-SemiBold.ttf", SemiBold, Normal),
                ("JetBrainsMonoNerdFont-Bold.ttf", Bold, Normal),
                ("JetBrainsMonoNerdFont-ExtraBold.ttf", ExtraBold, Normal),
                //
                ("JetBrainsMonoNerdFont-ThinItalic.ttf", Thin, Italic),
                (
                    "JetBrainsMonoNerdFont-ExtraLightItalic.ttf",
                    ExtraLight,
                    Italic,
                ),
                ("JetBrainsMonoNerdFont-LightItalic.ttf", Light, Italic),
                ("JetBrainsMonoNerdFont-Italic.ttf", Regular, Italic),
                ("JetBrainsMonoNerdFont-MediumItalic.ttf", Medium, Italic),
                ("JetBrainsMonoNerdFont-SemiBoldItalic.ttf", SemiBold, Italic),
                ("JetBrainsMonoNerdFont-BoldItalic.ttf", Bold, Italic),
                (
                    "JetBrainsMonoNerdFont-ExtraBoldItalic.ttf",
                    ExtraBold,
                    Italic,
                ),
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
