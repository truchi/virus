use text::{buffer::BufferRef, builder::Builder};

fn main() {
    let mut rope = Builder::new();

    for i in 0..100 {
        rope.push_str(&format!("{i} Hello"));
        rope.push_char(',');
        rope.push_str("world");
        rope.push_char('!');
        rope.push_str("\n");
    }

    let text = rope.build();

    // dbg!(&text);

    let mut i = 0;
    text.leaves(|info, leaf| {
        dbg!(i, info);
        i += 1;

        let buff = unsafe { BufferRef::from_buffer(&leaf.buffer, info.bytes, info.feeds) };
        dbg!(buff);
    });
}
