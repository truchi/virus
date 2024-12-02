use virus_editor::{document::DocumentId, ids};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Panes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

ids!(
    /// [`PaneId`] generator.
    pub PaneIds,
    /// Pane id.
    pub PaneId,
);

#[derive(Default, Debug)]
pub struct Panes {
    pane_ids: PaneIds,
    panes: Vec<(PaneId, DocumentId)>,
}

impl Panes {
    pub fn panes(&self) -> &[(PaneId, DocumentId)] {
        &self.panes
    }

    pub fn open_pane(&mut self, document_id: DocumentId) -> PaneId {
        let pane_id = self.pane_ids.id();
        self.panes.push((pane_id, document_id));

        pane_id
    }
}
