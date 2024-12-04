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
    pub id: PaneId,
    pub view: DocumentView,
}

#[derive(Debug)]
pub enum Pane {
    Document(DocumentPane),
}

impl Pane {
    pub fn id(&self) -> PaneId {
        match self {
            Pane::Document(pane) => pane.id,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Panes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Panes {
    pane_ids: PaneIds,
    active_pane_id: Option<PaneId>,
    panes: Vec<Pane>,
    theme: Weak<RefCell<UiTheme>>,
    lines_cache: Weak<RefCell<LinesCache>>,
}

impl Panes {
    pub const ACTIVE_PANE_COLUMNS: u32 = 100;

    pub fn new(theme: Weak<RefCell<UiTheme>>, lines_cache: Weak<RefCell<LinesCache>>) -> Self {
        Self {
            pane_ids: Default::default(),
            active_pane_id: Default::default(),
            panes: Default::default(),
            theme,
            lines_cache,
        }
    }

    pub fn is_animating(&self) -> bool {
        self.panes.iter().any(|pane| match pane {
            Pane::Document(DocumentPane { view, .. }) => view.is_animating(),
        })
    }

    pub fn position(&self, pane_id: PaneId) -> Option<usize> {
        self.panes.iter().position(|pane| pane.id() == pane_id)
    }

    pub fn active_position(&self) -> Option<usize> {
        self.active_pane_id.and_then(|active_pane_id| {
            self.panes
                .iter()
                .position(|pane| pane.id() == active_pane_id)
        })
    }

    pub fn get_active_pane_id(&self) -> Option<PaneId> {
        debug_assert!(if let Some(active_pane_id) = self.active_pane_id {
            self.get(active_pane_id).is_some()
        } else {
            true
        });

        self.active_pane_id
    }

    pub fn set_active_pane_id(&mut self, active_pane_id: Option<PaneId>) {
        debug_assert!(if let Some(active_pane_id) = active_pane_id {
            self.get(active_pane_id).is_some()
        } else {
            true
        });

        self.active_pane_id = active_pane_id;
    }

    pub fn panes(&self) -> &[Pane] {
        &self.panes
    }

    pub fn panes_mut(&mut self) -> &mut Vec<Pane> {
        &mut self.panes
    }

    pub fn get(&self, pane_id: PaneId) -> Option<&Pane> {
        self.panes.iter().find(|pane| match pane {
            Pane::Document(DocumentPane { id, .. }) => pane_id == *id,
        })
    }

    pub fn get_mut(&mut self, pane_id: PaneId) -> Option<&mut Pane> {
        self.panes.iter_mut().find(|pane| match pane {
            Pane::Document(DocumentPane { id, .. }) => pane_id == *id,
        })
    }

    pub fn open(&mut self, index: usize, document_id: DocumentId) -> PaneId {
        debug_assert!(index <= self.panes.len());

        let pane = DocumentPane {
            id: self.pane_ids.id(),
            view: DocumentView::new(document_id, self.theme.clone(), self.lines_cache.clone()),
        };
        let pane_id = pane.id;

        self.panes.insert(index, Pane::Document(pane));

        pane_id
    }

    pub fn update(&mut self, delta: Duration) {
        for pane in &mut self.panes {
            match pane {
                Pane::Document(DocumentPane { view, .. }) => view.update(delta),
            }
        }
    }

    pub fn render<'a>(
        &mut self,
        context: &mut Context,
        graphics: &mut Graphics,
        region: Rectangle,
        documents: impl Fn(DocumentId) -> Option<&'a Document>,
        mode: Mode,
    ) {
        // TODO when enough space

        let len = self.panes.len() as u32;
        let theme = *self.theme.upgrade().unwrap().borrow();
        let (active_pane_width, inactive_pane_width) = {
            let columns = theme.cells_and_pixels(region.size()).0.width;
            let active_pane_columns =
                columns.min(Self::ACTIVE_PANE_COLUMNS + DocumentView::GUTTER_COLUMNS);
            let inactive_pane_columns = if self.active_pane_id.is_some() {
                match len {
                    0 => unreachable!(),
                    1 => 0,
                    _ => columns.saturating_sub(active_pane_columns) / (len - 1),
                }
            } else {
                match len {
                    0 => 0,
                    _ => columns / len,
                }
            };

            (
                (active_pane_columns as f32 * theme.advance).ceil() as u32,
                (inactive_pane_columns as f32 * theme.advance).ceil() as u32,
            )
        };
        let mut left = region.left + {
            let width = if self.active_pane_id.is_some() {
                active_pane_width + inactive_pane_width * (len - 1)
            } else {
                inactive_pane_width * len
            };

            region.width.saturating_sub(width) / 2
        } as i32;

        for pane in &mut self.panes {
            match pane {
                Pane::Document(DocumentPane { id, view }) => {
                    let Some(document) = documents(view.document_id()) else {
                        continue;
                    };
                    let region = {
                        let width = (self.active_pane_id == Some(*id))
                            .then_some(active_pane_width)
                            .unwrap_or(inactive_pane_width);
                        let region = Rectangle {
                            top: region.top,
                            left,
                            width,
                            height: region.height,
                        };

                        left += width as i32;
                        region
                    };
                    let mode = (self.active_pane_id == Some(*id))
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

                    view.render(context, &mut graphics.layer(region, 0), document, mode);
                }
            }
        }
    }
}
