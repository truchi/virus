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
    editor::{Editor, EventLoopMessage},
    fuzzy::Fuzzy,
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
pub enum Mode {
    #[default]
    Normal,
    Select,
    Lines,
    Insert,
    Files,
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
        }
    }

    fn mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.keybindings.mode(mode);
    }

    fn ensure_visibility(&mut self) {
        self.ui.ensure_visibility(
            self.editor
                .get_active_document()
                .unwrap()
                .selection()
                .head
                .line,
        );
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
                Mode::Normal => {}
                Mode::Select => {}
                Mode::Lines => {}
                Mode::Insert => {}
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
        self.editor.get_active_document_mut().unwrap().parse();

        // TODO handle that better
        self.ui.window().request_redraw();

        return;

        match &mut self.mode {
            Mode::Normal => match event.modded().as_str() {
                // TODO
                // Key::Str("v") => match select_mode {
                //     Some(SelectMode::Range) => *select_mode = Some(SelectMode::Line),
                //     Some(SelectMode::Line) => {
                //         self.editor
                //             .get_active_document_mut()
                //             .unwrap()
                //             .flip_anchor_and_head();
                //         *select_mode = Some(SelectMode::Range);
                //     }
                //     None => *select_mode = Some(SelectMode::Range),
                // },
                // Key::Str("V") => {
                //     // *select_mode = None; // TODO
                //     self.editor
                //         .get_active_document_mut()
                //         .unwrap()
                //         .move_anchor_to_head();
                // }
                Key::Escape => self.mode = Mode::Insert,
                _ => (),
            },
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => match event.modded().as_str() {
                Key::Str("@") if event.command() => event_loop.exit(),
                Key::Str(str) => self
                    .editor
                    .get_active_document_mut()
                    .unwrap()
                    .edit(str.into()),
                Key::Space => self
                    .editor
                    .get_active_document_mut()
                    .unwrap()
                    .edit(" ".into()),
                Key::Backspace => self.editor.get_active_document_mut().unwrap().backspace(),
                Key::Enter => self
                    .editor
                    .get_active_document_mut()
                    .unwrap()
                    .edit("\n".into()),
                Key::Escape => {
                    // TODO
                    // self.mode = Mode::Normal {
                    //     select_mode: (!self
                    //         .editor
                    //         .get_active_document()
                    //         .unwrap()
                    //         .selection()
                    //         .range()
                    //         .is_empty())
                    //     .then_some(SelectMode::Range),
                    // }
                }
                _ => (),
            },
            Mode::Files => {}
        }

        // TODO handle that better
        self.editor.get_active_document_mut().unwrap().parse();

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
        self.ui.render(
            self.editor.get_active_document_mut().unwrap(),
            matches!(self.mode, Mode::Lines),
            match self.mode {
                Mode::Normal => outline_normal_mode_colors,
                Mode::Select => outline_select_mode_colors,
                Mode::Lines => outline_select_mode_colors,
                Mode::Insert => outline_insert_mode_colors,
                Mode::Files => outline_select_mode_colors,
            },
            match self.mode {
                Mode::Normal => self.ui.theme().caret_normal_mode_color,
                Mode::Select => self.ui.theme().caret_select_mode_color,
                Mode::Lines => self.ui.theme().caret_select_mode_color,
                Mode::Insert => self.ui.theme().caret_insert_mode_color,
                Mode::Files => self.ui.theme().caret_normal_mode_color,
            },
            match self.mode {
                Mode::Normal => self.ui.theme().caret_normal_mode_width,
                Mode::Select => self.ui.theme().caret_select_mode_width,
                Mode::Lines => self.ui.theme().caret_select_mode_width,
                Mode::Insert => self.ui.theme().caret_insert_mode_width,
                Mode::Files => self.ui.theme().caret_normal_mode_width,
            },
            match self.mode {
                Mode::Normal => self.ui.theme().selection_select_mode_color,
                Mode::Select => self.ui.theme().selection_select_mode_color,
                Mode::Lines => self.ui.theme().selection_select_mode_color,
                Mode::Insert => self.ui.theme().selection_insert_mode_color,
                Mode::Files => self.ui.theme().selection_select_mode_color,
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
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .top(blank)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {
                let (_, _, _, selected) = self.virus.search.as_mut().unwrap();
                *selected = 0;
            }
        }
    }

    fn move_up_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    let lines = self.virus.ui.screen_height_in_lines() as usize;
                    let lines = lines / if half { 2 } else { 1 };
                    document
                        .movements()
                        .up(pages * lines, wrap)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
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
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .up(lines, wrap)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
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
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .bottom(blank)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                *selected = haystacks.len().saturating_sub(1);
            }
        }
    }

    fn move_down_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    let lines = self.virus.ui.screen_height_in_lines() as usize;
                    let lines = lines / if half { 2 } else { 1 };
                    document
                        .movements()
                        .down(pages * lines, wrap)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
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
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .down(lines, wrap)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
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
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .start(blank)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {
                let (_, _, _, selected) = self.virus.search.as_mut().unwrap();
                *selected = 0;
            }
        }
    }

    // TODO: that could be nice to have:
    // - symbols=false: ignore non-alphanum words
    // - sub=false: WORD/longword
    //
    // NOTE: that could also be super nice to have a "smart word":
    // - in normal mode: stop at every boudaries (symbols, sub, start, end)
    // - in select mode:
    //   - no symbols, no sub
    //   - growing selection moves to growing end
    //   - shriking selection moves to shriking end
    //   - pairs?
    // Or syntaxically...
    fn move_left_word(&mut self, words: usize, symbols: bool, sub: bool, end: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    let mut movements = document.movements();
                    match (symbols, sub, end) {
                        (true, true, true) => movements.prev_end_of_subword(words, wrap),
                        (true, true, false) => movements.prev_start_of_subword(words, wrap),
                        _ => todo!(),
                    }
                    .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {}
        }
    }

    fn move_left_char(&mut self, chars: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .prev_grapheme(chars, wrap)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {}
        }
    }

    // MOVE right

    fn move_end(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .end(blank)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {
                let (_, _, haystacks, selected) = self.virus.search.as_mut().unwrap();
                *selected = haystacks.len().saturating_sub(1);
            }
        }
    }

    fn move_right_word(&mut self, words: usize, symbols: bool, sub: bool, end: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    let mut movements = document.movements();
                    match (symbols, sub, end) {
                        (true, true, true) => movements.next_end_of_subword(words, wrap),
                        (true, true, false) => movements.next_start_of_subword(words, wrap),
                        _ => todo!(),
                    }
                    .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
            }
            Mode::Files => {}
        }
    }

    fn move_right_char(&mut self, chars: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {
                if let Some(document) = self.virus.editor.get_active_document_mut() {
                    document
                        .movements()
                        .next_grapheme(chars, wrap)
                        .collapse(self.virus.mode == Mode::Normal);
                    self.virus.ensure_visibility();
                }
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
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn scroll_up_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn scroll_up_line(&mut self, lines: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {}
            Mode::Files => {}
        }
    }

    // SCROLL down

    fn scroll_bottom(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn scroll_down_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn scroll_down_line(&mut self, lines: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal | Mode::Select | Mode::Lines | Mode::Insert => {}
            Mode::Files => {}
        }
    }

    //
    //
    //

    fn copy(&mut self) {
        match self.virus.mode {
            Mode::Normal => self.virus.editor.copy(),
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn paste(&mut self) {
        match self.virus.mode {
            Mode::Normal => self.virus.editor.paste(),
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn undo(&mut self) {
        match self.virus.mode {
            Mode::Normal => self.virus.editor.get_active_document_mut().unwrap().undo(),
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn redo(&mut self) {
        match self.virus.mode {
            Mode::Normal => self.virus.editor.get_active_document_mut().unwrap().redo(),
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn select(&mut self) {
        match self.virus.mode {
            Mode::Normal => self.virus.mode(Mode::Select),
            Mode::Select => self.virus.mode(Mode::Lines),
            Mode::Lines => {
                self.virus
                    .editor
                    .get_active_document_mut()
                    .unwrap()
                    .movements()
                    .flip(true);
                self.virus.mode(Mode::Select);
            }
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn unselect(&mut self) {
        match self.virus.mode {
            Mode::Normal => {}
            Mode::Select | Mode::Lines => {
                self.virus
                    .editor
                    .get_active_document_mut()
                    .unwrap()
                    .movements()
                    .collapse(true);
                self.virus.mode(Mode::Normal);
            }
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn files(&mut self) {
        match self.virus.mode {
            Mode::Normal => {
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
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn open(&mut self) {
        match self.virus.mode {
            Mode::Normal => {}
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {
                let (needle, files, haystacks, selected) = self.virus.search.as_mut().unwrap();

                let path = self.virus.editor.root().join(&haystacks[*selected].0);
                self.virus.editor.open(path).unwrap();
                self.virus.search = None;
                self.virus.mode(Mode::Normal)
            }
        }
    }

    fn save(&mut self) {
        match self.virus.mode {
            Mode::Normal => self
                .virus
                .editor
                .get_active_document_mut()
                .unwrap()
                .save()
                .unwrap(),
            Mode::Select => {}
            Mode::Lines => {}
            Mode::Insert => {}
            Mode::Files => {}
        }
    }

    fn close(&mut self) {
        self.event_loop.exit();
    }

    #[cfg(test)]
    fn test(&mut self, value: usize) {}
}
