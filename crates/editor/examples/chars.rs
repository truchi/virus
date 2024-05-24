use ropey::{iter::Chunks, Rope};

const N: usize = 984;

fn main() {
    let string = "a".repeat(N) + &"b".repeat(N) + &"c".repeat(N);
    let rope = Rope::from(string.as_str());
    dbg!(rope.len_bytes());

    let index = rope.len_bytes();
    let index = N + 1;

    fn chars_from(rope: &Rope, index: usize) -> impl '_ + Iterator<Item = (usize, char)> {
        rope.chars_at(rope.byte_to_char(index))
            .scan(index, |index, char| {
                let i = *index;
                *index += char.len_utf8();

                Some((i, char))
            })
    }

    fn chars_to(rope: &Rope, index: usize) -> impl '_ + Iterator<Item = (usize, char)> {
        rope.chars_at(rope.byte_to_char(index))
            .reversed()
            .scan(index, |index, char| {
                *index -= char.len_utf8();

                Some((*index, char))
            })
    }
}

fn main2() {
    let string = "a".repeat(N) + &"b".repeat(N) + &"c".repeat(N);
    let rope = Rope::from(string);
    dbg!(rope.len_bytes());

    let index = rope.len_bytes() - 1;
    let (chunks, first_chunk_index, ..) = rope.chunks_at_byte(index);
    dbg!(index, first_chunk_index);

    let mut it = chunks
        .reversed()
        .into_iter()
        .scan(first_chunk_index, |prev_chunk_index, chunk| {
            *prev_chunk_index -= chunk.len();
            Some((*prev_chunk_index, chunk))
        })
        .inspect(|(i, chunk)| {
            dbg!(("1", i, chunk, chunk.len()));
        })
        // .enumerate()
        // .map(move |(i, (chunk_index, chunk))| {
        //     if i == 0 {
        //         &chunk[..index - chunk_index]
        //     } else {
        //         chunk
        //     }
        // })
        // .inspect(|chunk| {
        //     dbg!("2", chunk, chunk.len());
        // })
        .flat_map(|(chunk_index, chunk)| {
            chunk
                .char_indices()
                .rev()
                .map(move |(i, char)| (chunk_index + i, char))
        });

    for a in it.enumerate() {
        // dbg!(a);
    }

    dbg!(1 * N);
    dbg!(2 * N);
    dbg!(3 * N);
}

fn main3() {
    let string = "a".repeat(N) + &"b".repeat(N) + &"c".repeat(N);
    let rope = Rope::from(string);

    let (chunks, ..) = rope.chunks_at_byte(0);
    for (i, chunk) in chunks.enumerate() {
        if i == 0 {
            assert!(chunk.chars().all(|char| char == 'a'));
        } else if i == 1 {
            assert!(chunk.chars().all(|char| char == 'b'));
        } else if i == 2 {
            assert!(chunk.chars().all(|char| char == 'c'));
        }
        assert!(chunk.len() == N);
    }

    let (mut chunks, ..) = rope.chunks_at_byte(0 * N);
    dbg!(chunks.prev());
    let (mut chunks, ..) = rope.chunks_at_byte(1 * N);
    dbg!(chunks.prev());
    let (mut chunks, ..) = rope.chunks_at_byte(2 * N);
    dbg!(chunks.prev());
    let (mut chunks, ..) = rope.chunks_at_byte(3 * N);
    dbg!(chunks.prev());

    // for i in 0..=3 {
    //     println!("-------------------------------------");
    //     dbg!(i);

    //     let (mut chunks, i, ..) = rope.chunks_at_byte(i * N);
    //     dbg!(chunks.prev());
    //     dbg!(chunks.prev());
    //     dbg!(chunks.prev());
    //     dbg!(chunks.next());
    //     dbg!(chunks.next());
    //     dbg!(chunks.next());
    // }
}

pub struct CharCursor<'rope> {
    index: usize,
    chunks: Chunks<'rope>,
    chunk_offset: usize,
}

impl<'rope> CharCursor<'rope> {
    pub fn new(rope: &'rope Rope, index: usize) -> Self {
        let (chunks, chunk_offset, ..) = rope.chunks_at_byte(index);

        Self {
            index,
            chunks,
            chunk_offset,
        }
    }

    pub fn next(&mut self) -> (usize, char) {
        let str = "";
        todo!()
    }
}
