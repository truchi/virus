use std::ops::ControlFlow;
use tree_sitter::{Language, Node, Parser};

const INDENT: usize = 4;

fn main() {
    let path = std::env::args().nth(1).expect("path as first argument");
    let file = std::fs::read_to_string(&path).expect("readable file");
    let tree = {
        let mut parser = Parser::new();
        parser
            .set_language(&Language::from(tree_sitter_rust::LANGUAGE))
            .expect("Cannot set parser's language");
        parser
            .parse_with(&mut |index, _| &file[index..], None)
            .expect("Cannot parse")
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{path}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{file}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("{}", tree.root_node().to_sexp());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    traverse_depth_first(tree.root_node(), &mut |node, depth| {
        let depth = " ".repeat(depth * INDENT);
        let (start, end) = (
            (node.start_position().row, node.start_position().column),
            (node.end_position().row, node.end_position().column),
        );
        let kind = node.kind();
        let text = file[node.start_byte()..node.end_byte()]
            .lines()
            .map(|line| line.trim())
            .collect::<String>();

        println!("{depth}* {kind} {start:?}..{end:?} {text}");

        ControlFlow::<()>::Continue(())
    });

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn traverse_depth_first<'tree, B>(
    node: Node<'tree>,
    callback: &mut impl FnMut(Node<'tree>, usize) -> ControlFlow<B>,
) -> ControlFlow<B> {
    fn traverse_depth_first<'tree, B>(
        node: Node<'tree>,
        callback: &mut impl FnMut(Node<'tree>, usize) -> ControlFlow<B>,
        depth: usize,
    ) -> ControlFlow<B> {
        let mut cursor = node.walk();

        callback(node, depth)?;

        if cursor.goto_first_child() {
            let depth = depth + 1;

            loop {
                traverse_depth_first(cursor.node(), callback, depth)?;

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        ControlFlow::Continue(())
    }

    traverse_depth_first(node, callback, 0)
}
