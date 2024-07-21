use crate::rope::Edit;
use std::collections::VecDeque;

#[derive(Debug)]
enum Item {
    Single([Edit; 1]),
    Multiple(Vec<Edit>),
}

/// An undo/redo stack.
#[derive(Default, Debug)]
pub struct History {
    // ..index -> undo stack
    // index.. -> redo stack
    index: usize,
    items: VecDeque<Item>,
}

impl History {
    pub const MAX: usize = 128;

    /// Pushes an `edit` onto the undo stack, emptying the redo stack.
    pub fn push(&mut self, edit: Edit) {
        if !edit.is_noop().unwrap_or_default() {
            self.push_impl(Item::Single([edit]));
        }
    }

    /// Pushes a bulk of `edits` onto the undo stack, emptying the redo stack.
    pub fn push_bulk(&mut self, edits: impl IntoIterator<Item = Edit>) {
        let edits = edits
            .into_iter()
            .filter(|edit| !edit.is_noop().unwrap_or_default())
            .collect::<Vec<_>>();

        if !edits.is_empty() {
            self.push_impl(Item::Multiple(edits));
        }
    }

    /// Moves backward in the history, returning the edits to unapply.
    pub fn undo(&mut self) -> Option<&[Edit]> {
        if self.index > 0 {
            self.index -= 1;
            self.items.get(self.index).map(|item| match item {
                Item::Single(edit) => edit,
                Item::Multiple(edits) => edits.as_slice(),
            })
        } else {
            None
        }
    }

    /// Moves foreward in the history, returning the edits to apply.
    pub fn redo(&mut self) -> Option<&[Edit]> {
        if self.index < self.items.len() {
            self.index += 1;
            self.items.get(self.index - 1).map(|item| match item {
                Item::Single(edit) => edit,
                Item::Multiple(edits) => edits.as_slice(),
            })
        } else {
            None
        }
    }
}

/// Private.
impl History {
    fn push_impl(&mut self, item: Item) {
        self.items.truncate(self.index);
        self.items.push_back(item);
        self.index += 1;

        while Self::MAX < self.items.len() {
            self.items.pop_front();
            self.index -= 1;
        }
    }
}
