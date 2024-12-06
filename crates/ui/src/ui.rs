use crate::{
    panes::{DocumentPane, Pane, PaneId, Panes},
    syntax::Lines,
    theme::UiTheme,
    views::FilesView,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc, time::Duration};
use virus_editor::{
    add_in_range,
    document::{Document, DocumentId},
    fuzzy::Search,
    mode::Mode,
    sub_in_range,
};
use virus_graphics::{
    text::Context,
    types::{Rectangle, Size},
    wgpu::Graphics,
};
use winit::window::Window;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                 Ui                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub(crate) type LinesCache = HashMap<DocumentId, Lines>;

pub struct Ui {
    window: Arc<Window>,
    graphics: Graphics,
    context: Context,
    theme: Rc<RefCell<UiTheme>>,
    panes: Panes,
    files: FilesView,
    _lines_cache: Rc<RefCell<LinesCache>>,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let graphics = Graphics::new(Arc::clone(&window));
        let context = Context::new(crate::todo::fonts());
        let theme = Rc::new(RefCell::new(crate::todo::ui_theme(&context)));
        let lines_cache = Default::default();
        let files = FilesView::new(Rc::downgrade(&theme));
        let panes = Panes::new(Rc::downgrade(&theme), Rc::downgrade(&lines_cache));

        Self {
            window,
            graphics,
            context,
            theme,
            panes,
            files,
            _lines_cache: lines_cache,
        }
    }

    pub fn is_animating(&self) -> bool {
        self.panes.is_animating()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn theme(&self) -> UiTheme {
        *self.theme.borrow()
    }

    pub fn panes(&self) -> UiPanes {
        UiPanes { ui: self }
    }

    pub fn panes_mut(&mut self) -> UiPanesMut {
        UiPanesMut { ui: self }
    }

    /// Resizes the graphics canvas.
    ///
    /// Does not resizes the views. Views report their last renderd sizes.
    pub fn resize(&mut self) {
        self.graphics.resize(&self.window);
    }

    pub fn update(&mut self, delta: Duration) {
        self.panes.update(delta);
    }

    pub fn render<'a>(
        &mut self,
        documents: impl Fn(DocumentId) -> Option<&'a Document>,
        mode: Mode,
        file_search: Option<(usize, &'a str, &Search)>,
    ) {
        // TODO react to document closes

        let region = {
            let size = self.window.inner_size();
            let size = Size {
                width: size.width,
                height: size.height,
            };
            let (_, pixels) = self.theme().cells_and_pixels(size);

            Rectangle {
                top: (size.height - pixels.height) as i32 / 2,
                left: (size.width - pixels.width) as i32 / 2,
                width: pixels.width,
                height: pixels.height,
            }
        };

        self.panes.render(
            &mut self.context,
            &mut self.graphics,
            region,
            documents,
            mode,
        );

        if let Some((needle, haystack, selected)) = file_search {
            self.files.render(
                &mut self.context,
                self.graphics.layer(region, 1),
                needle,
                haystack,
                selected,
            );
        }

        self.graphics.render(self.theme().background_color);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            UiPanes                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct UiPanes<'ui> {
    ui: &'ui Ui,
}

impl<'ui> UiPanes<'ui> {
    pub fn active(&self) -> Option<&'ui Pane> {
        self.ui
            .panes
            .get_active_id()
            .map(|id| self.ui.panes.get(id))
            .flatten()
    }

    pub fn get(&self, pane_id: PaneId) -> Option<&'ui Pane> {
        self.ui.panes.get(pane_id)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           UiPanesMut                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct UiPanesMut<'ui> {
    ui: &'ui mut Ui,
}

impl<'ui> UiPanesMut<'ui> {
    pub fn open_first(&mut self, document_id: DocumentId) -> PaneId {
        self.ui.panes.open(0, document_id)
    }

    pub fn open_last(&mut self, document_id: DocumentId) -> PaneId {
        self.ui.panes.open(self.ui.panes.panes().len(), document_id)
    }

    pub fn open_prev(&mut self, document_id: DocumentId, panes: usize, wrap: bool) -> PaneId {
        self.ui.panes.open(
            sub_in_range(
                self.ui.panes.panes().len(),
                self.ui.panes.active_position().unwrap_or_default(),
                panes,
                wrap,
            ),
            document_id,
        )
    }

    pub fn open_next(&mut self, document_id: DocumentId, panes: usize, wrap: bool) -> PaneId {
        self.ui.panes.open(
            add_in_range(
                self.ui.panes.panes().len(),
                self.ui
                    .panes
                    .active_position()
                    .unwrap_or_else(|| self.ui.panes.panes().len()),
                panes,
                wrap,
            ),
            document_id,
        )
    }

    pub fn close(&mut self) {
        if let Some(position) = self.ui.panes.active_position() {
            self.ui.panes.panes_mut().remove(position);
            self.ui.panes.set_active_id(
                position
                    .checked_sub(1)
                    .map(|prev| self.ui.panes.panes()[prev].id())
                    .or_else(|| self.ui.panes.panes().get(position).map(Pane::id)),
            );
        }
    }

    pub fn close_others(&mut self) {
        let active = self
            .ui
            .panes
            .active_position()
            .map(|position| self.ui.panes.panes_mut().swap_remove(position));

        self.ui.panes.panes_mut().clear();
        active.map(|active| self.ui.panes.panes_mut().push(active));
    }

    pub fn focus_first(&mut self) {
        self.ui
            .panes
            .set_active_id(self.ui.panes.panes().first().map(Pane::id));
    }

    pub fn focus_last(&mut self) {
        self.ui
            .panes
            .set_active_id(self.ui.panes.panes().last().map(Pane::id));
    }

    pub fn focus_prev(&mut self, panes: usize, wrap: bool) {
        let position = sub_in_range(
            self.ui.panes.panes().len(),
            self.ui.panes.active_position().unwrap_or_default(),
            panes,
            wrap,
        );

        self.ui
            .panes
            .set_active_id(Some(self.ui.panes.panes()[position].id()));
    }

    pub fn focus_next(&mut self, panes: usize, wrap: bool) {
        let position = add_in_range(
            self.ui.panes.panes().len(),
            self.ui
                .panes
                .active_position()
                .unwrap_or_else(|| self.ui.panes.panes().len()),
            panes,
            wrap,
        );

        self.ui
            .panes
            .set_active_id(Some(self.ui.panes.panes()[position].id()));
    }

    pub fn swap_first(&mut self) {
        if let Some(position) = self.ui.panes.active_position() {
            self.ui.panes.panes_mut().swap(position, 0);
        }
    }

    pub fn swap_last(&mut self) {
        if let Some(position) = self.ui.panes.active_position() {
            let len = self.ui.panes.panes().len();
            self.ui.panes.panes_mut().swap(position, len);
        }
    }

    pub fn swap_prev(&mut self, panes: usize, wrap: bool) {
        if let Some(position) = self.ui.panes.active_position() {
            let len = self.ui.panes.panes().len();
            self.ui
                .panes
                .panes_mut()
                .swap(position, sub_in_range(len, position, panes, wrap));
        }
    }

    pub fn swap_next(&mut self, panes: usize, wrap: bool) {
        if let Some(position) = self.ui.panes.active_position() {
            let len = self.ui.panes.panes().len();
            self.ui
                .panes
                .panes_mut()
                .swap(position, add_in_range(len, position, panes, wrap));
        }
    }

    pub fn scroll(&mut self, pane_id: PaneId, line: u32) {
        let top = line * self.ui.theme().line_height;

        self.ui.panes.get_mut(pane_id).map(|pane| match pane {
            Pane::Document(DocumentPane { view, .. }) => view.scroll(top),
        });
    }
}
