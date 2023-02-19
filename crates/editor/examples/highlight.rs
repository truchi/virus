use editor::{Document, Highlights};
use tree_sitter::Query;

const HIGHLIGHT_QUERY: &str = include_str!("../treesitter/rust/highlights.scm");

fn main() {
    let mut document =
        Document::open("/home/romain/perso/virus/crates/virus/src/main.rs".into()).unwrap();
    dbg!(document.language());
    document.parse();

    let rope = document.rope();
    let language = document.language().unwrap();
    let _parser = document.parser().unwrap();
    let tree = document.tree().unwrap();

    let theme = Default::default();
    let highlights = Highlights::new(
        tree.root_node(),
        &rope,
        0..10000000,
        Query::new(language.language().unwrap(), HIGHLIGHT_QUERY).unwrap(),
        &theme,
    );

    for (_selection, cow, style) in highlights.iter() {
        if style.is_none() {
            print!("\x1B[0;31m{cow}\x1B[0m");
        } else {
            print!("\x1B[0;30m{cow}\x1B[0m");
        }
    }
}
