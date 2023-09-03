//! 🦠: a 😍 editor in 🦀 with ❤️

use crate::events::{Event, Events};
use std::time::Instant;
use virus_common::Cursor;
use virus_editor::document::{Document, Selection};
use virus_ui::ui::Ui;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Virus                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Virus {
    events: Events,
    document: Document,
    ui: Ui,
    last_render: Option<Instant>,
}

impl Virus {
    const FRAMES_PER_SECOND: u8 = 60;
    const MILLIS_PER_FRAME: u128 = 1000 / Virus::FRAMES_PER_SECOND as u128;

    fn new(window: Window) -> Self {
        let events = Events::new();
        let ui = Ui::new(window);
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

    pub fn run(title: &str) {
        let event_loop = EventLoop::new();
        let mut virus = Self::new({
            let window = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(PhysicalSize::new(1, 1))
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .build(&event_loop)
                .unwrap();
            window.set_cursor_visible(false);
            window
        });

        event_loop.run(move |event, _, flow| {
            let event = match virus.events.update(&event, virus.ui.window()) {
                Some(event) => event,
                None => return,
            };

            match event {
                Event::Char(char) => virus.on_char(char, flow),
                Event::Pressed(key) => virus.on_pressed(key, flow),
                Event::Released(key) => virus.on_released(key, flow),
                Event::Resized(size) => virus.on_resized(size, flow),
                Event::Moved(position) => virus.on_moved(position, flow),
                Event::Focused => virus.on_focused(flow),
                Event::Unfocused => virus.on_unfocused(flow),
                Event::Redraw => virus.on_redraw(flow),
                Event::Close => virus.on_close(flow),
                Event::Closed => virus.on_closed(flow),
                Event::Quit => virus.on_quit(flow),
            }
        });
    }
}

/// Event handlers.
impl Virus {
    fn on_char(&mut self, char: char, flow: &mut ControlFlow) {
        const TAB: char = '\t';
        const ENTER: char = '\r';
        const BACKSPACE: char = '\u{8}'; // Not emitted on MacOS?
        const ESCAPE: char = '\u{1b}';
        const UP: char = 'i';
        const DOWN: char = 'k';
        const LEFT: char = 'j';
        const RIGHT: char = 'l';
        const SAVE: char = 's';

        let modifiers = self.events.modifiers();

        match char {
            ESCAPE => return, // Handled as `Pressed`
            TAB => return,    // TODO
            ENTER => {
                self.document.edit_char('\n');
            }
            BACKSPACE => {
                self.document.backspace().unwrap();
            }
            'I' if modifiers.logo() => {
                self.ui.scroll_up();
            }
            'K' if modifiers.logo() => {
                self.ui.scroll_down();
            }
            UP if modifiers.logo() => {
                self.document.move_up();
                self.ui
                    .ensure_selection_is_visible(self.document.selection());
            }
            DOWN if modifiers.logo() => {
                self.document.move_down();
                self.ui
                    .ensure_selection_is_visible(self.document.selection());
            }
            LEFT if modifiers.logo() => {
                self.document.move_left();
            }
            RIGHT if modifiers.logo() => {
                self.document.move_right();
            }
            SAVE if modifiers.logo() => {
                self.document.save().unwrap();
            }
            _ => {
                self.document.edit_char(char);
            }
        }

        // TODO handle that better
        self.document.parse();
    }

    fn on_pressed(&mut self, key: VirtualKeyCode, flow: &mut ControlFlow) {
        match key {
            VirtualKeyCode::Escape => flow.set_exit(),
            VirtualKeyCode::Back => self.document.backspace().unwrap(),
            _ => {}
        }
    }

    fn on_released(&mut self, key: VirtualKeyCode, flow: &mut ControlFlow) {}

    fn on_resized(&mut self, size: PhysicalSize<u32>, flow: &mut ControlFlow) {
        self.ui.resize();
    }

    fn on_moved(&mut self, position: PhysicalPosition<i32>, flow: &mut ControlFlow) {}

    fn on_focused(&mut self, flow: &mut ControlFlow) {}

    fn on_unfocused(&mut self, flow: &mut ControlFlow) {}

    fn on_redraw(&mut self, flow: &mut ControlFlow) {
        let now = Instant::now();

        let delta = if let Some(last_render) = self.last_render {
            let delta = now - last_render;

            if delta.as_millis() < Self::MILLIS_PER_FRAME {
                return;
            }

            delta
        } else {
            Default::default()
        };

        self.last_render = Some(now);
        self.ui.update(delta);
        self.ui.render(&self.document);

        // TODO: constant redraw on MacOS?
        if self.ui.is_animating() {
            flow.set_poll();
        } else {
            flow.set_wait();
        }
    }

    fn on_close(&mut self, flow: &mut ControlFlow) {}

    fn on_closed(&mut self, flow: &mut ControlFlow) {}

    fn on_quit(&mut self, flow: &mut ControlFlow) {}
}
