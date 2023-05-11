use crate::events::{Event, Events};
use pixels::{Pixels, SurfaceTexture};
use virus_editor::{document::Document, language::Language, theme::Theme};
use virus_graphics::{
    pixels_mut::PixelsMut,
    text::{Context, Font, Fonts},
};
use virus_ui::document_view::DocumentView;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ModifiersState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

const RECURSIVE_VF: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Desktop/Recursive_VF_1.084.ttf";
const RECURSIVE: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
const UBUNTU: &str = "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf";
const FIRA: &str =
    "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";

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
    fn new(window: Window, font: Font, emoji: Font) -> Self {
        let key = font.key();
        let mut context = Context::new(Fonts::new([font], emoji));
        let mut events = Events::new(window.id());

        let mut pixels = {
            let PhysicalSize { width, height } = window.inner_size();
            Pixels::new(width, height, SurfaceTexture::new(width, height, &window)).unwrap()
        };

        let mut document = Document::empty(Some(Language::Rust));
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

        let font = {
            let fira = Font::from_file(FIRA).unwrap();
            let ubuntu = Font::from_file(UBUNTU).unwrap();
            let recursive = Font::from_file(RECURSIVE).unwrap();

            recursive
        };
        let emoji = Font::from_file(EMOJI).unwrap();

        let mut virus = Self::new(window, font, emoji);

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
        const LEFT: char = 'j';
        const RIGHT: char = 'l';

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
            LEFT if modifiers.alt() => {
                self.document.move_prev();
            }
            RIGHT if modifiers.alt() => {
                self.document.move_next();
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
