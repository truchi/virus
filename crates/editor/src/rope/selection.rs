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
        Self::cursor(cursor)
    }
}

impl Selection {
    pub fn new(anchor: Cursor, head: Cursor) -> Self {
        Self { anchor, head }
    }

    pub fn cursor(cursor: Cursor) -> Self {
        Self::new(cursor, cursor)
    }

    pub fn len(&self) -> usize {
        let range = self.range();

        range.end.index() - range.start.index()
    }

    pub fn is_empty(&self) -> bool {
        self.anchor.index() == self.head.index()
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

    pub fn flip(&self) -> Self {
        Self::new(self.head, self.anchor)
    }

    pub fn flip_mut(&mut self) {
        *self = self.flip();
    }

    pub fn move_to(&self, cursor: Cursor, selection: bool) -> Self {
        if selection {
            Self::new(self.anchor, cursor)
        } else {
            Self::cursor(cursor)
        }
    }

    pub fn move_to_mut(&mut self, cursor: Cursor, selection: bool) {
        *self = self.move_to(cursor, selection);
    }
}
