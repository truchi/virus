use grappe::{Line, Text};
use std::sync::Arc;

fn main() {
    let str = include_str!("../Cargo.toml");
    let text = Text::from(str);

    assert_eq!(text.len(), str.len());

    println!("{text}");

    let mut line = Line::from("Hello");

    let other = line.clone();
    dbg!(Arc::strong_count(&line.string));
    let string = Arc::make_mut(&mut line.string);
    string.push_str(" world!");

    println!("{line}");
    println!("{other}");
    dbg!(Arc::strong_count(&line.string));
}
