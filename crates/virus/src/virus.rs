//! 🦠: a 😍 editor in 🦀 with ❤️
// Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
// hello {((((world))))}, salut

use crate::events::{Event, Events, Key};
use std::{
    ops::Range,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use virus_editor::{document::Document, editor::Editor, fuzzy::Fuzzy};
use virus_ui::{theme::Theme, tween::Tween, ui::Ui};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
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
    editor: Editor,
    mode: Mode,
    ui: Ui,
    last_render: Option<Instant>,
    search: Option<(
        String,
        Vec<String>,
        Vec<(String, isize, Vec<Range<usize>>)>,
        usize,
    )>,
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
                font_size: 20,
                line_height: 25,
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
        let editor = {
            let file = PathBuf::from(std::env::args().skip(1).next().expect("File argument"));
            let root = Editor::find_git_root(file.clone())
                .unwrap_or_else(|| std::env::current_dir().expect("Current dir").into());

            let mut editor = Editor::new(root);
            editor.open(file).unwrap();
            editor
        };

        Self {
            events,
            editor,
            mode: Mode::default(),
            ui,
            last_render: None,
            search: None,
        }
    }
}

/// Event handlers.
impl Virus {
    fn on_key(&mut self, key: Key, event_loop: &ActiveEventLoop) {
        if let Some((needle, files, haystacks, selected)) = &mut self.search {
            match key {
                Key::Str("i") if self.events.command() => {
                    if *selected == 0 {
                        *selected = haystacks.len().saturating_sub(1);
                    } else {
                        *selected = *selected - 1;
                    }
                }
                Key::Str("k") if self.events.command() => {
                    if *selected == haystacks.len().saturating_sub(1) {
                        *selected = 0;
                    } else {
                        *selected = *selected + 1;
                    }
                }
                Key::Str(str) => {
                    needle.push_str(str);
                    *selected = 0;
                    *haystacks = Fuzzy::new_file_search(needle)
                        .scores(files.iter().map(|file| file.as_str()));
                }
                Key::Tab => {}
                Key::Space => {
                    needle.push(' ');
                    *selected = 0;
                    *haystacks = Fuzzy::new_file_search(needle)
                        .scores(files.iter().map(|file| file.as_str()));
                }
                Key::Backspace => {
                    needle.pop();
                    *selected = 0;
                    if needle.is_empty() {
                        *haystacks = files
                            .iter()
                            .map(|file| (file.to_owned(), 0, Vec::new()))
                            .collect();
                    } else {
                        *haystacks = Fuzzy::new_file_search(needle)
                            .scores(files.iter().map(|file| file.as_str()));
                    }
                }
                Key::Enter => {
                    let path = self.editor.root().join(&haystacks[*selected].0);
                    self.editor.open(path).unwrap();
                    self.search = None;
                }
                Key::Escape => {
                    self.search = None;
                }
            }
        } else {
            match &mut self.mode {
                Mode::Normal { select_mode } => match key {
                    Key::Str("@") if self.events.command() => event_loop.exit(),
                    Key::Str("i") => {
                        self.editor
                            .active_document_mut()
                            .move_up(select_mode.is_some(), 1);
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("k") => {
                        self.editor
                            .active_document_mut()
                            .move_down(select_mode.is_some(), 1);
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("j") => self
                        .editor
                        .active_document_mut()
                        .move_prev_grapheme(select_mode.is_some()),
                    Key::Str("l") => self
                        .editor
                        .active_document_mut()
                        .move_next_grapheme(select_mode.is_some()),
                    Key::Str("e") => {
                        self.editor
                            .active_document_mut()
                            .move_next_end_of_word(select_mode.is_some());
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("E") => {
                        self.editor
                            .active_document_mut()
                            .move_prev_end_of_word(select_mode.is_some());
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("w") => {
                        self.editor
                            .active_document_mut()
                            .move_next_start_of_word(select_mode.is_some());
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("W") => {
                        self.editor
                            .active_document_mut()
                            .move_prev_start_of_word(select_mode.is_some());
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("y") => {
                        self.editor
                            .active_document_mut()
                            .move_up(select_mode.is_some(), 10);
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("h") => {
                        self.editor
                            .active_document_mut()
                            .move_down(select_mode.is_some(), 10);
                        self.ui
                            .ensure_visibility(self.editor.active_document().head_line());
                    }
                    Key::Str("v") => match select_mode {
                        Some(SelectMode::Range) => *select_mode = Some(SelectMode::Line),
                        Some(SelectMode::Line) => {
                            self.editor.active_document_mut().flip_anchor_and_head();
                            *select_mode = Some(SelectMode::Range);
                        }
                        None => *select_mode = Some(SelectMode::Range),
                    },
                    Key::Str("V") => {
                        *select_mode = None;
                        self.editor.active_document_mut().move_anchor_to_head();
                    }
                    Key::Str("s") if self.events.command() => {
                        self.editor.active_document_mut().save().unwrap()
                    }
                    Key::Str("/") => {
                        let files = self
                            .editor
                            .files(true, false)
                            .filter_map(|file| {
                                file.as_os_str().to_str().map(|file| file.to_owned())
                            })
                            .collect::<Vec<_>>();
                        let haystacks = files
                            .iter()
                            .map(|file| (file.to_owned(), 0, Vec::new()))
                            .collect();
                        self.search = Some((String::new(), files, haystacks, 0));
                    }
                    Key::Escape => self.mode = Mode::Insert,
                    _ => (),
                },
                Mode::Insert => match key {
                    Key::Str("@") if self.events.command() => event_loop.exit(),
                    Key::Str(str) => self.editor.active_document_mut().edit(str),
                    Key::Space => self.editor.active_document_mut().edit(" "),
                    Key::Backspace => self.editor.active_document_mut().backspace(),
                    Key::Enter => self.editor.active_document_mut().edit("\n"),
                    Key::Escape => {
                        self.mode = Mode::Normal {
                            select_mode: (!self
                                .editor
                                .active_document()
                                .selection()
                                .range()
                                .is_empty())
                            .then_some(SelectMode::Range),
                        }
                    }

                    _ => (),
                },
            }
        }

        // TODO handle that better
        self.editor.active_document_mut().parse();

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
            self.editor.active_document_mut(),
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
            self.search
                .as_ref()
                .map(|(needle, _, haystacks, selected)| {
                    (needle.as_str(), haystacks.as_slice(), *selected)
                }),
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
