//! 🦠: a 😍 editor in 🦀 with ❤️
// Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
// hello {((((world))))}, salut

use crate::events::{Event, Events, Key};
use std::{
    ops::Range,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    process::Command,
    sync::{mpsc::unbounded_channel, oneshot::channel},
};
use virus_editor::{
    async_actor::AsyncActor,
    editor::{Editor, EventLoopMessage},
    fuzzy::Fuzzy,
};
use virus_ui::{theme::Theme, tween::Tween, ui::Ui};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Handler                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

enum Handler {
    Uninitialized { editor: Option<Arc<Mutex<Editor>>> },
    Initialized { virus: Virus },
}

impl ApplicationHandler<EventLoopMessage> for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let editor = match self {
            Handler::Uninitialized { editor } => editor,
            Handler::Initialized { .. } => panic!("Already initialized"),
        };

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title(Virus::TITLE)
                    // .with_fullscreen(Some(Fullscreen::Borderless(None)))
                    .with_decorations(false),
            )
            .expect("Cannot create window");
        window.set_cursor_visible(false);

        *self = Handler::Initialized {
            virus: Virus::new(window, editor.take().unwrap()),
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let virus = match self {
            Handler::Uninitialized { .. } => panic!("Not initialized"),
            Handler::Initialized { virus } => virus,
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

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: EventLoopMessage) {}
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

#[derive(Clone, Eq, PartialEq, Debug)]
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
    editor: Arc<Mutex<Editor>>,
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
        let event_loop = EventLoop::<EventLoopMessage>::with_user_event()
            .build()
            .expect("Cannot create event loop");
        let event_loop_proxy = event_loop.create_proxy();
        let (async_actor_sender, async_actor_receiver) = unbounded_channel();
        let editor = {
            let file = PathBuf::from(std::env::args().skip(1).next().expect("File argument"));
            let root = Editor::find_git_root(file.clone())
                .unwrap_or_else(|| std::env::current_dir().expect("Current directory").into());
            let mut editor = Editor::new(
                root,
                (Command::new("rust-analyzer"),),
                async_actor_sender,
                Box::new(move |message| {
                    event_loop_proxy
                        .send_event(message)
                        .expect("Send event loop event")
                }),
            );

            editor.open(file).unwrap();

            Arc::new(Mutex::new(editor))
        };
        let async_actor = AsyncActor::new(Arc::downgrade(&editor), async_actor_receiver);

        let async_actor_handle = std::thread::spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Tokio runtime")
                .block_on(async {
                    tokio::spawn(async_actor.run())
                        .await
                        .expect("AsyncActor::run");
                });
        });

        //
        // Run
        //

        let mut handler = Handler::Uninitialized {
            editor: Some(editor),
        };
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut handler).unwrap();

        //
        // Exit
        //

        match handler {
            Handler::Uninitialized { .. } => panic!("Not initialized"),
            Handler::Initialized { virus } => virus.exit(),
        };

        async_actor_handle.join().unwrap();
    }
}

/// Private.
impl Virus {
    const TITLE: &'static str = "Virus";
    const FRAMES_PER_SECOND: u8 = 60;
    const MILLIS_PER_FRAME: u128 = 1000 / Virus::FRAMES_PER_SECOND as u128;

    fn new(window: Window, editor: Arc<Mutex<Editor>>) -> Self {
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

        Self {
            events,
            editor,
            mode: Mode::default(),
            ui,
            last_render: None,
            search: None,
        }
    }

    pub fn exit(self) {
        // NOTE: This is supposed to close the window early but doesn't
        // TODO: Try a very small example?
        drop(self.ui);

        let (sender, receiver) = channel();
        self.editor.lock().unwrap().exit(sender);

        receiver.blocking_recv().unwrap();
    }
}

/// Event handlers.
impl Virus {
    fn on_key(&mut self, key: Key, event_loop: &ActiveEventLoop) {
        let mut editor = self.editor.lock().unwrap();

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
                    let path = editor.root().join(&haystacks[*selected].0);
                    editor.open(path).unwrap();
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
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_up(select_mode.is_some(), 1);
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("k") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_down(select_mode.is_some(), 1);
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("j") => editor
                        .get_active_document_mut()
                        .unwrap()
                        .move_prev_grapheme(select_mode.is_some()),
                    Key::Str("l") => editor
                        .get_active_document_mut()
                        .unwrap()
                        .move_next_grapheme(select_mode.is_some()),
                    Key::Str("e") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_next_end_of_word(select_mode.is_some());
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("E") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_prev_end_of_word(select_mode.is_some());
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("w") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_next_start_of_word(select_mode.is_some());
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("W") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_prev_start_of_word(select_mode.is_some());
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("c") if self.events.command() => editor.paste(),
                    Key::Str("c") => editor.copy(),
                    Key::Str("z") if self.events.command() => {
                        editor.get_active_document_mut().unwrap().redo();
                        if editor.get_active_document().unwrap().selection().is_empty() {
                            *select_mode = None;
                        } else {
                            *select_mode = Some(SelectMode::Range);
                        }
                    }
                    Key::Str("z") => {
                        editor.get_active_document_mut().unwrap().undo();
                        if editor.get_active_document().unwrap().selection().is_empty() {
                            *select_mode = None;
                        } else {
                            *select_mode = Some(SelectMode::Range);
                        }
                    }
                    Key::Str("y") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_up(select_mode.is_some(), 10);
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("h") => {
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_down(select_mode.is_some(), 10);
                        self.ui.ensure_visibility(
                            editor
                                .get_active_document()
                                .unwrap()
                                .selection()
                                .head
                                .line(),
                        );
                    }
                    Key::Str("v") => match select_mode {
                        Some(SelectMode::Range) => *select_mode = Some(SelectMode::Line),
                        Some(SelectMode::Line) => {
                            editor
                                .get_active_document_mut()
                                .unwrap()
                                .flip_anchor_and_head();
                            *select_mode = Some(SelectMode::Range);
                        }
                        None => *select_mode = Some(SelectMode::Range),
                    },
                    Key::Str("V") => {
                        *select_mode = None;
                        editor
                            .get_active_document_mut()
                            .unwrap()
                            .move_anchor_to_head();
                    }
                    Key::Str("s") if self.events.command() => {
                        editor.get_active_document_mut().unwrap().save().unwrap()
                    }
                    Key::Str("/") => {
                        let files = editor
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
                    Key::Str(str) => editor.get_active_document_mut().unwrap().edit(str.into()),
                    Key::Space => editor.get_active_document_mut().unwrap().edit(" ".into()),
                    Key::Backspace => editor.get_active_document_mut().unwrap().backspace(),
                    Key::Enter => editor.get_active_document_mut().unwrap().edit("\n".into()),
                    Key::Escape => {
                        self.mode = Mode::Normal {
                            select_mode: (!editor
                                .get_active_document()
                                .unwrap()
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
        editor.get_active_document_mut().unwrap().parse();

        // TODO handle that better
        self.ui.window().request_redraw();
    }

    fn on_resized(&mut self, event_loop: &ActiveEventLoop) {
        self.ui.resize();
        self.on_redraw(event_loop);
    }

    fn on_redraw(&mut self, _event_loop: &ActiveEventLoop) {
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
        let mut editor = self.editor.lock().unwrap();
        self.ui.render(
            editor.get_active_document_mut().unwrap(),
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
