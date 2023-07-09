use tree_sitter::Node;

const SKIP: &[&str] = &["{", "}", "(", ")", "[", "]", "::", ":", ";", ",", "."];

fn check(node: &Node) -> bool {
    !SKIP.contains(&node.kind())
}

#[derive(Copy, Clone, Debug)]
pub struct Walk<'tree>(pub Node<'tree>);

impl<'tree> Walk<'tree> {
    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self)
    }

    pub fn parent_or_node(&self) -> Self {
        self.parent().unwrap_or(*self)
    }

    pub fn first_child(&self) -> Option<Self> {
        self.0
            .children(&mut self.0.walk())
            .find(check)
            .filter(|child| self.0.byte_range() != child.byte_range())
            .map(Self)
    }

    pub fn first_child_or_node(&self) -> Self {
        self.first_child().unwrap_or(*self)
    }

    pub fn last_child(&self) -> Option<Self> {
        (0..self.0.child_count())
            .rev()
            .filter_map(|i| self.0.child(i))
            .find(check)
            .filter(|child| self.0.byte_range() != child.byte_range())
            .map(Self)
    }

    pub fn last_child_or_node(&self) -> Self {
        self.last_child().unwrap_or(*self)
    }

    pub fn first_sibling(&self) -> Self {
        self.parent()
            .and_then(|parent| parent.first_child())
            .unwrap_or(*self)
    }

    pub fn prev_sibling(&self) -> Option<Self> {
        let mut node = self.0.prev_sibling()?;

        while !check(&node) {
            node = node.prev_sibling()?;
        }

        Some(Self(node))
    }

    pub fn next_sibling(&self) -> Option<Self> {
        let mut node = self.0.next_sibling()?;

        while !check(&node) {
            node = node.next_sibling()?;
        }

        Some(Self(node))
    }

    pub fn last_sibling(&self) -> Self {
        self.parent()
            .and_then(|parent| parent.last_child())
            .unwrap_or(*self)
    }

    pub fn prev_or_last_sibling(&self) -> Self {
        self.prev_sibling().unwrap_or_else(|| self.last_sibling())
    }

    pub fn next_or_first_sibling(&self) -> Self {
        self.next_sibling().unwrap_or_else(|| self.first_sibling())
    }
}
