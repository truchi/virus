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
    ids: PaneIds,
    active_id: Option<PaneId>,
    panes: Vec<Pane>,
    theme: Weak<RefCell<UiTheme>>,
    lines_cache: Weak<RefCell<LinesCache>>,
}

impl Panes {
    pub const ACTIVE_COLUMNS: u32 = 100;

    pub fn new(theme: Weak<RefCell<UiTheme>>, lines_cache: Weak<RefCell<LinesCache>>) -> Self {
        Self {
            ids: Default::default(),
            active_id: Default::default(),
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
        self.active_id
            .and_then(|active_id| self.panes.iter().position(|pane| pane.id() == active_id))
    }

    pub fn get_active_id(&self) -> Option<PaneId> {
        debug_assert!(if let Some(active_id) = self.active_id {
            self.get(active_id).is_some()
        } else {
            true
        });

        self.active_id
    }

    pub fn set_active_id(&mut self, active_id: Option<PaneId>) {
        debug_assert!(if let Some(active_id) = active_id {
            self.get(active_id).is_some()
        } else {
            true
        });

        self.active_id = active_id;
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
            id: self.ids.id(),
            view: DocumentView::new(document_id, self.theme.clone(), self.lines_cache.clone()),
        };
        let pane_id = pane.id;

        self.panes.insert(index, Pane::Document(pane));
        self.set_active_id(Some(pane_id));

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
        let len = self.panes.len() as u32;
        let theme = *self.theme.upgrade().unwrap().borrow();
        let region_columns = theme.cells_and_pixels(region.size()).0.width;
        let columns_to_pixels = |columns: u32| (columns as f32 * theme.advance).ceil() as u32;
        let desired_columns = Self::ACTIVE_COLUMNS + DocumentView::GUTTER_COLUMNS;

        let (active_width, inactive_width) = if len * desired_columns < region_columns {
            (
                columns_to_pixels(desired_columns),
                columns_to_pixels(desired_columns),
            )
        } else {
            let active_columns = region_columns.min(desired_columns);
            let inactive_columns = if self.active_id.is_some() {
                match len {
                    0 => unreachable!(),
                    1 => 0,
                    _ => region_columns.saturating_sub(active_columns) / (len - 1),
                }
            } else {
                match len {
                    0 => 0,
                    _ => region_columns / len,
                }
            };

            (
                columns_to_pixels(active_columns),
                columns_to_pixels(inactive_columns),
            )
        };
        let margin = {
            let remaining_width = region.width.saturating_sub(if self.active_id.is_some() {
                active_width + inactive_width * (len - 1)
            } else {
                inactive_width * len
            });

            // Space evenly
            (remaining_width / (len + 1)) as i32
        };

        let mut left = region.left + margin as i32;

        for pane in &mut self.panes {
            match pane {
                Pane::Document(DocumentPane { id, view }) => {
                    let Some(document) = documents(view.document_id()) else {
                        continue;
                    };
                    let is_active = self.active_id == Some(*id);
                    let region = {
                        let width = is_active.then_some(active_width).unwrap_or(inactive_width);
                        let region = Rectangle {
                            top: region.top,
                            left,
                            width,
                            height: region.height,
                        };

                        left += width as i32 + margin;
                        region
                    };
                    let mode = is_active.then_some(mode).unwrap_or_else(|| {
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

                    view.render(
                        context,
                        &mut graphics.layer(region, 0),
                        document,
                        mode,
                        is_active,
                    );
                }
            }
        }
    }
}
