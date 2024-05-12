use graphics::text::Font;
use swash::FontDataRef;

const NOTO_EMOJI: &str = "/Users/romain/Library/Fonts/NotoColorEmoji.ttf";
const APPLE_EMOJI: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";

fn main() {
    inspect("Noto Emoji", NOTO_EMOJI);
    inspect("Apple Emoji", APPLE_EMOJI);
}

fn inspect(name: &str, path: &str) {
    println!("================================================");
    println!("{name}");
    println!("================================================");

    let font = Font::from_file(path).unwrap();
    font.inspect();
    println!();

    let data = FontDataRef::new(font.as_ref().data).unwrap();
    dbg!(data.len());
    dbg!(data.is_collection());
    dbg!(data.get(1).is_some());
    dbg!(data.fonts().len());
    println!();

    dbg!(font.as_ref().metrics(&[]));
    println!();
}
