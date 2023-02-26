use editor::{Document, Highlights, Theme};
use tree_sitter::Query;
use virus_common::{Rgba, Style};

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

    let theme = dracula();
    let highlights = Highlights::new(
        rope,
        tree.root_node(),
        0..117,
        Query::new(language.language().unwrap(), HIGHLIGHT_QUERY).unwrap(),
        theme,
    );

    let mut line = 0;

    for highlight in highlights.iter() {
        assert!(highlight.start.line == highlight.end.line);
        assert!(highlight.end.column != 0);

        if line != highlight.start.line {
            line = highlight.start.line;
            println!();
        }

        let cow = rope.byte_slice(highlight.start.index..highlight.end.index);

        let r = highlight.style.unwrap_or_default().foreground.r;
        let g = highlight.style.unwrap_or_default().foreground.g;
        let b = highlight.style.unwrap_or_default().foreground.b;
        print!("\x1B[38;2;{r};{g};{b}m{cow}\x1B[0m");
    }

    println!();
}

fn dracula() -> Theme {
    fn style(r: u8, g: u8, b: u8) -> Style {
        Style {
            foreground: Rgba {
                r,
                g,
                b,
                a: u8::MAX,
            },
            ..Default::default()
        }
    }

    let background = style(40, 42, 54);
    let current = style(68, 71, 90);
    let foreground = style(248, 248, 242);
    let comment = style(98, 114, 164);
    let cyan = style(139, 233, 253);
    let green = style(80, 250, 123);
    let orange = style(255, 184, 108);
    let pink = style(255, 121, 198);
    let purple = style(189, 147, 249);
    let red = style(255, 85, 85);
    let yellow = style(241, 250, 140);

    Theme {
        attribute: green,
        comment: current,
        constant: green,
        constant_builtin_boolean: purple,
        constant_character: purple,
        constant_character_escape: purple,
        constant_numeric_float: purple,
        constant_numeric_integer: purple,
        constructor: foreground,
        function: pink,
        function_macro: pink,
        function_method: pink,
        keyword: red,
        keyword_control: red,
        keyword_control_conditional: red,
        keyword_control_import: red,
        keyword_control_repeat: red,
        keyword_control_return: red,
        keyword_function: red,
        keyword_operator: red,
        keyword_special: red,
        keyword_storage: red,
        keyword_storage_modifier: red,
        keyword_storage_modifier_mut: red,
        keyword_storage_modifier_ref: red,
        keyword_storage_type: red,
        label: foreground,
        namespace: foreground,
        operator: foreground,
        punctuation_bracket: yellow,
        punctuation_delimiter: yellow,
        special: yellow,
        string: cyan,
        r#type: cyan,
        type_builtin: cyan,
        type_enum_variant: cyan,
        variable: orange,
        variable_builtin: orange,
        variable_other_member: orange,
        variable_parameter: orange,
    }
}
