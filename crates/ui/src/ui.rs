use crate::{
    panes::{DocumentPane, Pane, PaneId, Panes},
    views::{FilesView, StatusView},
    Context,
};
use std::{sync::Arc, time::Duration};
use virus_editor::{
    add_in_range,
    document::{Document, DocumentId},
    fuzzy::Search,
    mode::Mode,
    sub_in_range,
};
use virus_graphics::{
    types::{Rectangle, Size},
    wgpu::Graphics,
};
use winit::window::Window;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                 Ui                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Ui {
    window: Arc<Window>,
    context: Context,
    graphics: Graphics,
    panes: Panes,
    files: FilesView,
    status: StatusView,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let graphics = Graphics::new(Arc::clone(&window));
        let context = {
            let fonts = crate::todo::fonts();
            let theme = crate::todo::ui_theme(&fonts);

            Context {
                fonts,
                shape: Default::default(),
                scale: Default::default(),
                theme,
                highlighteds: Default::default(),
            }
        };

        Self {
            window,
            context,
            graphics,
            panes: Panes::new(),
            files: FilesView::new(),
            status: StatusView::new(),
        }
    }

    pub fn is_animating(&self) -> bool {
        self.panes.is_animating()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn context(&self) -> &Context {
        &self.context
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
        keybindings: &[String],
        file_name: Option<(String, bool)>,
    ) {
        let theme = self.context.theme;
        let window = {
            let window = self.window.inner_size();
            Size::new(window.width as f32, window.height as f32)
        };
        let region = {
            let size = Size {
                width: window.width,
                height: window.height - theme.line_height,
            };
            let pixels = theme.pixels(size);

            Rectangle {
                top: (size.height - pixels.height) / 2.0,
                left: (size.width - pixels.width) / 2.0,
                width: pixels.width,
                height: pixels.height,
            }
        };
        let status_region = Rectangle {
            top: region.top + region.height,
            left: 0.0,
            width: window.width,
            height: theme.line_height,
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

        self.status.render(
            &mut self.context,
            self.graphics.layer(status_region, 0),
            mode,
            keybindings,
            file_name,
        );

        self.graphics.render(self.context.theme.background_color);
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

    pub fn get_document(&self, document_id: DocumentId) -> Option<&'ui DocumentPane> {
        self.ui.panes.panes().iter().find_map(|pane| match pane {
            Pane::Document(pane) if pane.view.document_id() == document_id => Some(pane),
            _ => None,
        })
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

    pub fn scroll(&mut self, pane_id: PaneId, line: usize) {
        let top = line as f32 * self.ui.context.theme.line_height;
        let tween = self.ui.context.theme.scroll_tween;
        let duration = self.ui.context.theme.scroll_duration;

        self.ui.panes.get_mut(pane_id).map(|pane| match pane {
            Pane::Document(DocumentPane { view, .. }) => view.scroll(top, tween, duration),
        });
    }
}
