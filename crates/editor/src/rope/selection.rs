use crate::rope::Cursor;
use std::ops::Range;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Selection {
    pub anchor: Cursor,
    pub head: Cursor,
}

impl From<Cursor> for Selection {
    fn from(cursor: Cursor) -> Self {
        Self::new(cursor, cursor)
    }
}

impl Selection {
    pub fn new(anchor: Cursor, head: Cursor) -> Self {
        Self { anchor, head }
    }

    pub fn len(&self) -> usize {
        let range = self.range();

        range.end.index - range.start.index
    }

    pub fn is_empty(&self) -> bool {
        self.anchor.index == self.head.index
    }

    pub fn is_forward(&self) -> bool {
        if self.anchor <= self.head {
            true
        } else {
            false
        }
    }

    pub fn range(&self) -> Range<Cursor> {
        if self.is_forward() {
            self.anchor..self.head
        } else {
            self.head..self.anchor
        }
    }

    pub fn collapse(&self) -> Self {
        Self::new(self.head, self.head)
    }

    pub fn collapse_mut(&mut self) {
        *self = self.collapse();
    }

    pub fn flip(&self) -> Self {
        Self::new(self.head, self.anchor)
    }

    pub fn flip_mut(&mut self) {
        *self = self.flip();
    }
}

#[cfg(test)]
impl Selection {
    /// Extracts one or two `┃`s as selection in the rope.
    pub fn extract(str: &str) -> (ropey::Rope, Self) {
        let (rope, cursors) = Cursor::extract_all(str);
        let anchor = cursors.get(0).copied().unwrap();
        let head = cursors.get(1).copied().unwrap_or(anchor);

        (rope, Self::new(anchor, head))
    }
}
