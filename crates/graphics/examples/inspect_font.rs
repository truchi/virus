use graphics::text::Font;
use swash::FontDataRef;

// const RECURSIVE_VF: &str =
//     "/home/romain/.local/share/fonts/Recursive/Recursive_Desktop/Recursive_VF_1.084.ttf";
// const RECURSIVE: &str =
//     "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
// const FIRA: &str =
//     "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
const EMOJI: &str = "/Users/romain/Library/Fonts/NotoColorEmoji.ttf";

fn main() {
    // println!("Recursive VF");
    // Font::from_file(RECURSIVE_VF).unwrap();
    // vf.inspect();
    // println!();

    // println!("Recursive");
    // Font::from_file(RECURSIVE).unwrap().inspect();
    // println!();

    // println!("Fira");
    // Font::from_file(FIRA).unwrap().inspect();
    // println!();

    println!("Emoji");
    let emoji = Font::from_file(EMOJI).unwrap();
    emoji.inspect();
    println!();

    let data_ref = FontDataRef::new(emoji.as_ref().data).unwrap();

    dbg!(data_ref.len());
    dbg!(data_ref.is_collection());
    dbg!(data_ref.get(1).is_some());
    dbg!(data_ref.fonts().len());
}
