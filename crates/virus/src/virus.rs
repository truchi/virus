//! 🦠: a 😍 editor in 🦀 with ❤️
// Hello! -- the-world - _ hello- _ ____world - the----w __ salut HTTPProxyOfTheDeath23MORE123
// hello {((((world))))}, salut

use crate::{
    events::{Event, Events, Key, KeyEvent},
    keybindings::{ActionHandler, Keybindings},
};
use std::{sync::Arc, time::Instant};
use virus_editor::{
    add_in_range,
    document::Document,
    editor::{Editor, EventLoopMessage},
    fuzzy::Search,
    mode::{Mode, Select},
    rope::{Boundaries, Cursor, Text},
    sub_in_range,
};
use virus_ui::{
    panes::{DocumentPane, Pane},
    ui::Ui,
};
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

#[derive(Clone, Debug)]
pub struct Clipboard {
    select: Select,
    text: Text,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

struct FileSearch {
    selected: usize,
    needle: String,
    search: Search,
}

impl Default for FileSearch {
    fn default() -> Self {
        Self {
            selected: 0,
            needle: String::new(),
            search: Search::new_file_search(),
        }
    }
}

impl FileSearch {
    fn selected(&self) -> Option<&str> {
        self.search
            .matches()
            .get(self.selected)
            .map(|m| self.search.haystack()[m.index].as_str())
    }

    fn clear(&mut self) {
        self.selected = 0;
        self.needle.clear();
        self.search.clear();
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct Virus {
    events: Events,
    editor: Editor,
    mode: Mode,
    ui: Ui,
    last_render: Option<Instant>,
    file_search: FileSearch,
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
            let current_dir = std::env::current_dir().expect("Current directory");
            let root = Editor::find_git_root(current_dir.clone()).unwrap_or(current_dir);

            Editor::new(root)
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
        Self {
            events: Default::default(),
            editor,
            mode: Default::default(),
            ui: Ui::new(Arc::new(window)),
            last_render: Default::default(),
            file_search: Default::default(),
            keybindings: Default::default(),
            clipboard: Default::default(),
        }
    }

    fn get_active_document(&self) -> Option<(&DocumentPane, &Document)> {
        self.ui
            .panes()
            .active()
            .map(|pane| match pane {
                Pane::Document(pane) => (pane, pane.view.document_id()),
            })
            .map(|(pane, document_id)| {
                self.editor
                    .get_document(document_id)
                    .map(|document| (pane, document))
                    .unwrap()
            })
    }

    fn get_active_document_mut(&mut self) -> Option<(&DocumentPane, &mut Document)> {
        self.ui
            .panes()
            .active()
            .map(|pane| match pane {
                Pane::Document(pane) => (pane, pane.view.document_id()),
            })
            .map(|(pane, document_id)| {
                self.editor
                    .get_document_mut(document_id)
                    .map(|document| (pane, document))
                    .unwrap()
            })
    }

    // TODO ...
    fn unwrap_active_document_mut(&mut self) -> &mut Document {
        self.get_active_document_mut().unwrap().1
    }

    fn mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
            self.keybindings.mode(mode);
        }
    }

    fn ensure_visibility(&mut self) {
        let Some((pane, document)) = self.get_active_document() else {
            return;
        };

        let pane_id = pane.id;
        let line = document.selection().head.line as u32;
        let height_in_lines = pane.view.cells().height;
        let start = pane.view.line();
        let end = start + height_in_lines;

        let line = if line < start {
            line
        } else if line >= end {
            line + 1 - height_in_lines
        } else {
            return;
        };

        self.ui.panes_mut().scroll(pane_id, line);
    }
}

/// Event handlers.
impl Virus {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
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

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: EventLoopMessage) {}

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
                Mode::Insert { .. } => match event.modded() {
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
                        self.unwrap_active_document_mut()
                            .edition()
                            .edit("\n".into(), false);
                    }
                    _ => {}
                },
                Mode::Files => match event.modded().as_str() {
                    Key::Str(str) => {
                        self.file_search.selected = 0;
                        self.file_search.needle.push_str(str);
                        self.file_search.search.search(&self.file_search.needle);
                    }
                    Key::Space => {
                        self.file_search.selected = 0;
                        self.file_search.needle.push(' ');
                        self.file_search.search.search(&self.file_search.needle);
                    }
                    Key::Backspace => {
                        self.file_search.selected = 0;
                        self.file_search.needle.pop();

                        if self.file_search.needle.is_empty() {
                            self.file_search.search.reset();
                        } else {
                            self.file_search.search.search(&self.file_search.needle);
                        }
                    }
                    _ => {}
                },
            },
        }

        // TODO handle that better
        self.get_active_document_mut()
            .map(|(_, document)| document.parse());

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
        self.ui.render(
            |id| self.editor.get_document(id),
            self.mode,
            (self.mode == Mode::Files).then_some((
                self.file_search.selected,
                &self.file_search.needle,
                &self.file_search.search,
            )),
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
                self.virus.file_search.selected = 0;
            }
        }
    }

    fn move_up_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let lines = pages * pane.view.cells().height as usize / if half { 2 } else { 1 };
                document
                    .movements()
                    .up(lines, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let lines = 10; // TODO: we don't know the height in lines here...
                let lines = pages * lines / if half { 2 } else { 1 };

                self.virus.file_search.selected = sub_in_range(
                    self.virus.file_search.search.haystack().len(),
                    self.virus.file_search.selected,
                    lines,
                    wrap,
                );
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
                self.virus.file_search.selected = sub_in_range(
                    self.virus.file_search.search.haystack().len(),
                    self.virus.file_search.selected,
                    lines,
                    wrap,
                );
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
                self.virus.file_search.selected = self
                    .virus
                    .file_search
                    .search
                    .haystack()
                    .len()
                    .saturating_sub(1);
            }
        }
    }

    fn move_down_page(&mut self, pages: usize, half: bool, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { select } | Mode::Insert { select } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let lines = pages * pane.view.cells().height as usize / if half { 2 } else { 1 };
                document
                    .movements()
                    .down(lines, wrap)
                    .collapse(select == Select::None);
                self.virus.ensure_visibility();
            }
            Mode::Files => {
                let lines = 10; // TODO: we don't know the height in lines here...
                let lines = pages * lines / if half { 2 } else { 1 };

                self.virus.file_search.selected = add_in_range(
                    self.virus.file_search.search.haystack().len(),
                    self.virus.file_search.selected,
                    lines,
                    wrap,
                );
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
                self.virus.file_search.selected = add_in_range(
                    self.virus.file_search.search.haystack().len(),
                    self.virus.file_search.selected,
                    lines,
                    wrap,
                );
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
            Mode::Files => {}
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
                    .prev(
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
                    .prev(Boundaries::GRAPHEME, chars, wrap)
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
                    .prev(
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
                    .prev(
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
            Mode::Files => {}
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
                    .next(
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
                    .next(Boundaries::GRAPHEME, chars, wrap)
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
                    .next(
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
                    .next(
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
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let offset = blank
                    .then_some(0)
                    .unwrap_or_else(|| document.leading_blank_lines() as u32);
                let line = offset;

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    fn scroll_up_page(&mut self, pages: usize, half: bool, blank: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let offset = blank
                    .then_some(0)
                    .unwrap_or_else(|| document.leading_blank_lines() as u32);
                let line = pane
                    .view
                    .line()
                    .saturating_sub(pages as u32 * cells.height / if half { 2 } else { 1 });
                let line = line.max(offset);

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    fn scroll_up_line(&mut self, lines: usize, blank: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let offset = blank
                    .then_some(0)
                    .unwrap_or_else(|| document.leading_blank_lines() as u32);
                let line = pane.view.line().saturating_sub(lines as u32);
                let line = line.max(offset);

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    // SCROLL down

    fn scroll_bottom(&mut self, blank: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let offset = blank
                    .then_some(0)
                    .unwrap_or_else(|| document.trailing_blank_lines() as u32);
                let line = (document.rope().len_lines() as u32)
                    .saturating_sub(offset)
                    .saturating_sub(cells.height);

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    fn scroll_down_page(&mut self, pages: usize, half: bool, blank: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let offset = blank
                    .then_some(0)
                    .unwrap_or_else(|| document.trailing_blank_lines() as u32);
                let line =
                    pane.view.line() + pages as u32 * cells.height / if half { 2 } else { 1 };
                let line = line.min(
                    (document.rope().len_lines() as u32)
                        .saturating_sub(cells.height)
                        .saturating_sub(offset),
                );

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    fn scroll_down_line(&mut self, lines: usize, blank: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let offset = blank
                    .then_some(0)
                    .unwrap_or_else(|| document.trailing_blank_lines() as u32);
                let line = pane.view.line() + lines as u32;
                let line = line.min(
                    (document.rope().len_lines() as u32)
                        .saturating_sub(cells.height)
                        .saturating_sub(offset),
                );

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    // SCROLL align

    fn scroll_align_top(&mut self, margin: usize) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let line = document.selection().head.line as u32;
                let line = line.saturating_sub(margin as u32);
                let line =
                    line.min((document.rope().len_lines() as u32).saturating_sub(cells.height));

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    fn scroll_align_center(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let line = document.selection().head.line as u32;
                let line = line.saturating_sub(cells.height / 2);
                let line =
                    line.min((document.rope().len_lines() as u32).saturating_sub(cells.height));

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    fn scroll_align_bottom(&mut self, margin: usize) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let Some((pane, document)) = self.virus.get_active_document_mut() else {
                    return;
                };
                let pane_id = pane.id;
                let cells = pane.view.cells();

                if document.rope().len_lines() <= cells.height as usize {
                    return;
                }

                let line = document.selection().head.line as u32;
                let line = line.saturating_sub(cells.height.saturating_sub(1 + margin as u32));
                let line =
                    line.min((document.rope().len_lines() as u32).saturating_sub(cells.height));

                self.virus.ui.panes_mut().scroll(pane_id, line);
            }
            Mode::Files => {}
        }
    }

    //
    // SELECTION
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
    // PANES
    //

    // PANES open

    fn panes_open_first(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let document_id = self.virus.get_active_document().unwrap().1.id();
                self.virus.ui.panes_mut().open_first(document_id);
            }
            Mode::Files => {
                let Some(file) = self.virus.file_search.selected() else {
                    return;
                };

                self.virus.ui.panes_mut().open_first(
                    self.virus
                        .editor
                        .open(self.virus.editor.root().join(file))
                        .unwrap(),
                );
                self.virus.file_search.clear();
                self.virus.mode(Mode::Normal {
                    select: Select::None,
                });
            }
        }
    }

    fn panes_open_prev(&mut self, panes: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let document_id = self.virus.get_active_document().unwrap().1.id();
                self.virus
                    .ui
                    .panes_mut()
                    .open_prev(document_id, panes, wrap);
            }
            Mode::Files => {
                let Some(file) = self.virus.file_search.selected() else {
                    return;
                };

                self.virus.ui.panes_mut().open_prev(
                    self.virus
                        .editor
                        .open(self.virus.editor.root().join(file))
                        .unwrap(),
                    panes,
                    wrap,
                );
                self.virus.file_search.clear();
                self.virus.mode(Mode::Normal {
                    select: Select::None,
                });
            }
        }
    }

    fn panes_open_next(&mut self, panes: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let document_id = self.virus.get_active_document().unwrap().1.id();
                self.virus
                    .ui
                    .panes_mut()
                    .open_next(document_id, panes, wrap);
            }
            Mode::Files => {
                let Some(file) = self.virus.file_search.selected() else {
                    return;
                };

                self.virus.ui.panes_mut().open_next(
                    self.virus
                        .editor
                        .open(self.virus.editor.root().join(file))
                        .unwrap(),
                    panes,
                    wrap,
                );
                self.virus.file_search.clear();
                self.virus.mode(Mode::Normal {
                    select: Select::None,
                });
            }
        }
    }

    fn panes_open_last(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let document_id = self.virus.get_active_document().unwrap().1.id();
                self.virus.ui.panes_mut().open_last(document_id);
            }
            Mode::Files => {
                let Some(file) = self.virus.file_search.selected() else {
                    return;
                };

                self.virus.ui.panes_mut().open_last(
                    self.virus
                        .editor
                        .open(self.virus.editor.root().join(file))
                        .unwrap(),
                );
                self.virus.file_search.clear();
                self.virus.mode(Mode::Normal {
                    select: Select::None,
                });
            }
        }
    }

    // PANES close

    fn panes_close(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().close();
            }
            Mode::Files => {}
        }
    }

    fn panes_close_others(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().close_others();
            }
            Mode::Files => {}
        }
    }

    // PANES focus

    fn panes_focus_first(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().focus_first();
            }
            Mode::Files => {}
        }
    }

    fn panes_focus_prev(&mut self, panes: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().focus_prev(panes, wrap);
            }
            Mode::Files => {}
        }
    }

    fn panes_focus_next(&mut self, panes: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().focus_next(panes, wrap);
            }
            Mode::Files => {}
        }
    }

    fn panes_focus_last(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().focus_last();
            }
            Mode::Files => {}
        }
    }

    // PANES swap

    fn panes_swap_first(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().swap_first();
            }
            Mode::Files => {}
        }
    }

    fn panes_swap_prev(&mut self, panes: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().swap_prev(panes, wrap);
            }
            Mode::Files => {}
        }
    }

    fn panes_swap_next(&mut self, panes: usize, wrap: bool) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().swap_next(panes, wrap);
            }
            Mode::Files => {}
        }
    }

    fn panes_swap_last(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                self.virus.ui.panes_mut().swap_last();
            }
            Mode::Files => {}
        }
    }

    //
    // MODES
    //

    fn normal(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } => {}
            Mode::Insert { select } => self.virus.mode(Mode::Normal { select }),
            Mode::Files => {
                self.virus.file_search.clear();
                self.virus.mode(Mode::Normal {
                    select: Select::None,
                });
            }
        }
    }

    fn insert(&mut self) {
        match self.virus.mode {
            Mode::Normal { select } => self.virus.mode(Mode::Insert { select }),
            Mode::Insert { .. } => {}
            Mode::Files => {
                self.virus.file_search.clear();
                self.virus.mode(Mode::Insert {
                    select: Select::None,
                });
            }
        }
    }

    fn files(&mut self) {
        match self.virus.mode {
            Mode::Normal { .. } | Mode::Insert { .. } => {
                let files = self
                    .virus
                    .editor
                    .files(true, false)
                    .filter_map(|file| file.as_os_str().to_str().map(|file| file.to_owned()));

                self.virus.file_search.clear();
                self.virus.file_search.search.set_haystack(files);
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
                            .next(Boundaries::GRAPHEME, 1, false)
                            .collapse(true);
                        document.edition().edit(clipboard.text, is_select);

                        // Restore the cursor's width
                        let cursor =
                            Cursor::build(document.rope().slice(..)).at_width(line + 1, width);
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
                            .prev(Boundaries::GRAPHEME, 1, false)
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
                            .prev(Boundaries::GRAPHEME, 1, false)
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
    fn test(&mut self, _: usize) {}
}
