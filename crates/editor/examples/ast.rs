#![allow(unused)]

use std::ops::Range;

use editor::document::Document;
use tree_sitter::{Node, Point, Tree, TreeCursor};

fn main() {
    let mut document =
        Document::open("/home/romain/perso/rust/virus/crates/virus/src/main.rs".into()).unwrap();
    let tree = document.parse().unwrap();
    dbg!(tree);
    dbg!(tree.root_node().to_sexp());

    let mut walker = tree.walk();
    let i = walker
        .goto_first_child_for_point(Point { row: 4, column: 4 })
        .unwrap();
    dbg!(i);
    let i = walker
        .goto_first_child_for_point(Point { row: 4, column: 4 })
        .unwrap();
    dbg!(i);
    let node = walker.node();
    dbg!(node);
    dbg!(node.range());
    dbg!(node.id());

    let node = find(
        tree.root_node(),
        Point { row: 4, column: 4 }..Point { row: 4, column: 4 },
    )
    .unwrap();
    dbg!(node.id());
}

fn find(node: Node, range: Range<Point>) -> Option<Node> {
    fn contains(node: Node, range: &Range<Point>) -> bool {
        node.start_position() <= range.start && range.end <= node.end_position()
    }

    pub fn children<'a, 'tree>(
        walker: &'a mut TreeCursor<'tree>,
    ) -> Option<impl Iterator<Item = Node<'tree>> + 'a> {
        if walker.node().child_count() > 0 {
            let mut first = true;
            Some(std::iter::from_fn(move || {
                if first {
                    first = false;
                    walker.goto_first_child();
                    Some(walker.node())
                } else if walker.goto_next_sibling() {
                    Some(walker.node())
                } else {
                    None
                }
            }))
        } else {
            None
        }
    }

    if !contains(node, &range) {
        return None;
    }

    let mut walker = node.walk();

    loop {
        let node = walker.node();

        // Advance `walker` to the first `node` child containing `range`
        if children(&mut walker)
            .and_then(|children| children.filter(|child| contains(*child, &range)).next())
            .is_none()
        {
            return Some(node);
        }
    }
}
