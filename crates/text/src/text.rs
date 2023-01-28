use crate::{info::Info, Leaf, Node};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Text {
    pub node: Arc<Node>,
    pub info: Info,
}

impl Text {
    pub fn new(node: Arc<Node>, info: Info) -> Self {
        Self { node, info }
    }

    pub fn info(&self) -> Info {
        self.info
    }

    pub fn node(&self) -> &Arc<Node> {
        &self.node
    }
    pub fn leaves<T: FnMut(Info, &Leaf)>(&self, f: T) {
        self.node.leaves(f, self.info);
    }
}
