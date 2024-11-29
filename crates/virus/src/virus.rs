//! 🦠: a 😍 editor in 🦀 with ❤️
// Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
// hello {((((world))))}, salut

use crate::{
    events::{Event, Events, Key, KeyEvent},
    keybindings::{ActionHandler, Keybindings},
};
use std::{
    ops::Range,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use virus_editor::{
    add_in_range,
    document::Document,
    editor::{Editor, EventLoopMessage},
    fuzzy::Fuzzy,
    rope::{Boundaries, Cursor, Text},
    sub_in_range,
};
use virus_ui::{theme::UiTheme, tween::Tween, ui::Ui};
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
    Uninitialized { editor: Option<Editor> },
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
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let virus = match self {
            Handler::Uninitialized { .. } => panic!("Not initialized"),
            Handler::Initialized { virus } => virus,
        };

        virus.window_event(event_loop, window_id, event);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: EventLoopMessage) {
        let virus = match self {
            Handler::Uninitialized { .. } => panic!("Not initialized"),
            Handler::Initialized { virus } => virus,
        };

        virus.user_event(event_loop, event);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Virus                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Select {
    #[default]
    None,
    Range,
    Lines,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Normal { select: Select },
    Insert { select: Select },
    Files,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal {
            select: Default::default(),
        }
    }
}

impl Mode {
    pub fn select(&self) -> Select {
        match self {
            Mode::Normal { select } => *select,
            Mode::Insert { select } => *select,
            Mode::Files => Select::None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Clone, Debug)]
pub struct Clipboard {
    select: Select,
    text: Text,
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
    keybindings: Keybindings,
    clipboard: Option<Clipboard>,
}

impl Virus {
    /// Runs `virus`.
    pub fn run() {
        let event_loop = EventLoop::<EventLoopMessage>::with_user_event()
            .build()
            .expect("Cannot create event loop");
        let editor = {
            let file = PathBuf::from(std::env::args().skip(1).next().expect("File argument"));
            let root = Editor::find_git_root(file.clone())
                .unwrap_or_else(|| std::env::current_dir().expect("Current directory").into());
            let mut editor = Editor::new(root);

            editor.open(file).unwrap();

            editor
        };

        //
        // Run
        //

        let mut handler = Handler::Uninitialized {
            editor: Some(editor),
        };
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut handler).unwrap();
    }
}

/// Private.
impl Virus {
    const TITLE: &'static str = "Virus";
    const FRAMES_PER_SECOND: u8 = 60;
    const MILLIS_PER_FRAME: u128 = 1000 / Virus::FRAMES_PER_SECOND as u128;

    fn new(window: Window, editor: Editor) -> Self {
        let events = Events::new();
        let ui = Ui::new(Arc::new(window), {
            let catppuccin = virus_ui::Catppuccin::default();
            let normal_mode = catppuccin.blue;
            let select_mode = catppuccin.pink;
            let insert_mode = catppuccin.green;

            UiTheme {
                syntax: virus_ui::syntax::SyntaxTheme::catppuccin(),
                family: "Victor",
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
                caret_normal_mode_width: 2,
                caret_select_mode_width: 2,
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
            keybindings: Default::default(),
            clipboard: None,
        }
    }

    fn unwrap_active_document(&self) -> &Document {
        self.editor.get_active_document().unwrap()
    }

    fn unwrap_active_document_mut(&mut self) -> &mut Document {
        self.editor.get_active_document_mut().unwrap()
    }

    fn mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
            self.keybindings.mode(mode);
        }
    }

    fn ensure_visibility(&mut self) {
        self.ui
            .ensure_visibility(self.unwrap_active_document().selection().head.line);
    }
}

/// Event handlers.
impl Virus {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match self.events.update(&event) {
            Some(Event::Key(event)) => self.on_key(event, event_loop),
            Some(Event::Resized) => self.on_resized(event_loop),
            Some(Event::Redraw) => self.on_redraw(event_loop),
            Some(Event::Close) => self.on_close(),
            Some(Event::Closed) => self.on_closed(),
            None => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: EventLoopMessage) {}

    fn on_key(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        match self.keybindings.handle(&event) {
            Ok(Some(action)) => {
                VirusActionHandler {
                    virus: self,
                    event_loop,
                }
                .handle(action);
            }
            Ok(None) => {}
            Err(()) => match self.mode {
                Mode::Normal { .. } => {}
                Mode::Insert { select } => match event.modded() {
                    Key::Str(str) => {
                        self.unwrap_active_document_mut()
                            .edition()
                            .edit(str.as_str().into(), false);
                    }
                    Key::Tab => {
                        self.unwrap_active_document_mut()
                            .edition()
                            .edit("    ".into(), false);
                    }
                    Key::Space => {
                        self.unwrap_active_document_mut()
                            .edition()
                            .edit(" ".into(), false);
                    }
                    Key::Backspace => {
                        self.unwrap_active_document_mut().edition().backspace();
                    }
                    Key::Enter => {
                        self.editor
                            .get_active_document_mut()
                            .unwrap()
                            .edition()
                            .edit("\n".into(), false);
                    }
                    _ => {}
                },
                Mode::Files => {
                    let (needle, files, haystacks, selected) = self.search.as_mut().unwrap();

                    match event.modded().as_str() {
                        Key::Str(str) => {
                            needle.push_str(str);
                            *selected = 0;
                            *haystacks = Fuzzy::new_file_search(needle)
                                .scores(files.iter().map(|file| file.as_str()));
                        }
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
                        _ => {}
                    }
                }
            },
        }

        // TODO handle that better
        self.unwrap_active_document_mut().parse();

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
        let outline_normal_mode_colors = &self.ui.theme().outline_normal_mode_colors.clone();
        let outline_select_mode_colors = &self.ui.theme().outline_select_mode_colors.clone();
        let outline_insert_mode_colors = &self.ui.theme().outline_insert_mode_colors.clone();
        self.ui.render(
            self.editor.get_active_document().unwrap(),
            self.mode.select() == Select::Lines,
            match self.mode {
                Mode::Normal {
                    select: Select::None,
                } => outline_normal_mode_colors,
                Mode::Insert { .. } => outline_insert_mode_colors,
                _ => outline_select_mode_colors,
            },
            match self.mode {
                Mode::Normal {
                    select: Select::None,
                } => self.ui.theme().caret_normal_mode_color,
                Mode::Insert { .. } => self.ui.theme().caret_insert_mode_color,
                _ => self.ui.theme().caret_select_mode_color,
            },
            match self.mode {
                Mode::Normal {
                    select: Select::None,
                } => self.ui.theme().caret_normal_mode_width,
                Mode::Insert { .. } => self.ui.theme().caret_insert_mode_width,
                _ => self.ui.theme().caret_select_mode_width,
            },
            match self.mode {
                Mode::Normal {
                    select: Select::None,
                } => self.ui.theme().selection_select_mode_color,
                Mode::Insert { .. } => self.ui.theme().selection_insert_mode_color,
                _ => self.ui.theme().selection_select_mode_color,
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

    fn on_close(&mut self) {}

    fn on_closed(&mut self) {}
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       VirusActionHandler                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct VirusActionHandler<'a> {
    virus: &'a mut Virus,
    event_loop: &'a ActiveEventLoop,
}

impl<'a> ActionHandler for VirusActionHandler<'a> {
    //
    // MOVE
    //

    // MOVE up

    fn move_top(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .top(blank)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, _, selected) = self.virus.search.as_mut().unwrap();
                *selected = 0;
            }
        }
    }

    fn move_up_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let lines = self.virus.ui.screen_height_in_lines() as usize;
                let lines = lines / if half { 2 } else { 1 };
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .up(pages * lines, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                let lines = 10; // TODO: we don't know the height in lines here...
                let lines = lines / if half { 2 } else { 1 };
                *selected = sub_in_range(haystacks.len(), *selected, pages * lines, wrap);
            }
        }
    }

    fn move_up_line(&mut self, lines: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .up(lines, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                *selected = sub_in_range(haystacks.len(), *selected, lines, wrap);
            }
        }
    }

    // MOVE down

    fn move_bottom(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .bottom(blank)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                *selected = haystacks.len().saturating_sub(1);
            }
        }
    }

    fn move_down_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let lines = self.virus.ui.screen_height_in_lines() as usize;
                let lines = lines / if half { 2 } else { 1 };
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .down(pages * lines, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                let lines = 10; // TODO: we don't know the height in lines here...
                let lines = lines / if half { 2 } else { 1 };
                *selected = add_in_range(haystacks.len(), *selected, pages * lines, wrap);
            }
        }
    }

    fn move_down_line(&mut self, lines: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .down(lines, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                *selected = add_in_range(haystacks.len(), *selected, lines, wrap);
            }
        }
    }

    // MOVE left

    fn move_start(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .start(blank)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, _, selected) = self.virus.search.as_mut().unwrap();
                *selected = 0;
            }
        }
    }

    fn move_left_boundary(
        &mut self,
        boundaries: usize,
        punctuation_start: bool,
        punctuation_end: bool,
        short_word_start: bool,
        short_word_end: bool,
        long_word_start: bool,
        long_word_end: bool,
        wrap: bool,
    ) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .left(
                        Boundaries::from_bools(
                            punctuation_start,
                            punctuation_end,
                            short_word_start,
                            short_word_end,
                            long_word_start,
                            long_word_end,
                        ),
                        boundaries,
                        wrap,
                    )
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    fn move_left_char(&mut self, chars: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .left(Boundaries::GRAPHEME, chars, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    fn move_left_smart(&mut self, repeat: usize, wrap: bool) {
        let in_normal = Boundaries::PUNCTUATION | Boundaries::WORD;
        let in_insert = Boundaries::GRAPHEME;
        let in_select =
            Boundaries::LINE_FIRST | Boundaries::LINE_LAST | Boundaries::LONG_WORD_START;

        match self.virus.mode {
            Mode::Normal { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .left(
                        (select == Select::None)
                            .then_some(in_normal)
                            .unwrap_or(in_select),
                        repeat,
                        wrap,
                    )
                    .collapse(select == Select::None);

                self.virus.ensure_visibility();
            }
            Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .left(
                        (select == Select::None)
                            .then_some(in_insert)
                            .unwrap_or(in_select),
                        repeat,
                        wrap,
                    )
                    .collapse(select == Select::None);

                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    // MOVE right

    fn move_end(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .end(blank)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                *selected = haystacks.len().saturating_sub(1);
            }
        }
    }

    fn move_right_boundary(
        &mut self,
        boundaries: usize,
        punctuation_start: bool,
        punctuation_end: bool,
        short_word_start: bool,
        short_word_end: bool,
        long_word_start: bool,
        long_word_end: bool,
        wrap: bool,
    ) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .right(
                        Boundaries::from_bools(
                            punctuation_start,
                            punctuation_end,
                            short_word_start,
                            short_word_end,
                            long_word_start,
                            long_word_end,
                        ),
                        boundaries,
                        wrap,
                    )
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    fn move_right_char(&mut self, chars: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .right(Boundaries::GRAPHEME, chars, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    fn move_right_smart(&mut self, repeat: usize, wrap: bool) {
        let in_normal = Boundaries::PUNCTUATION | Boundaries::WORD;
        let in_insert = Boundaries::GRAPHEME;
        let in_select = Boundaries::LINE_FIRST | Boundaries::LINE_LAST | Boundaries::LONG_WORD_END;

        match self.virus.mode {
            Mode::Normal { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .right(
                        (select == Select::None)
                            .then_some(in_normal)
                            .unwrap_or(in_select),
                        repeat,
                        wrap,
                    )
                    .collapse(select == Select::None);

                self.virus.ensure_visibility();
            }
            Mode::Insert { select } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .right(
                        (select == Select::None)
                            .then_some(in_insert)
                            .unwrap_or(in_select),
                        repeat,
                        wrap,
                    )
                    .collapse(select == Select::None);

                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    //
    // SCROLL
    //

    // SCROLL up

    fn scroll_top(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { select } => {}
            Mode::Insert { select } => {}
            Mode::Files => {}
        }
    }

    fn scroll_up_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } => {}
            Mode::Insert { select } => {}
            Mode::Files => {}
        }
    }

    fn scroll_up_line(&mut self, lines: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } => {}
            Mode::Insert { select } => {}
            Mode::Files => {}
        }
    }

    // SCROLL down

    fn scroll_bottom(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { select } => {}
            Mode::Insert { select } => {}
            Mode::Files => {}
        }
    }

    fn scroll_down_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } => {}
            Mode::Insert { select } => {}
            Mode::Files => {}
        }
    }

    fn scroll_down_line(&mut self, lines: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } => {}
            Mode::Insert { select } => {}
            Mode::Files => {}
        }
    }

    //
    // Selection
    //

    fn select(&mut self, lines: bool) {
        match &mut self.virus.mode {
            Mode::Normal { select } => *select = if lines { Select::Lines } else { Select::Range },
            Mode::Insert { select } => *select = if lines { Select::Lines } else { Select::Range },
            Mode::Files => {}
        }
    }

    fn select_smart(&mut self) {
        match &mut self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let flip = *select == Select::Lines;

                *select = match select {
                    Select::None => Select::Range,
                    Select::Range => Select::Lines,
                    Select::Lines => Select::Range,
                };

                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .flip(flip);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    fn flip_selection(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .flip(true);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    fn unselect(&mut self, anchor: bool) {
        match &mut self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                *select = Select::None;

                self.virus
                    .unwrap_active_document_mut()
                    .movements()
                    .flip(anchor)
                    .collapse(true);
                self.virus.ensure_visibility();
            }
            Mode::Files => {}
        }
    }

    //
    // Modes
    //

    fn normal(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } => {}
            Mode::Insert { select } => self.virus.mode(Mode::Normal { select }),
            Mode::Files => self.virus.mode(Mode::Normal {
                select: Select::None,
            }),
        }
    }

    fn insert(&mut self) {
        match self.virus.mode {
            Mode::Normal { select } => self.virus.mode(Mode::Insert { select }),
            Mode::Insert { .. } => {}
            Mode::Files => self.virus.mode(Mode::Normal {
                select: Select::None,
            }),
        }
    }

    fn files(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let files = self
                    .virus
                    .editor
                    .files(true, false)
                    .filter_map(|file| file.as_os_str().to_str().map(|file| file.to_owned()))
                    .collect::<Vec<_>>();
                let haystacks = files
                    .iter()
                    .map(|file| (file.to_owned(), 0, Vec::new()))
                    .collect();

                self.virus.search = Some((String::new(), files, haystacks, 0));
                self.virus.mode(Mode::Files)
            }
            Mode::Files => {}
        }
    }

    //
    //
    //

    fn cut(&mut self) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let document = self.virus.unwrap_active_document_mut();

                debug_assert!(
                    select != Select::None
                        || (select == Select::None && document.selection().is_empty())
                );

                // Select whole lines if non-range selection
                match select {
                    Select::None | Select::Lines => {
                        let lines = document.lines_selection();
                        document.movements().selection(lines, false);
                    }
                    Select::Range => {}
                }

                // Cut and save into clipboard
                if let Some(edit) = document.edition().edit(Text::default(), false) {
                    self.virus.clipboard = Some(Clipboard {
                        select,
                        text: Text::from(edit.into_removed_and_inserted().0),
                    });
                }
            }
            Mode::Files => {}
        }
    }

    fn copy(&mut self) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let document = self.virus.unwrap_active_document_mut();

                debug_assert!(
                    select != Select::None
                        || (select == Select::None && document.selection().is_empty())
                );

                // Select whole lines if non-range selection
                let range = match select {
                    Select::None | Select::Lines => document.lines_selection().range(),
                    Select::Range => document.selection().range(),
                };

                // Copy into clipboard
                self.virus.clipboard = Some(Clipboard {
                    select,
                    text: Text::from(
                        document
                            .rope()
                            .byte_slice(range.start.index..range.end.index),
                    ),
                });
            }
            Mode::Files => {}
        }
    }

    fn paste(&mut self) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let Some(mut clipboard) = self.virus.clipboard.clone() else {
                    return;
                };
                let document = self.virus.unwrap_active_document_mut();
                let is_select = select != Select::None;

                debug_assert!(
                    select != Select::None
                        || (select == Select::None && document.selection().is_empty())
                );

                match (clipboard.select, select) {
                    (Select::None | Select::Lines, Select::None) => {
                        let line = document.selection().head.line;
                        let width = document.selection().head.width;

                        // Paste at the start of the next line
                        document
                            .movements()
                            .end(true)
                            .right(Boundaries::GRAPHEME, 1, false)
                            .collapse(true);
                        document.edition().edit(clipboard.text, is_select);

                        // Restore the cursor's width
                        let cursor =
                            Cursor::builder(document.rope().slice(..)).at_width(line + 1, width);
                        document.movements().head(cursor, false).collapse(true);
                    }
                    (Select::None | Select::Lines, Select::Range) => {
                        // Remove the trailing line break
                        // to fit nicely in the current range selection
                        clipboard.text.trailing_line_break(false);
                        document.edition().edit(clipboard.text, is_select);
                    }
                    (Select::None | Select::Lines, Select::Lines) => {
                        // Paste in the whole lines selection
                        let selection = document.lines_selection();
                        document.movements().selection(selection, false);
                        document.edition().edit(clipboard.text, is_select);

                        // Move forward cursor to the end of the last pasted line
                        document
                            .movements()
                            .flip(!selection.is_forward())
                            .left(Boundaries::GRAPHEME, 1, false)
                            .flip(!selection.is_forward());
                    }
                    (Select::Range, Select::None | Select::Range) => {
                        // Easy :)
                        document.edition().edit(clipboard.text, is_select);
                    }
                    (Select::Range, Select::Lines) => {
                        // Ensure a trailing line break
                        // to fit nicely in the current line selection
                        clipboard.text.trailing_line_break(true);

                        // Paste in the whole lines selection
                        let selection = document.lines_selection();
                        document.movements().selection(selection, false);
                        document.edition().edit(clipboard.text, is_select);

                        // Move forward cursor to the end of the last pasted line
                        document
                            .movements()
                            .flip(!selection.is_forward())
                            .left(Boundaries::GRAPHEME, 1, false)
                            .flip(!selection.is_forward());
                    }
                }
            }
            Mode::Files => {}
        }
    }

    fn undo(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.unwrap_active_document_mut().edition().undo();
            }
            Mode::Files => {}
        }
    }

    fn redo(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.unwrap_active_document_mut().edition().redo();
            }
            Mode::Files => {}
        }
    }

    fn open(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } => {}
            Mode::Insert { .. } => {}
            Mode::Files => {
                let (needle, files, haystacks, selected) = self.virus.search.as_mut().unwrap();

                let path = self.virus.editor.root().join(&haystacks[*selected].0);
                self.virus.editor.open(path).unwrap();
                self.virus.search = None;
                self.virus.mode(Mode::Normal {
                    select: Select::None,
                });
            }
        }
    }

    fn save(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.unwrap_active_document_mut().save().unwrap();
            }
            Mode::Files => {}
        }
    }

    fn close(&mut self) {
        self.event_loop.exit();
    }

    #[cfg(test)]
    fn test(&mut self, value: usize) {}
}
