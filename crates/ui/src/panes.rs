use crate::{theme::UiTheme, ui::LinesCache, views::DocumentView};
use std::{cell::RefCell, rc::Weak, time::Duration};
use virus_editor::{
    document::{Document, DocumentId},
    ids,
    mode::{Mode, Select},
};
use virus_graphics::{text::Context, types::Rectangle, wgpu::Graphics};

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
pub struct DocumentPane {
    pub pane_id: PaneId,
    pub region: Rectangle,
    pub document_id: DocumentId,
    pub document_view: DocumentView,
}

#[derive(Debug)]
pub enum Pane {
    Document(DocumentPane),
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

    pub fn get(&self, pane_id: PaneId) -> Option<&Pane> {
        let requested_pane_id = pane_id;

        self.panes.iter().find(|pane| match pane {
            Pane::Document(DocumentPane { pane_id, .. }) => requested_pane_id == *pane_id,
        })
    }

    pub fn get_mut(&mut self, pane_id: PaneId) -> Option<&mut Pane> {
        let requested_pane_id = pane_id;

        self.panes.iter_mut().find(|pane| match pane {
            Pane::Document(DocumentPane { pane_id, .. }) => requested_pane_id == *pane_id,
        })
    }

    pub fn panes(&self) -> &[Pane] {
        &self.panes
    }

    pub fn open_pane(&mut self, document_id: DocumentId) -> PaneId {
        let pane_id = self.pane_ids.id();
        self.panes.push(Pane::Document(DocumentPane {
            pane_id,
            region: Default::default(),
            document_id,
            document_view: DocumentView::new(self.theme.clone(), self.lines_cache.clone()),
        }));

        pane_id
    }

    pub fn is_animating(&self) -> bool {
        self.panes.iter().any(|pane| match pane {
            Pane::Document(DocumentPane { document_view, .. }) => document_view.is_animating(),
        })
    }

    pub fn resize(&mut self, region: Rectangle) {
        self.region = region;
    }

    pub fn update(&mut self, delta: Duration) {
        for pane in &mut self.panes {
            match pane {
                Pane::Document(DocumentPane { document_view, .. }) => document_view.update(delta),
            }
        }
    }

    pub fn render<'a>(
        &mut self,
        context: &mut Context,
        graphics: &mut Graphics,
        documents: impl Fn(DocumentId) -> Option<&'a Document>,
        active_pane_id: PaneId,
        mode: Mode,
    ) {
        let width = (self.region.width as f32 / self.panes.len() as f32).round() as u32;

        for (i, pane) in self.panes.iter_mut().enumerate() {
            match pane {
                Pane::Document(DocumentPane {
                    pane_id,
                    region,
                    document_id,
                    document_view,
                }) => {
                    let Some(document) = documents(*document_id) else {
                        continue;
                    };

                    *region = Rectangle {
                        top: self.region.top,
                        left: self.region.left + i as i32 * width as i32,
                        width,
                        height: self.region.height,
                    };
                    let mode = (*pane_id == active_pane_id)
                        .then_some(mode)
                        .unwrap_or_else(|| {
                            // We want the document to look the same when it will be active again
                            // Assuming this is the logic:
                            Mode::Normal {
                                select: if document.selection().is_empty() {
                                    Select::None
                                } else {
                                    Select::Range
                                },
                            }
                        });

                    document_view.render(context, &mut graphics.layer(*region, 0), document, mode);
                }
            }
        }
    }
}
