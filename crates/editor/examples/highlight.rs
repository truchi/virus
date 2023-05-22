use editor::{document::Document, highlights::Highlights, theme::Theme};
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

    let theme = Theme::dracula();
    let _lines = rope.len_lines();

    let highlights = Highlights::new(
        rope,
        tree.root_node(),
        0..117,
        &Query::new(language.language().unwrap(), HIGHLIGHT_QUERY).unwrap(),
    );

    let mut line = 0;

    for highlight in highlights.highlights() {
        assert!(highlight.start.index != highlight.end.index);
        assert!(highlight.start.line == highlight.end.line);
        assert!(highlight.end.column != 0);

        if line != highlight.start.line {
            line = highlight.start.line;
            println!();
        }
        let cow = rope.byte_slice(highlight.start.index..highlight.end.index);
        let style = theme[highlight.key];
        let r = style.foreground.r;
        let g = style.foreground.g;
        let b = style.foreground.b;
        print!("\x1B[38;2;{r};{g};{b}m{cow}\x1B[0m");
    }

    println!();
}
