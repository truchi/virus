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
        // let front = prev_chars.front_index();
        // let back = prev_chars.back_index();
        let front = prev_chars.front().index();
        let back = prev_chars.back().index();

        (
            (front.offset(), (front.row(), front.column())),
            (back.offset(), (back.row(), back.column())),
        )
    };

    println!("{str:?}");

    println!("-------------------------next");

    let mut prev_chars = cursor.prev_chars();
    println!("{:?}", cursors(&prev_chars));

    while let Some((cursor, char)) = prev_chars.next() {
        let indexes = (cursor.offset(), (cursor.row(), cursor.column()));
        println!("{:?}", (indexes, char));
        println!("{:?}", cursors(&prev_chars));
    }

    println!("-------------------------next_back");

    let mut prev_chars = cursor.prev_chars();
    println!("{:?}", cursors(&prev_chars));

    while let Some((cursor, char)) = prev_chars.next_back() {
        let indexes = (cursor.offset(), (cursor.row(), cursor.column()));
        println!("{:?}", (indexes, char));
        println!("{:?}", cursors(&prev_chars));
    }

    println!("-------------------------both");

    let mut prev_chars = cursor.prev_chars();
    // println!("{:?}", cursors(&prev_chars));

    let next = |prev_chars: &mut TextPrevChars| prev_chars.next().map(|(_, char)| char);
    let back = |prev_chars: &mut TextPrevChars| prev_chars.next_back().map(|(_, char)| char);

    dbg!(back(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));
    dbg!(next(&mut prev_chars));

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
