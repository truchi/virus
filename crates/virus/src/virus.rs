use crate::events::{Event, Events};
use pixels::{Pixels, SurfaceTexture};
use virus_editor::{document::Document, language::Language, theme::Theme};
use virus_graphics::{
    pixels_mut::PixelsMut,
    text::{Context, Font, FontKey, FontStyle, FontWeight, Fonts},
};
use virus_ui::document_view::DocumentView;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ModifiersState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

const HIGHLIGHT_QUERY: &str = include_str!("../../editor/treesitter/rust/highlights.scm");

const SCALE: u32 = 1;

pub struct Virus {
    window: Window,
    events: Events,
    context: Context,
    document: Document,
    document_view: DocumentView,
    pixels: Pixels,
}

impl Virus {
    fn new(window: Window, fonts: Fonts, key: FontKey) -> Self {
        let mut context = Context::new(fonts);
        let mut events = Events::new(window.id());

        let mut pixels = {
            let PhysicalSize { width, height } = window.inner_size();
            Pixels::new(width, height, SurfaceTexture::new(width, height, &window)).unwrap()
        };

        let mut document = Document::empty(Some(Language::Rust));
        let mut document = Document::open(std::env::args().skip(1).next().unwrap()).unwrap();
        document.parse();

        let mut document_view =
            DocumentView::new(HIGHLIGHT_QUERY.into(), Theme::dracula(), key, 40, 50);

        Self {
            window,
            events,
            context,
            document,
            document_view,
            pixels,
        }
    }

    pub fn run(title: &str) {
        const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";
        const UBUNTU: &str = "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf";
        const FIRA_LIGHT: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Light Nerd Font Complete Mono.ttf";
        const FIRA_REGULAR: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
        const FIRA_MEDIUM: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Medium Nerd Font Complete Mono.ttf";
        const FIRA_BOLD: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Bold Nerd Font Complete Mono.ttf";
        const RECURSIVE_REGULAR: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
        const RECURSIVE_BOLD: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Bold-1.084.ttf";
        const RECURSIVE_ITALIC: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Italic-1.084.ttf";
        const RECURSIVE_BOLD_ITALIC: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-BoldItalic-1.084.ttf";
        const JETBRAINS_THIN: &str =
            "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Thin.ttf";
        const JETBRAINS_EXTRA_LIGHT : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraLight.ttf";
        const JETBRAINS_LIGHT: &str =
            "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Light.ttf";
        const JETBRAINS_REGULAR : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf";
        const JETBRAINS_MEDIUM : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Medium.ttf";
        const JETBRAINS_BOLD: &str =
            "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Bold.ttf";
        const JETBRAINS_SEMI_BOLD : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-SemiBold.ttf";
        const JETBRAINS_EXTRA_BOLD : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraBold.ttf";
        const JETBRAINS_THIN_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ThinItalic.ttf";
        const JETBRAINS_EXTRA_LIGHT_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraLightItalic.ttf";
        const JETBRAINS_LIGHT_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-LightItalic.ttf";
        const JETBRAINS_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Italic.ttf";
        const JETBRAINS_MEDIUM_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-MediumItalic.ttf";
        const JETBRAINS_BOLD_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-BoldItalic.ttf";
        const JETBRAINS_SEMI_BOLD_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-SemiBoldItalic.ttf";
        const JETBRAINS_EXTRA_BOLD_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraBoldItalic.ttf";
        const VICTOR_THIN: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Thin.ttf";
        const VICTOR_EXTRA_LIGHT: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ExtraLight.ttf";
        const VICTOR_LIGHT: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Light.ttf";
        const VICTOR_REGULAR: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Regular.ttf";
        const VICTOR_MEDIUM: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Medium.ttf";
        const VICTOR_SEMI_BOLD: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-SemiBold.ttf";
        const VICTOR_BOLD: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Bold.ttf";
        const VICTOR_THIN_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ThinItalic.ttf";
        const VICTOR_EXTRA_LIGHT_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ExtraLightItalic.ttf";
        const VICTOR_LIGHT_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-LightItalic.ttf";
        const VICTOR_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Italic.ttf";
        const VICTOR_MEDIUM_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-MediumItalic.ttf";
        const VICTOR_SEMI_BOLD_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-SemiBoldItalic.ttf";
        const VICTOR_BOLD_ITALIC: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-BoldItalic.ttf";
        const VICTOR_THIN_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ThinOblique.ttf";
        const VICTOR_EXTRA_LIGHT_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ExtraLightOblique.ttf";
        const VICTOR_LIGHT_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-LightOblique.ttf";
        const VICTOR_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Oblique.ttf";
        const VICTOR_MEDIUM_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-MediumOblique.ttf";
        const VICTOR_SEMI_BOLD_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-SemiBoldOblique.ttf";
        const VICTOR_BOLD_OBLIQUE: &str =
            "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-BoldOblique.ttf";

        let event_loop = EventLoop::new();
        let window = {
            let window = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(PhysicalSize::new(1, 1))
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .build(&event_loop)
                .unwrap();
            window.set_cursor_visible(false);
            window
        };

        let mut fonts = Fonts::new(Font::from_file(EMOJI).unwrap());

        let emoji = fonts.set(Font::from_file(EMOJI).unwrap());
        let ubuntu = fonts.set(Font::from_file(UBUNTU).unwrap());
        let fira_light = fonts.set(Font::from_file(FIRA_LIGHT).unwrap());
        let fira_regular = fonts.set(Font::from_file(FIRA_REGULAR).unwrap());
        let fira_medium = fonts.set(Font::from_file(FIRA_MEDIUM).unwrap());
        let fira_bold = fonts.set(Font::from_file(FIRA_BOLD).unwrap());
        let recursive_regular = fonts.set(Font::from_file(RECURSIVE_REGULAR).unwrap());
        let recursive_bold = fonts.set(Font::from_file(RECURSIVE_BOLD).unwrap());
        let recursive_italic = fonts.set(Font::from_file(RECURSIVE_ITALIC).unwrap());
        let recursive_bold_italic = fonts.set(Font::from_file(RECURSIVE_BOLD_ITALIC).unwrap());
        let jetbrains_thin = fonts.set(Font::from_file(JETBRAINS_THIN).unwrap());
        let jetbrains_extra_light = fonts.set(Font::from_file(JETBRAINS_EXTRA_LIGHT).unwrap());
        let jetbrains_light = fonts.set(Font::from_file(JETBRAINS_LIGHT).unwrap());
        let jetbrains_regular = fonts.set(Font::from_file(JETBRAINS_REGULAR).unwrap());
        let jetbrains_medium = fonts.set(Font::from_file(JETBRAINS_MEDIUM).unwrap());
        let jetbrains_bold = fonts.set(Font::from_file(JETBRAINS_BOLD).unwrap());
        let jetbrains_semi_bold = fonts.set(Font::from_file(JETBRAINS_SEMI_BOLD).unwrap());
        let jetbrains_extra_bold = fonts.set(Font::from_file(JETBRAINS_EXTRA_BOLD).unwrap());
        let jetbrains_thin_italic = fonts.set(Font::from_file(JETBRAINS_THIN_ITALIC).unwrap());
        let jetbrains_extra_light_italic =
            fonts.set(Font::from_file(JETBRAINS_EXTRA_LIGHT_ITALIC).unwrap());
        let jetbrains_light_italic = fonts.set(Font::from_file(JETBRAINS_LIGHT_ITALIC).unwrap());
        let jetbrains_italic = fonts.set(Font::from_file(JETBRAINS_ITALIC).unwrap());
        let jetbrains_medium_italic = fonts.set(Font::from_file(JETBRAINS_MEDIUM_ITALIC).unwrap());
        let jetbrains_bold_italic = fonts.set(Font::from_file(JETBRAINS_BOLD_ITALIC).unwrap());
        let jetbrains_semi_bold_italic =
            fonts.set(Font::from_file(JETBRAINS_SEMI_BOLD_ITALIC).unwrap());
        let jetbrains_extra_bold_italic =
            fonts.set(Font::from_file(JETBRAINS_EXTRA_BOLD_ITALIC).unwrap());
        let victor_thin = fonts.set(Font::from_file(VICTOR_THIN).unwrap());
        let victor_extra_light = fonts.set(Font::from_file(VICTOR_EXTRA_LIGHT).unwrap());
        let victor_light = fonts.set(Font::from_file(VICTOR_LIGHT).unwrap());
        let victor_regular = fonts.set(Font::from_file(VICTOR_REGULAR).unwrap());
        let victor_medium = fonts.set(Font::from_file(VICTOR_MEDIUM).unwrap());
        let victor_semi_bold = fonts.set(Font::from_file(VICTOR_SEMI_BOLD).unwrap());
        let victor_bold = fonts.set(Font::from_file(VICTOR_BOLD).unwrap());
        let victor_thin_italic = fonts.set(Font::from_file(VICTOR_THIN_ITALIC).unwrap());
        let victor_extra_light_italic =
            fonts.set(Font::from_file(VICTOR_EXTRA_LIGHT_ITALIC).unwrap());
        let victor_light_italic = fonts.set(Font::from_file(VICTOR_LIGHT_ITALIC).unwrap());
        let victor_italic = fonts.set(Font::from_file(VICTOR_ITALIC).unwrap());
        let victor_medium_italic = fonts.set(Font::from_file(VICTOR_MEDIUM_ITALIC).unwrap());
        let victor_semi_bold_italic = fonts.set(Font::from_file(VICTOR_SEMI_BOLD_ITALIC).unwrap());
        let victor_bold_italic = fonts.set(Font::from_file(VICTOR_BOLD_ITALIC).unwrap());
        let victor_thin_oblique = fonts.set(Font::from_file(VICTOR_THIN_OBLIQUE).unwrap());
        let victor_extra_light_oblique =
            fonts.set(Font::from_file(VICTOR_EXTRA_LIGHT_OBLIQUE).unwrap());
        let victor_light_oblique = fonts.set(Font::from_file(VICTOR_LIGHT_OBLIQUE).unwrap());
        let victor_oblique = fonts.set(Font::from_file(VICTOR_OBLIQUE).unwrap());
        let victor_medium_oblique = fonts.set(Font::from_file(VICTOR_MEDIUM_OBLIQUE).unwrap());
        let victor_semi_bold_oblique =
            fonts.set(Font::from_file(VICTOR_SEMI_BOLD_OBLIQUE).unwrap());
        let victor_bold_oblique = fonts.set(Font::from_file(VICTOR_BOLD_OBLIQUE).unwrap());

        use FontStyle::*;
        use FontWeight::*;

        fonts.set(
            (&[
                (Thin, Normal, victor_thin),
                (ExtraLight, Normal, victor_extra_light),
                (Light, Normal, victor_light),
                (Regular, Normal, victor_regular),
                (Medium, Normal, victor_medium),
                (SemiBold, Normal, victor_semi_bold),
                (Bold, Normal, victor_bold),
                (Thin, Italic, victor_thin_italic),
                (ExtraLight, Italic, victor_extra_light_italic),
                (Light, Italic, victor_light_italic),
                (Regular, Italic, victor_italic),
                (Medium, Italic, victor_medium_italic),
                (SemiBold, Italic, victor_semi_bold_italic),
                (Bold, Italic, victor_bold_italic),
                (Thin, Oblique, victor_thin_oblique),
                (ExtraLight, Oblique, victor_extra_light_oblique),
                (Light, Oblique, victor_light_oblique),
                (Regular, Oblique, victor_oblique),
                (Medium, Oblique, victor_medium_oblique),
                (SemiBold, Oblique, victor_semi_bold_oblique),
                (Bold, Oblique, victor_bold_oblique),
            ]) as &[_],
        );
        fonts.set(
            (&[
                (Light, Normal, fira_light),
                (Regular, Normal, fira_regular),
                (Medium, Normal, fira_medium),
                (Bold, Normal, fira_bold),
            ]) as &[_],
        );
        fonts.set(
            (&[
                (Regular, Normal, recursive_regular),
                (Bold, Normal, recursive_bold),
                (Regular, Italic, recursive_italic),
                (Bold, Italic, recursive_bold_italic),
            ]) as &[_],
        );
        fonts.set(
            (&[
                (Thin, Normal, jetbrains_thin),
                (ExtraLight, Normal, jetbrains_extra_light),
                (Light, Normal, jetbrains_light),
                (Regular, Normal, jetbrains_regular),
                (Medium, Normal, jetbrains_medium),
                (SemiBold, Normal, jetbrains_semi_bold),
                (Bold, Normal, jetbrains_bold),
                (ExtraBold, Normal, jetbrains_extra_bold),
                (Thin, Italic, jetbrains_thin_italic),
                (ExtraLight, Italic, jetbrains_extra_light_italic),
                (Light, Italic, jetbrains_light_italic),
                (Regular, Italic, jetbrains_italic),
                (Medium, Italic, jetbrains_medium_italic),
                (SemiBold, Italic, jetbrains_semi_bold_italic),
                (Bold, Italic, jetbrains_bold_italic),
                (ExtraBold, Italic, jetbrains_extra_bold_italic),
            ]) as &[_],
        );

        let mut virus = Self::new(window, fonts, jetbrains_regular);

        event_loop.run(move |event, _, flow| {
            let event = match virus.events.update(&event) {
                Some(event) => event,
                None => return,
            };
            let modifiers = virus.events.modifiers();

            match event {
                Event::Char(char) => virus.on_char(char, modifiers, flow),
                Event::Pressed(key) => virus.on_pressed(key, modifiers, flow),
                Event::Released(key) => virus.on_released(key, modifiers, flow),
                Event::Resized(size) => virus.on_resized(size, modifiers, flow),
                Event::Moved(position) => virus.on_moved(position, modifiers, flow),
                Event::Focused => virus.on_focused(modifiers, flow),
                Event::Unfocused => virus.on_unfocused(modifiers, flow),
                Event::Close => virus.on_close(modifiers, flow),
                Event::Closed => virus.on_closed(modifiers, flow),
                Event::Update => virus.on_update(modifiers, flow),
                Event::Redraw => virus.on_redraw(modifiers, flow),
                Event::Quit => virus.on_quit(modifiers, flow),
            }
        });
    }
}

/// Event handlers.
impl Virus {
    fn on_char(&mut self, char: char, modifiers: ModifiersState, flow: &mut ControlFlow) {
        const TAB: char = '\t';
        const ENTER: char = '\r';
        const BACKSPACE: char = '\u{8}';
        const ESCAPE: char = '\u{1b}';
        const UP: char = 'i';
        const DOWN: char = 'k';
        const LEFT: char = 'j';
        const RIGHT: char = 'l';
        const SAVE: char = 's';

        dbg!(char);
        match char {
            ESCAPE => return, // Handled as `Pressed`
            TAB => return,    // TODO
            ENTER => {
                self.document.edit_char('\n');
            }
            BACKSPACE => {
                self.document.backspace();
            }
            UP if modifiers.alt() => {
                self.document.move_up();
            }
            DOWN if modifiers.alt() => {
                self.document.move_down();
            }
            LEFT if modifiers.alt() => {
                self.document.move_prev();
            }
            RIGHT if modifiers.alt() => {
                self.document.move_next();
            }
            SAVE if modifiers.alt() => {
                self.document.save();
            }
            _ => {
                self.document.edit_char(char);
            }
        }

        self.document.parse();
    }

    fn on_pressed(
        &mut self,
        key: VirtualKeyCode,
        modifiers: ModifiersState,
        flow: &mut ControlFlow,
    ) {
        match key {
            VirtualKeyCode::Escape => flow.set_exit(),
            _ => {}
        }
    }

    fn on_released(
        &mut self,
        key: VirtualKeyCode,
        modifiers: ModifiersState,
        flow: &mut ControlFlow,
    ) {
    }

    fn on_resized(
        &mut self,
        size: PhysicalSize<u32>,
        modifiers: ModifiersState,
        flow: &mut ControlFlow,
    ) {
        let (width, height) = (size.width, size.height);

        if width != 1 {
            self.pixels.resize_surface(width, height).unwrap();
            self.pixels
                .resize_buffer(width / SCALE, height / SCALE)
                .unwrap();
        }
    }

    fn on_moved(
        &mut self,
        position: PhysicalPosition<i32>,
        modifiers: ModifiersState,
        flow: &mut ControlFlow,
    ) {
    }

    fn on_focused(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {}

    fn on_unfocused(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {}

    fn on_close(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {}

    fn on_closed(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {}

    fn on_update(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {
        self.window.request_redraw();
    }

    fn on_redraw(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {
        let mut pixels_mut = {
            let PhysicalSize { width, height } = self.window.inner_size();
            PixelsMut::new(width / SCALE, height / SCALE, self.pixels.get_frame_mut())
        };

        for (i, u) in pixels_mut.pixels_mut().iter_mut().enumerate() {
            *u = match i % 4 {
                0 => 0,
                1 => 0,
                2 => 0,
                _ => 255,
            };
        }

        let width = pixels_mut.width();
        let height = pixels_mut.height();

        if pixels_mut.pixels().len() == 4 {
            return;
        }

        self.document_view.render(
            &mut pixels_mut.surface(0, 0, width, height),
            &mut self.context,
            &self.document,
            0,
        );

        self.pixels.render().unwrap();
    }

    fn on_quit(&mut self, modifiers: ModifiersState, flow: &mut ControlFlow) {}
}
