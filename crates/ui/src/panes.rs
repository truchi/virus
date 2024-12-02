use crate::{theme::UiTheme, ui::LinesCache, views::DocumentView};
use std::{cell::RefCell, rc::Weak, time::Duration};
use virus_editor::{
    document::{Document, DocumentId},
    ids,
};
use virus_graphics::{
    text::Context,
    types::{Rectangle, Rgba},
    wgpu::Graphics,
};

ids!(
    /// [`PaneId`] generator.
    pub PaneIds,
    /// Pane id.
    pub PaneId,
);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Pane                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub enum Pane {
    Document {
        pane_id: PaneId,
        document_id: DocumentId,
        document_view: DocumentView,
    },
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Panes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Panes {
    pane_ids: PaneIds,
    panes: Vec<Pane>,
    region: Rectangle,
    theme: Weak<RefCell<UiTheme>>,
    lines_cache: Weak<RefCell<LinesCache>>,
}

impl Panes {
    pub fn new(theme: Weak<RefCell<UiTheme>>, lines_cache: Weak<RefCell<LinesCache>>) -> Self {
        Self {
            pane_ids: Default::default(),
            panes: Default::default(),
            region: Default::default(),
            theme,
            lines_cache,
        }
    }

    pub fn panes(&self) -> &[Pane] {
        &self.panes
    }

    pub fn open_pane(&mut self, document_id: DocumentId) -> PaneId {
        let pane_id = self.pane_ids.id();
        self.panes.push(Pane::Document {
            pane_id,
            document_id,
            document_view: DocumentView::new(self.theme.clone(), self.lines_cache.clone()),
        });

        pane_id
    }

    pub fn is_animating(&self) -> bool {
        self.panes.iter().any(|pane| match pane {
            Pane::Document { document_view, .. } => document_view.is_animating(),
        })
    }

    pub fn resize(&mut self, region: Rectangle) {
        self.region = region;
    }

    pub fn update(&mut self, delta: Duration) {
        for pane in &mut self.panes {
            match pane {
                Pane::Document { document_view, .. } => document_view.update(delta),
            }
        }
    }

    pub fn render<'a>(
        &mut self,
        context: &mut Context,
        graphics: &mut Graphics,
        documents: impl Fn(DocumentId) -> Option<&'a Document>,
        show_selection_as_lines: bool,
        outline_colors: &[Rgba],
        caret_color: Rgba,
        caret_width: u32,
        selection_color: Rgba,
    ) {
        let width = (self.region.width as f32 / self.panes.len() as f32).round() as u32;

        for (i, pane) in self.panes.iter_mut().enumerate() {
            match pane {
                Pane::Document {
                    document_id,
                    document_view,
                    ..
                } => {
                    let Some(document) = documents(*document_id) else {
                        continue;
                    };

                    let region = Rectangle {
                        top: self.region.top,
                        left: self.region.left + i as i32 * width as i32,
                        width,
                        height: self.region.height,
                    };

                    document_view.render(
                        context,
                        &mut graphics.layer(region, 0),
                        document,
                        show_selection_as_lines,
                        outline_colors,
                        caret_color,
                        caret_width,
                        selection_color,
                    );
                }
            }
        }
    }
}
