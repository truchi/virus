use tree_sitter::{InputEdit, Node, Parser, Point, Query, QueryCursor, Tree, TreeCursor};
use tree_sitter_rust::{HIGHLIGHT_QUERY, TAGGING_QUERY};

fn ed(start: usize, old: usize, new: usize) -> InputEdit {
    InputEdit {
        start_byte: start,
        old_end_byte: old,
        new_end_byte: new,
        start_position: Point {
            row: 0,
            column: start,
        },
        old_end_position: Point {
            row: 0,
            column: old,
        },
        new_end_position: Point {
            row: 0,
            column: new,
        },
    }
}

fn highlight(tree: &Tree, source: &str) {
    let query = Query::new(tree_sitter_rust::language(), HIGHLIGHT_QUERY).unwrap();
    // let query = Query::new(tree_sitter_rust::language(), TAGGING_QUERY).unwrap();
    dbg!(query.capture_names());
    let mut cursor = QueryCursor::new();

    let captures = cursor.matches(&query, tree.root_node(), source.as_bytes());

    // for (query_match, capture_index) in captures {
    for query_match in captures {
        let pattern_index = query_match.pattern_index;
        let capture_index = "NOP";

        println!(
            "pattern_index: {pattern_index} ({} captures) (capture_index: {capture_index})",
            query_match.captures.len()
        );

        for capture in query_match.captures {
            let range = capture.node.byte_range();
            let text = &source[range];
            let capture_index = capture.index as usize;
            let capture_name = &query.capture_names()[capture_index];

            println!("- index {capture_index}, name {capture_name}, text: {text:?}");
            // println!("- capture_index: {capture_index}");
            // println!("- capture name: {capture_name}");
            // println!("- text: {:?}", text);
        }
    }
}

fn main() {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser.set_language(language).unwrap();

    let source =
        r#"const STR: &str = "0123456789"; fn main() {}const LOL: &str = "abcdef"; fn lol() {}"#;

    println!("{source}");
    println!();

    let tree = parser.parse(source, None).unwrap();
    // walk(&mut tree.walk(), 0, source);
    // debug(&mut tree.walk(), 0, source);

    // println!("{:?}", tree.root_node().to_sexp());
    // println!();

    highlight(&tree, source);
}

fn _main3() {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser.set_language(language).unwrap();

    let old = r#"const STR: &str = "0123456789";const STR: &str = "0123456789";"#;
    let new = r#"const STR01234567890123456789: &str = "";const STR: &str = "0123456789";"#;
    let const_edit = ed(6, 9, 29);
    let str_edit_orig = ed(19, 29, 19);
    let str_edit_inter = ed(39, 49, 39);

    println!("{old}");
    println!("{new}");
    println!();

    for edit in [str_edit_orig, str_edit_inter] {
        let mut tree = parser.parse(old, None).unwrap();

        tree.edit(&const_edit);
        tree.edit(&edit);

        let tree = parser.parse(new, Some(&tree)).unwrap();

        debug(&mut tree.walk(), 0, new);
        walk(&mut tree.walk(), 0, new);

        println!("{:?}", tree.root_node().to_sexp());
    }

    // highlight(&tree, source_code);
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

fn debug(cursor: &mut TreeCursor, depth: usize, source: &str) {
    let node = cursor.node();

    println!("{}", &source[node.byte_range()]);
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
