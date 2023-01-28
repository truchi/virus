use crate::{info::Info, Node};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Text {
    node: Arc<Node>,
    info: Info,
}

impl Text {
    fn debug(&self) {
        fn debug(node: Node, info: Info) {}

        match self.node.as_ref() {
            Node::Internal(internal) => todo!(),
            Node::Leaf(leaf) => todo!(),
        }
    }
}
