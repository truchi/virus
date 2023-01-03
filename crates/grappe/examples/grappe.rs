use grappe::{
    line::Line,
    text::{Text, TextCursor, TextPrevChars},
};
use std::sync::Arc;

fn main() {
    let str = "😍\n🦀\n";
    let text = Text::from(str);
    let cursor = TextCursor::from_end(&text);
    let map = |o: Option<(TextCursor, char)>| {
        o.map(|(cursor, char)| (cursor.offset(), (cursor.row(), cursor.column()), char))
    };
    let cursors = |prev_chars: &TextPrevChars| {
        (
            (
                prev_chars.front_offset(),
                (prev_chars.front_row(), prev_chars.front_column()),
            ),
            (
                prev_chars.back_offset(),
                (prev_chars.back_row(), prev_chars.back_column()),
            ),
        )
    };

    println!("{str:?}");

    println!("-------------------------");

    let mut prev_chars = cursor.prev_chars();
    // println!("{:?}", cursors(&prev_chars));

    while let Some((cursor, char)) = prev_chars.next_back() {
        let indexes = (cursor.offset(), (cursor.row(), cursor.column()));
        println!("{:?}", (indexes, char));
        // println!("{:?}", cursors(&prev_chars));
    }

    println!("-------------------------");

    let mut prev_chars = cursor.prev_chars();
    // println!("{:?}", cursors(&prev_chars));

    while let Some((cursor, char)) = prev_chars.next() {
        let indexes = (cursor.offset(), (cursor.row(), cursor.column()));
        println!("{:?}", (indexes, char));
        // println!("{:?}", cursors(&prev_chars));
    }

    // println!("{:?}", cursors(&prev_chars));
    // let (cursor, char) = prev_chars.next().unwrap();
    // dbg!((cursor, char));
    // println!("{:?}", cursors(&prev_chars));
    // let (cursor, char) = prev_chars.next().unwrap();
    // dbg!((cursor, char));
    // println!("{:?}", cursors(&prev_chars));
    // let (cursor, char) = prev_chars.next().unwrap();
    // dbg!((cursor, char));
    // println!("{:?}", cursors(&prev_chars));
    // let (cursor, char) = prev_chars.next().unwrap();
    // dbg!((cursor, char));
    // println!("{:?}", cursors(&prev_chars));

    // for (cursor, char) in prev_chars {
    //     dbg!((cursor, char));
    // }

    // assert!(cursors(prev_chars) == ((9, (0, 0)), (9, (0, 0))));

    // assert!(map(prev_chars.next()) == Some((8, (0, 0), '\n')));
    // assert!(cursors(prev_chars) == ((9, (0, 0)), (9, (0, 0))));

    // assert!(map(prev_chars.next_back()) == Some((0, '😍')));
    // assert!(cursors(prev_chars) == ((9, (0, 0)), (9, (0, 0))));

    // assert!(map(prev_chars.next()) == Some((4, '🦀')));
    // assert!(cursors(prev_chars) == ((9, (0, 0)), (9, (0, 0))));

    // assert!(map(prev_chars.next_back()) == None);
    // assert!(map(prev_chars.next()) == None);
}
