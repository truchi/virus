//! 🦠: a 😍 editor in 🦀 with ❤️
// Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
// hello {((((world))))}, salut

use crate::events::{Event, Events, Key};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use virus_editor::document::Document;
use virus_ui::{theme::Theme, tween::Tween, ui::Ui};
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SelectMode {
    Range,
    Line,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Normal { select_mode: Option<SelectMode> },
    Insert,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal {
            select_mode: Default::default(),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

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
        let ui = Ui::new(Arc::new(window), {
            let catppuccin = virus_ui::Catppuccin::default();
            let normal_mode = catppuccin.blue;
            let select_mode = catppuccin.pink;
            let insert_mode = catppuccin.green;

            Theme {
                syntax: virus_editor::syntax::Theme::catppuccin(),
                font_size: 24,
                line_height: 30,
                scrollbar_color: catppuccin.surface1.solid(),
                scroll_duration: Duration::from_millis(500),
                scroll_tween: Tween::ExpoOut,
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
                caret_normal_mode_width: 4,
                caret_select_mode_width: 4,
                caret_insert_mode_width: 2,
                selection_select_mode_color: select_mode.solid().transparent(255 / 2),
                selection_insert_mode_color: insert_mode.solid().transparent(255 / 2),
            }
        });
        let mut document = Document::open(&std::env::args().skip(1).next().unwrap()).unwrap();
        document.parse();

        Self {
            events,
            document,
            mode: Mode::default(),
            ui,
            last_render: None,
        }
    }
}

/// Event handlers.
impl Virus {
    fn on_key(&mut self, key: Key, event_loop: &ActiveEventLoop) {
        match &mut self.mode {
            Mode::Normal { select_mode } => match key {
                Key::Str("@") if self.events.command() => event_loop.exit(),
                Key::Str("i") => {
                    self.document.move_up(select_mode.is_some());
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("k") => {
                    self.document.move_down(select_mode.is_some());
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("j") => self.document.move_prev_grapheme(select_mode.is_some()),
                Key::Str("l") => self.document.move_next_grapheme(select_mode.is_some()),
                Key::Str("e") => {
                    self.document.move_next_end_of_word(select_mode.is_some());
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("E") => {
                    self.document.move_prev_end_of_word(select_mode.is_some());
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("w") => {
                    self.document.move_next_start_of_word(select_mode.is_some());
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("W") => {
                    self.document.move_prev_start_of_word(select_mode.is_some());
                    self.ui.ensure_visibility(self.document.selection());
                }
                Key::Str("y") => self.ui.scroll_up(),
                Key::Str("h") => self.ui.scroll_down(),
                Key::Str("v") => match select_mode {
                    Some(SelectMode::Range) => *select_mode = Some(SelectMode::Line),
                    Some(SelectMode::Line) => {
                        self.document.flip_anchor_and_head();
                        *select_mode = Some(SelectMode::Range);
                    }
                    None => *select_mode = Some(SelectMode::Range),
                },
                Key::Str("V") => {
                    *select_mode = None;
                    self.document.move_anchor_to_head();
                }
                // Key::Str("s") if self.events.command() => self.document.save().unwrap(),
                Key::Escape => self.mode = Mode::Insert,
                _ => (),
            },
            Mode::Insert => match key {
                Key::Str("@") if self.events.command() => event_loop.exit(),
                Key::Str(str) => self.document.edit(str),
                Key::Space => self.document.edit(" "),
                Key::Backspace => self.document.backspace().unwrap(),
                Key::Enter => self.document.edit("\n"),
                Key::Escape => self.mode = Mode::default(),
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
        // TODO Oh God! Modes back to editor?
        let outline_normal_mode_colors = &self.ui.theme().outline_normal_mode_colors.clone();
        let outline_select_mode_colors = &self.ui.theme().outline_select_mode_colors.clone();
        let outline_insert_mode_colors = &self.ui.theme().outline_insert_mode_colors.clone();
        self.ui.render(
            &mut self.document,
            matches!(
                self.mode,
                Mode::Normal {
                    select_mode: Some(SelectMode::Line)
                },
            ),
            match self.mode {
                Mode::Normal {
                    select_mode: Some(_),
                } => outline_select_mode_colors,
                Mode::Normal { select_mode: None } => outline_normal_mode_colors,
                Mode::Insert => outline_insert_mode_colors,
            },
            match self.mode {
                Mode::Normal {
                    select_mode: Some(_),
                } => self.ui.theme().caret_select_mode_color,
                Mode::Normal { select_mode: None } => self.ui.theme().caret_normal_mode_color,
                Mode::Insert => self.ui.theme().caret_insert_mode_color,
            },
            match self.mode {
                Mode::Normal {
                    select_mode: Some(_),
                } => self.ui.theme().caret_select_mode_width,
                Mode::Normal { select_mode: None } => self.ui.theme().caret_normal_mode_width,
                Mode::Insert => self.ui.theme().caret_insert_mode_width,
            },
            match self.mode {
                Mode::Normal {
                    select_mode: Some(_),
                } => self.ui.theme().selection_select_mode_color,
                Mode::Normal { select_mode: None } => self.ui.theme().selection_select_mode_color,
                Mode::Insert => self.ui.theme().selection_insert_mode_color,
            },
        );

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
