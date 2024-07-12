use crate::rope::Edit;
use std::collections::VecDeque;

/// An undo/redo stack.
#[derive(Default, Debug)]
pub struct History {
    // ..index -> undo stack
    // index.. -> redo stack
    index: usize,
    edits: VecDeque<Edit>,
}

impl History {
    pub const MAX: usize = 128;

    /// Pushes an `edit` onto the undo stack, emptying the redo stack.
    pub fn edit(&mut self, edit: Edit) {
        self.edits.truncate(self.index);
        self.edits.push_back(edit);
        self.index += 1;

        while Self::MAX < self.edits.len() {
            self.edits.pop_front();
            self.index -= 1;
        }
    }

    /// Moves backward in the history, returning the edit to unapply.
    pub fn undo(&mut self) -> Option<&Edit> {
        if self.index > 0 {
            self.index -= 1;
            self.edits.get(self.index)
        } else {
            None
        }
    }

    /// Moves foreward in the history, returning the edit to apply.
    pub fn redo(&mut self) -> Option<&Edit> {
        if self.index < self.edits.len() {
            self.index += 1;
            self.edits.get(self.index - 1)
        } else {
            None
        }
    }
}
