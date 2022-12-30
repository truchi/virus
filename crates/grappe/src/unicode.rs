use unicode_segmentation::GraphemeCursor;

pub struct ChunksGraphemeCursor<'a> {
    chunks: [&'a str; 2],
    index: usize,
    cursor: GraphemeCursor,
}

impl<'a> ChunksGraphemeCursor<'a> {
    pub fn new(chunks: [&'a str; 2], offset: usize) -> Self {
        let len = chunks[0].len() + chunks[1].len();
        debug_assert!(offset <= len);

        Self {
            chunks,
            index: if offset < chunks[0].len() { 0 } else { 1 },
            cursor: GraphemeCursor::new(offset, len, true),
        }
    }

    pub fn set_cursor(&mut self, offset: usize) {
        let len = self.chunks[0].len() + self.chunks[1].len();
        debug_assert!(offset <= len);

        self.index = if offset < self.chunks[0].len() { 0 } else { 1 };
        self.cursor.set_cursor(offset);
    }

    pub fn cur_cursor(&self) -> usize {
        self.cursor.cur_cursor()
    }

    pub fn is_boundary(&mut self) -> bool {
        // Eurk
        match self.cursor.is_boundary(
            self.chunks[self.index],
            if self.index == 0 {
                0
            } else {
                self.chunks[0].len()
            },
        ) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}
