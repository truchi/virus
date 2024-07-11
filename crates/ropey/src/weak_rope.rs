use crate::{tree::Node, Rope};
use std::sync::{Arc, Weak};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            WeakRope                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Default)]
pub struct WeakRope {
    pub(crate) root: Weak<Node>,
}

impl WeakRope {
    pub const fn new() -> Self {
        Self { root: Weak::new() }
    }

    pub fn upgrade(&self) -> Option<Rope> {
        self.root.upgrade().map(|root| Rope { root })
    }

    pub fn is_instance(&self, other: &WeakRope) -> bool {
        Weak::ptr_eq(&self.root, &other.root)
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl Rope {
    pub fn downgrade(this: &Rope) -> WeakRope {
        WeakRope {
            root: Arc::downgrade(&this.root),
        }
    }
}
