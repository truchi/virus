use crate::{info::Info, Node};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Child {
    node: Arc<Node>,
    info: Info,
}

impl Child {
    pub fn new(node: Arc<Node>, info: Info) -> Self {
        Self { node, info }
    }

    pub fn info(&self) -> Info {
        self.info
    }
}
