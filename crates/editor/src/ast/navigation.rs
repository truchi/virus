use ropey::RopeSlice;
use std::ops::{ControlFlow, Range};
use tree_sitter::{Node, Tree, TreeCursor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Pairs                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

bitflags::bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Pairs: u16 {
        const PAREN  = 1 << 1;
        const SQUARE = 1 << 2;
        const CURLY  = 1 << 3;
        const ANGLE  = 1 << 4;
        const PIPE   = 1 << 5;
        const SINGLE = 1 << 6;
        const DOUBLE = 1 << 7;
        const BACK   = 1 << 8;
    }
}

impl Pairs {
    pub fn from_bools(
        paren: bool,
        square: bool,
        curly: bool,
        angle: bool,
        pipe: bool,
        single: bool,
        double: bool,
        back: bool,
    ) -> Self {
        let mut pairs = Self::empty();

        paren.then(|| pairs.insert(Self::PAREN));
        square.then(|| pairs.insert(Self::SQUARE));
        curly.then(|| pairs.insert(Self::CURLY));
        angle.then(|| pairs.insert(Self::ANGLE));
        pipe.then(|| pairs.insert(Self::PIPE));
        single.then(|| pairs.insert(Self::SINGLE));
        double.then(|| pairs.insert(Self::DOUBLE));
        back.then(|| pairs.insert(Self::BACK));

        pairs
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Navigation                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Navigation<'rope, 'tree> {
    rope: RopeSlice<'rope>,
    tree: &'tree Tree,
    node: Node<'tree>,
    start: usize,
    end: usize,
}

impl<'rope, 'tree> Navigation<'rope, 'tree> {
    pub fn new(rope: RopeSlice<'rope>, tree: &'tree Tree, range: Range<usize>) -> Self {
        Self {
            rope,
            tree,
            node: Self::find_node(tree, range.clone()).unwrap_or_else(|| tree.root_node()),
            start: range.start,
            end: range.end,
        }
    }

    pub fn parent(&self, pairs: Pairs, mut count: usize) -> Option<Node<'tree>> {
        let mut parent = None;
        let mut node = self.node;

        while count > 0 {
            if self.is_pairs(node, pairs) && (self.start..self.end) != node.byte_range() {
                parent = Some(node);
                count -= 1;
            }

            node = if let Some(node) = node.parent() {
                node
            } else {
                break;
            };
        }

        parent
    }

    pub fn child(&self, pairs: Pairs, count: usize, wrap: bool) -> Option<Node<'tree>> {
        self.prev_or_next(
            self.node,
            self.node.child(0)?,
            Node::next_sibling,
            &TreeCursor::goto_first_child,
            &TreeCursor::goto_next_sibling,
            pairs,
            count,
            wrap,
        )
    }

    pub fn prev(&self, pairs: Pairs, count: usize, wrap: bool) -> Option<Node<'tree>> {
        self.prev_or_next(
            self.parent(pairs, 1)
                .unwrap_or_else(|| self.tree.root_node()),
            self.node,
            Node::prev_sibling,
            &TreeCursor::goto_last_child,
            &TreeCursor::goto_previous_sibling,
            pairs,
            count,
            wrap,
        )
    }

    pub fn next(&self, pairs: Pairs, count: usize, wrap: bool) -> Option<Node<'tree>> {
        self.prev_or_next(
            self.parent(pairs, 1)
                .unwrap_or_else(|| self.tree.root_node()),
            self.node,
            Node::next_sibling,
            &TreeCursor::goto_first_child,
            &TreeCursor::goto_next_sibling,
            pairs,
            count,
            wrap,
        )
    }
}

/// Private.
impl<'rope, 'tree> Navigation<'rope, 'tree> {
    fn is_pairs(&self, node: Node, pairs: Pairs) -> bool {
        if node.id() == self.tree.root_node().id() {
            return true;
        }

        let start = self.rope.byte_to_char(node.start_byte());
        let end = self.rope.byte_to_char(node.end_byte().saturating_sub(1));
        let start = self.rope.char(start);
        let end = self.rope.char(end);

        pairs.iter().any(|pairs| match () {
            _ if pairs == Pairs::PAREN => start == '(' && end == ')',
            _ if pairs == Pairs::SQUARE => start == '[' && end == ']',
            _ if pairs == Pairs::CURLY => start == '{' && end == '}',
            _ if pairs == Pairs::ANGLE => start == '<' && end == '>',
            _ if pairs == Pairs::PIPE => start == '|' && end == '|',
            _ if pairs == Pairs::SINGLE => start == '\'' && end == '\'',
            _ if pairs == Pairs::DOUBLE => start == '"' && end == '"',
            _ if pairs == Pairs::BACK => start == '`' && end == '`',
            _ => {
                debug_assert!(false);
                false
            }
        })
    }

    fn find_node(tree: &'tree Tree, range: Range<usize>) -> Option<Node<'tree>> {
        // NOTE: there is `Node::descendant_for_byte_range()` but I think it's buggy.

        let mut node = None;
        let mut cursor = tree.root_node().walk();

        loop {
            if !cursor.goto_first_child() {
                return node;
            }

            loop {
                if cursor.node().start_byte() <= range.start
                    && range.end <= cursor.node().end_byte()
                {
                    node = Some(cursor.node());
                    break;
                }

                if !cursor.goto_next_sibling() {
                    return node;
                }
            }
        }
    }

    fn prev_or_next<F, G>(
        &self,
        root: Node<'tree>,
        mut node: Node<'tree>,
        prev_or_next_sibling: impl Fn(&Node<'tree>) -> Option<Node<'tree>>,
        goto_first_or_last_child: &F,
        goto_prev_or_next_child: &G,
        pairs: Pairs,
        mut count: usize,
        mut wrap: bool,
    ) -> Option<Node<'tree>>
    where
        F: Fn(&mut TreeCursor<'tree>) -> bool,
        G: Fn(&mut TreeCursor<'tree>) -> bool,
    {
        let mut prev_or_next = None;

        while count > 0 {
            if let Some(sibling) = prev_or_next_sibling(&node) {
                node = sibling;

                if let ControlFlow::Break(node) = traverse_depth_first(
                    node,
                    goto_first_or_last_child,
                    goto_prev_or_next_child,
                    &mut |node| {
                        self.is_pairs(node, pairs)
                            .then_some(ControlFlow::Break(node))
                            .unwrap_or(ControlFlow::Continue(()))
                    },
                ) {
                    prev_or_next = Some(node);
                    count -= 1;
                }
            } else {
                node = if let Some(node) = node.parent().filter(|parent| parent.id() != root.id()) {
                    node
                } else if wrap {
                    let mut cursor = root.walk();

                    if goto_first_or_last_child(&mut cursor) {
                        wrap = false;
                        cursor.node()
                    } else {
                        break;
                    }
                } else {
                    break;
                };
            }
        }

        prev_or_next
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn traverse_depth_first<'tree, F, G, B>(
    node: Node<'tree>,
    goto_first_or_last_child: &F,
    goto_prev_or_next_child: &G,
    callback: &mut impl FnMut(Node<'tree>) -> ControlFlow<B>,
) -> ControlFlow<B>
where
    F: Fn(&mut TreeCursor<'tree>) -> bool,
    G: Fn(&mut TreeCursor<'tree>) -> bool,
{
    callback(node)?;

    let mut cursor = node.walk();

    if goto_first_or_last_child(&mut cursor) {
        loop {
            traverse_depth_first(
                cursor.node(),
                goto_first_or_last_child,
                goto_prev_or_next_child,
                callback,
            )?;

            if !goto_prev_or_next_child(&mut cursor) {
                break;
            }
        }
    }

    ControlFlow::Continue(())
}
