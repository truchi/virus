use std::time::Instant;

use crate::events::{Event, Events};
use virus_editor::document::Document;
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
    window: Window,
    events: Events,
    document: Document,
    ui: Ui,
    last_render: Option<Instant>,
}

impl Virus {
    fn new(window: Window) -> Self {
        let events = Events::new(window.id());
        let ui = Ui::new(&window);
        let mut document = Document::open(std::env::args().skip(1).next().unwrap()).unwrap();
        document.parse();

        Self {
            window,
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
            flow.set_wait_timeout(std::time::Duration::from_secs(2));

            let event = match virus.events.update(&event) {
                Some(event) => event,
                None => return,
            };

            // Add Event::Timeout for fixed timesteps -> request redraw
            // Let on_redraw try to figure if in animation or not, anyway no big deal if we draw at 30pfs
            // Can we be elegant with flow?

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
        const BACKSPACE: char = '\u{8}';
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
            UP if modifiers.alt() => {
                self.document.move_up();
            }
            'I' if modifiers.alt() => {
                self.ui.scroll_up();
            }
            'K' if modifiers.alt() => {
                self.ui.scroll_down();
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
                self.document.save().unwrap();
            }
            _ => {
                self.document.edit_char(char);
            }
        }

        self.document.parse();
        self.window.request_redraw();
    }

    fn on_pressed(&mut self, key: VirtualKeyCode, flow: &mut ControlFlow) {
        match key {
            VirtualKeyCode::Escape => flow.set_exit(),
            _ => {}
        }
    }

    fn on_released(&mut self, key: VirtualKeyCode, flow: &mut ControlFlow) {}

    fn on_resized(&mut self, size: PhysicalSize<u32>, flow: &mut ControlFlow) {
        self.ui.resize(size);
    }

    fn on_moved(&mut self, position: PhysicalPosition<i32>, flow: &mut ControlFlow) {}

    fn on_focused(&mut self, flow: &mut ControlFlow) {}

    fn on_unfocused(&mut self, flow: &mut ControlFlow) {}

    fn on_redraw(&mut self, flow: &mut ControlFlow) {
        let now = Instant::now();
        let delta = now - self.last_render.unwrap_or(now);

        self.ui.update(delta);
        self.ui.render(&self.document);

        if self.ui.is_animating() {
            self.window.request_redraw();
        }

        self.last_render = Some(now);
    }

    fn on_close(&mut self, flow: &mut ControlFlow) {}

    fn on_closed(&mut self, flow: &mut ControlFlow) {}

    fn on_quit(&mut self, flow: &mut ControlFlow) {}
}
