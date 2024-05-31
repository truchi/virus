//! 🦠: a 😍 editor in 🦀 with ❤️
// Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
// hello {((((world))))}, salut

use crate::events::{Event, Events, Key};
use std::{sync::Arc, time::Instant};
use virus_common::Cursor;
use virus_editor::document::Document;
use virus_ui::ui::Ui;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowAttributes, WindowId},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Handler                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

enum Handler {
    Uninitialized,
    Initialized(Virus),
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Handler::Initialized(_) = self {
            panic!("Already initialized");
        }

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title(Virus::TITLE)
                    // .with_fullscreen(Some(Fullscreen::Borderless(None)))
                    .with_decorations(false),
            )
            .expect("Cannot create window");
        window.set_cursor_visible(false);

        *self = Handler::Initialized(Virus::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let virus = match self {
            Handler::Uninitialized => panic!("Not initialized"),
            Handler::Initialized(virus) => virus,
        };

        let event = match virus.events.update(&event) {
            Some(event) => event,
            None => return,
        };

        match event {
            Event::Key(key) => virus.on_key(key, event_loop),
            Event::Resized => virus.on_resized(event_loop),
            Event::Redraw => virus.on_redraw(event_loop),
            Event::Close => virus.on_close(),
            Event::Closed => virus.on_closed(),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Virus                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

enum Mode {
    Normal,
    Insert,
}

pub struct Virus {
    events: Events,
    document: Document,
    mode: Mode,
    ui: Ui,
    last_render: Option<Instant>,
}

impl Virus {
    /// Runs `virus`.
    pub fn run() {
        let event_loop = EventLoop::new().expect("Cannot create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut Handler::Uninitialized);
    }
}

/// Private.
impl Virus {
    const TITLE: &'static str = "Virus";
    const FRAMES_PER_SECOND: u8 = 60;
    const MILLIS_PER_FRAME: u128 = 1000 / Virus::FRAMES_PER_SECOND as u128;

    fn new(window: Window) -> Self {
        let events = Events::new();
        let ui = Ui::new(Arc::new(window));
        let mut document = Document::open(&std::env::args().skip(1).next().unwrap()).unwrap();
        document.parse();

        Self {
            events,
            document,
            mode: Mode::Normal,
            ui,
            last_render: None,
        }
    }
}

/// Event handlers.
impl Virus {
    fn on_key(&mut self, key: Key, event_loop: &ActiveEventLoop) {
        match self.mode {
            Mode::Normal => match key {
                Key::Str("i") => {
                    self.document.move_up();
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("k") => {
                    self.document.move_down();
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("j") => self.document.move_prev_grapheme(),
                Key::Str("l") => self.document.move_next_grapheme(),
                Key::Str("e") => {
                    self.document.move_next_end_of_word();
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("E") => {
                    self.document.move_prev_end_of_word();
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("w") => {
                    self.document.move_next_start_of_word();
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("W") => {
                    self.document.move_prev_start_of_word();
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("y") => self.ui.scroll_up(),
                Key::Str("h") => self.ui.scroll_down(),
                // Key::Str("s") if self.events.command() => self.document.save().unwrap(),
                Key::Escape => event_loop.exit(),
                _ => (),
            },
            Mode::Insert => match key {
                Key::Str(str) => self.document.edit(str),
                Key::Space => self.document.edit(" "),
                Key::Backspace => self.document.backspace().unwrap(),
                Key::Enter => self.document.edit("\n"),
                Key::Escape => self.mode = Mode::Normal,
                _ => (),
            },
        }

        // TODO handle that better
        self.document.parse();

        // TODO handle that better
        self.ui.window().request_redraw();
    }

    fn on_resized(&mut self, event_loop: &ActiveEventLoop) {
        self.ui.resize();
        self.on_redraw(event_loop);
    }

    fn on_redraw(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta = if let Some(delta) = self.last_render.map(|last_render| now - last_render) {
            if delta.as_millis() < Self::MILLIS_PER_FRAME {
                // TODO better frame scheduling
                self.ui.window().request_redraw();
                return;
            }

            delta
        } else {
            Default::default()
        };

        self.last_render = Some(now);
        self.ui.update(delta);
        self.ui.render(&self.document);

        if self.ui.is_animating() {
            self.ui.window().request_redraw();
        }
    }

    fn on_close(&mut self) {
        println!("Close")
    }

    fn on_closed(&mut self) {
        println!("Closed")
    }
}
