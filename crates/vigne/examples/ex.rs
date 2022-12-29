use tree_sitter::{InputEdit, Node, Parser, Point, Query, QueryCursor, Tree, TreeCursor};
use tree_sitter_rust::{HIGHLIGHT_QUERY, TAGGING_QUERY};

fn main() {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser.set_language(language).unwrap();

    let source_code = include_str!("./ex.rs");
    let source_code = "fn test1() {}";
    let mut tree = parser.parse(source_code, None).unwrap();

    tree.edit(&InputEdit {
        start_byte: 3,
        old_end_byte: 8,
        new_end_byte: 8,
        start_position: Point { row: 1, column: 3 },
        old_end_position: Point { row: 1, column: 8 },
        new_end_position: Point { row: 1, column: 8 },
    });
    tree.edit(&InputEdit {
        start_byte: 4,
        old_end_byte: 5,
        new_end_byte: 5,
        start_position: Point { row: 1, column: 4 },
        old_end_position: Point { row: 1, column: 5 },
        new_end_position: Point { row: 1, column: 5 },
    });

    let source_code = "fn aaaaa() {}";
    let tree = parser.parse(source_code, Some(&tree)).unwrap();

    walk(&mut tree.walk(), 0, source_code);

    highlight(&tree, source_code);
}

fn _main2() {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser.set_language(language).unwrap();

    let source_code = include_str!("./ex.rs");
    let source_code = "fn test1(a: A, b: B) {}";
    let tree = parser.parse(source_code, None).unwrap();

    // let root_node = tree.root_node();
    // assert_eq!(root_node.kind(), "source_file");
    // assert_eq!(root_node.start_position().column, 0);
    // assert_eq!(root_node.end_position().column, 12);

    let mut cursor = tree.walk();

    // walk(&mut cursor, 0);

    // println!("{}", tree_sitter_rust::GRAMMAR);
    // println!("{}", tree_sitter_rust::NODE_TYPES);
    // println!("{}", HIGHLIGHT_QUERY);
    // println!("{}", TAGGING_QUERY);

    let query = Query::new(language, HIGHLIGHT_QUERY).unwrap();
    let mut cursor = QueryCursor::new();

    {
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
        for match_ in matches {
            dbg!(match_);
        }
    }

    {
        let captures = cursor
            .captures(&query, tree.root_node(), source_code.as_bytes())
            .map(|capture| capture.0.captures)
            .flatten();

        for capture in captures {
            let range = capture.node.byte_range();
            let text = &source_code[range];
            println!(
                "{} {}",
                text,
                &query.capture_names()[capture.index as usize]
            );
        }
    }

    // for (i, name) in query.capture_names().iter().enumerate() {
    //     dbg!((i, name));
    // }
}

fn walk(cursor: &mut TreeCursor, depth: usize, source: &str) {
    let node = cursor.node();
    let kind = node.kind();
    let id = node.kind_id();
    let prefix = " ".repeat(depth * 4);

    println!("{prefix}{id} {kind}: {}", &source[node.byte_range()]);

    if !cursor.goto_first_child() {
        return;
    }

    loop {
        walk(cursor, depth + 1, source);
        if !cursor.goto_next_sibling() {
            cursor.goto_parent();
            return;
        }
    }
}

fn highlight(tree: &Tree, source: &str) {
    let query = Query::new(tree_sitter_rust::language(), HIGHLIGHT_QUERY).unwrap();
    let mut cursor = QueryCursor::new();

    let captures = cursor
        .captures(&query, tree.root_node(), source.as_bytes())
        .map(|capture| capture.0.captures)
        .flatten();

    for capture in captures {
        let range = capture.node.byte_range();
        let text = &source[range];
        println!(
            "{} {}",
            text,
            &query.capture_names()[capture.index as usize]
        );
    }
}
