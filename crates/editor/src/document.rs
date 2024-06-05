use crate::{cursor::Cursor, rope::RopeExt, syntax::Highlights};
use ropey::Rope;
use std::{fs::File, io::BufReader, ops::Range, usize};
use tree_sitter::{Parser, Query, Tree};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Default, Debug)]
pub struct Selection {
    anchor: Cursor,
    head: Cursor,
}

impl From<Cursor> for Selection {
    fn from(cursor: Cursor) -> Self {
        Self::cursor(cursor)
    }
}

impl Selection {
    pub fn new(anchor: Cursor, head: Cursor) -> Self {
        Self { anchor, head }
    }

    pub fn cursor(cursor: Cursor) -> Self {
        Self::new(cursor, cursor)
    }

    pub fn is_forward(&self) -> bool {
        if self.anchor <= self.head {
            true
        } else {
            false
        }
    }

    pub fn range(&self) -> Range<Cursor> {
        if self.is_forward() {
            self.anchor..self.head
        } else {
            self.head..self.anchor
        }
    }

    pub fn flip(&self) -> Self {
        Self::new(self.head, self.anchor)
    }

    pub fn flip_mut(&mut self) {
        *self = Self::new(self.head, self.anchor);
    }

    pub fn move_to(&self, cursor: Cursor, selection: bool) -> Self {
        if selection {
            Self::new(self.anchor, cursor)
        } else {
            Self::cursor(cursor)
        }
    }

    pub fn move_to_mut(&mut self, cursor: Cursor, selection: bool) {
        *self = self.move_to(cursor, selection);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Document                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Document {
    rope: Rope,
    selection: Selection, // TODO Should we really have this here?
    highlights: Query,
    parser: Parser,
    tree: Tree,
}

impl Document {
    // NOTE
    // This function is convenient for now.
    // We will need to deal with unsupported languages later.
    pub fn open(path: &str) -> std::io::Result<Self> {
        if !path.ends_with(".rs") {
            panic!("File type not supported");
        }

        const HIGHLIGHTS_QUERY: &str = include_str!("../treesitter/rust/highlights.scm");
        let language = tree_sitter_rust::language();

        let rope = Rope::from_reader(&mut BufReader::new(File::open(path)?))?;
        let highlights =
            Query::new(&language, HIGHLIGHTS_QUERY).expect("Cannot create highlights query");
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("Cannot set parser's language");
        let tree = Self::parse_with(&rope, &mut parser, None);

        Ok(Self {
            rope,
            selection: Default::default(),
            highlights,
            parser,
            tree,
        })
    }

    /// Reparses the AST.
    ///
    /// Call this function after your edits to the document to update the AST.
    pub fn parse(&mut self) {
        // TODO We could skip this if document is not "dirty"
        Self::parse_with(&self.rope, &mut self.parser, Some(&self.tree));
    }
}

/// Getters.
impl Document {
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn highlights(&self, lines: Range<usize>) -> Highlights {
        Highlights::new(&self.rope, self.tree.root_node(), lines, &self.highlights)
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }
}

/// Movements.
impl Document {
    pub fn move_anchor_to_head(&mut self) {
        self.selection = self.selection.head.into();
    }

    pub fn flip_anchor_and_head(&mut self) {
        self.selection.flip_mut();
    }

    pub fn move_up(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.grapheme().above(self.selection.head), selection);
    }

    pub fn move_down(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.grapheme().below(self.selection.head), selection);
    }

    pub fn move_prev_grapheme(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.grapheme().prev(self.selection.head), selection);
    }

    pub fn move_next_grapheme(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.grapheme().next(self.selection.head), selection);
    }

    pub fn move_prev_start_of_word(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.word().prev_start(self.selection.head), selection);
    }

    pub fn move_prev_end_of_word(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.word().prev_end(self.selection.head), selection);
    }

    pub fn move_next_start_of_word(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.word().next_start(self.selection.head), selection);
    }

    pub fn move_next_end_of_word(&mut self, selection: bool) {
        self.selection
            .move_to_mut(self.rope.word().next_end(self.selection.head), selection);
    }
}

/// Edition.
impl Document {
    pub fn edit(&mut self, str: &str) {
        // TODO edit cursors and tree

        let Range { start, end } = self.selection.range();
        self.rope.replace(start..end, str);

        let cursor = self.rope.cursor().index(start.index + str.len());
        self.edit_tree(start, end, cursor);
        self.selection = cursor.into();
    }

    pub fn backspace(&mut self) -> Result<(), ()> {
        let Range { start, end } = self.selection.range();

        if start != end {
            // TODO What to do here?
            return Err(());
        }

        let start = self.rope.grapheme().prev(end);

        if start != end {
            self.rope
                .remove(self.rope.byte_to_char(start.index)..self.rope.byte_to_char(end.index));

            self.edit_tree(start, end, start);
            self.selection = start.into();
        }

        Ok(())
    }
}

/// Private.
impl Document {
    fn parse_with(rope: &Rope, parser: &mut Parser, tree: Option<&Tree>) -> Tree {
        parser
            .parse_with(
                &mut |index, _| {
                    let (chunk, chunk_index, ..) = rope.chunk_at_byte(index);
                    &chunk[index - chunk_index..]
                },
                tree,
            )
            .expect("Cannot parse")
    }

    fn edit_tree(&mut self, start: Cursor, old_end: Cursor, new_end: Cursor) {
        self.tree
            .edit(&Cursor::into_input_edit(start, old_end, new_end));
    }
}
