use crate::{info::Info, Leaf, Node};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Text {
    pub node: Arc<Node>,
    pub info: Info,
}

impl Text {
    pub fn leaves<T: FnMut(Info, &Leaf)>(&self, f: T) {
        self.node.leaves(f, self.info);
    }
}
