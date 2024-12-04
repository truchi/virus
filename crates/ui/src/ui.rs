use crate::{
    panes::{DocumentPane, Pane, PaneId, Panes},
    syntax::{Lines, SyntaxTheme},
    theme::UiTheme,
    tween::Tween,
    views::FilesView,
};
use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, sync::Arc, time::Duration};
use virus_editor::{
    document::{Document, DocumentId},
    mode::Mode,
};
use virus_graphics::{
    text::{Context, Font, FontSize, FontStyle, FontWeight, Fonts, LineHeight},
    types::{Rectangle, Rgba, Size},
    wgpu::Graphics,
    Catppuccin,
};
use winit::window::Window;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                 Ui                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub(crate) type LinesCache = HashMap<DocumentId, Lines>;

pub struct Ui {
    window: Arc<Window>,
    graphics: Graphics,
    context: Context,
    theme: Rc<RefCell<UiTheme>>,
    panes: Panes,
    files: FilesView,
    region: Rectangle,
    cells: Size,
    _lines_cache: Rc<RefCell<LinesCache>>,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let graphics = Graphics::new(Arc::clone(&window));
        let context = Context::new(fonts());
        let theme = Rc::new(RefCell::new(ui_theme(&context)));
        let lines_cache = Default::default();
        let files = FilesView::new(Rc::downgrade(&theme), Rgba::WHITE);
        let panes = Panes::new(Rc::downgrade(&theme), Rc::downgrade(&lines_cache));

        Self {
            window,
            graphics,
            context,
            theme,
            panes,
            files,
            region: Default::default(),
            cells: Default::default(),
            _lines_cache: lines_cache,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn theme(&self) -> UiTheme {
        *self.theme.borrow()
    }

    pub fn region(&self) -> Rectangle {
        self.region
    }

    pub fn cells(&self) -> Size {
        self.cells
    }

    pub fn is_animating(&self) -> bool {
        self.panes.is_animating()
    }

    pub fn pane(&self, pane_id: PaneId) -> Option<&Pane> {
        self.panes.get(pane_id)
    }

    pub fn scroll_to(&mut self, pane_id: PaneId, line: u32) {
        let top = line * self.theme().line_height;

        self.panes.get_mut(pane_id).map(|pane| match pane {
            Pane::Document(DocumentPane { document_view, .. }) => document_view.scroll_to(top),
        });
    }

    pub fn open_pane(&mut self, document_id: DocumentId) -> PaneId {
        self.panes.open_pane(document_id)
    }

    pub fn get_active_pane_id(&self) -> Option<PaneId> {
        self.panes.get_active_pane_id()
    }

    pub fn set_active_pane_id(&mut self, active_pane_id: Option<PaneId>) {
        self.panes.set_active_pane_id(active_pane_id);
    }

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        let size = Size {
            width: size.width,
            height: size.height,
        };
        let (cells, pixels) = self.theme().cells_and_pixels(size);

        self.cells = cells;
        self.region = Rectangle {
            top: (size.height - pixels.height) as i32 / 2,
            left: (size.width - pixels.width) as i32 / 2,
            width: pixels.width,
            height: pixels.height,
        };

        self.graphics.resize(&self.window);
        self.panes.resize(self.region);
    }

    pub fn update(&mut self, delta: Duration) {
        self.panes.update(delta);
    }

    pub fn render<'a>(
        &mut self,
        documents: impl Fn(DocumentId) -> Option<&'a Document>,
        mode: Mode,
        search: Option<(&'a str, &'a [(String, isize, Vec<Range<usize>>)], usize)>,
    ) {
        // TODO react to document closes

        let region = self.region();

        self.panes
            .render(&mut self.context, &mut self.graphics, documents, mode);

        if let Some((needle, haystack, selected)) = search {
            self.files.render(
                &mut self.context,
                self.graphics.layer(region, 1),
                needle,
                haystack,
                selected,
            );
        }

        self.graphics.render(self.theme().background_color);
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

fn ui_theme(context: &Context) -> UiTheme {
    let catppuccin = Catppuccin::default();
    let family = context.fonts().get("Victor").unwrap().key();
    let font_size = 15 as FontSize;
    let line_height = font_size as LineHeight + font_size as LineHeight / 4;

    UiTheme {
        syntax: SyntaxTheme::catppuccin(),
        background_color: catppuccin.crust.solid(),

        family,
        font_size,
        line_height,
        advance: context
            .fonts()
            .get((family, FontWeight::default(), FontStyle::default()))
            .unwrap()
            .advance_for_size(font_size),

        scroll_duration: Duration::from_millis(500),
        scroll_tween: Tween::ExpoOut,
        scrollbar_color: catppuccin.surface1.solid(),

        normal_mode_color: catppuccin.blue.solid(),
        insert_mode_color: catppuccin.pink.solid(),

        caret_width: 2,
    }
}
