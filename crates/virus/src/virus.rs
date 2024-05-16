//! 🦠: a 😍 editor in 🦀 with ❤️

use crate::events::{Event, Events, Key};
use std::{sync::Arc, time::Instant};
use virus_common::Cursor;
use virus_editor::document::{Document, Selection};
use virus_ui::ui::Ui;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowAttributes, WindowId},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Handler                                            //
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
                Window::default_attributes().with_title(Virus::TITLE), // .with_fullscreen(Some(Fullscreen::Borderless(None))),
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

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        println!("vvvvvvvvvvvvvvvvvvvvvvvv");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        println!("user events");
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        println!("device events");
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        println!("^^^^^^^^^^^^^^^^^^^^^^^^");
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        println!("suspended");
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("exiting");
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        println!("memory warning");
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Virus                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Virus {
    events: Events,
    document: Document,
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
        let mut document = Document::open(std::env::args().skip(1).next().unwrap()).unwrap();
        document.parse();

        // TODO
        *document.selection_mut() = Selection::ast(Cursor::default()..Cursor::default());

        Self {
            events,
            document,
            ui,
            last_render: None,
        }
    }
}

/// Event handlers.
impl Virus {
    fn on_key(&mut self, key: Key, event_loop: &ActiveEventLoop) {
        match key {
            Key::Str("i") if self.events.command() => {
                self.document.move_up();
                self.ui
                    .ensure_selection_is_visible(self.document.selection());
            }
            Key::Str("k") if self.events.command() => {
                self.document.move_down();
                self.ui
                    .ensure_selection_is_visible(self.document.selection());
            }
            Key::Str("j") if self.events.command() => self.document.move_left(),
            Key::Str("l") if self.events.command() => self.document.move_right(),
            Key::Str("I") if self.events.command() => self.ui.scroll_up(),
            Key::Str("K") if self.events.command() => self.ui.scroll_down(),
            Key::Str("s") if self.events.command() => self.document.save().unwrap(),
            Key::Str(str) => self.document.edit_char(str.chars().next().unwrap()), // TODO edit_str
            Key::Tab => (),
            Key::Space => self.document.edit_char(' '),
            Key::Backspace => self.document.backspace().unwrap(),
            Key::Enter => self.document.edit_char('\n'),
            Key::Escape => event_loop.exit(),
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
