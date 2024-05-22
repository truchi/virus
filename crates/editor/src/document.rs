use crate::{highlights::Highlights, rope::RopeExt};
use ropey::Rope;
use std::{fs::File, io::BufReader, ops::Range, usize};
use tree_sitter::{Parser, Query, Tree};
use virus_common::Cursor;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Document                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Document {
    rope: Rope,
    selection: Range<Cursor>,
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

    pub fn selection(&self) -> Range<Cursor> {
        self.selection.clone()
    }

    pub fn selection_mut(&mut self) -> &mut Range<Cursor> {
        &mut self.selection
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
    pub fn move_up(&mut self) {
        // TODO start != end?
        let cursor = self.rope.grapheme().above(self.selection.start);
        self.selection = cursor..cursor;
    }

    pub fn move_down(&mut self) {
        // TODO start != end?
        let cursor = self.rope.grapheme().below(self.selection.start);
        self.selection = cursor..cursor;
    }

    pub fn move_left(&mut self) {
        // TODO start != end?
        let cursor = self.rope.grapheme().before(self.selection.start);
        self.selection = cursor..cursor;
    }

    pub fn move_right(&mut self) {
        // TODO start != end?
        let cursor = self.rope.grapheme().after(self.selection.start);
        self.selection = cursor..cursor;
    }

    pub fn move_prev_word(&mut self) {
        // TODO start != end?
        let cursor = self.rope.word().before(self.selection.start);
        self.selection = cursor..cursor;
    }

    pub fn move_next_start_of_word(&mut self) {
        // TODO start != end?
        let cursor = self.rope.word().next_start(self.selection.start);
        self.selection = cursor..cursor;
    }

    pub fn move_next_end_of_word(&mut self) {
        // TODO start != end?
        let cursor = self.rope.word().next_end(self.selection.start);
        self.selection = cursor..cursor;
    }
}

/// Edition.
impl Document {
    pub fn edit(&mut self, str: &str) {
        // TODO edit cursors and tree

        let Range { start, end } = self.selection;
        self.rope.replace(start..end, str);

        let cursor = self.rope.cursor().index(start.index + str.len());
        self.edit_tree(start, end, cursor);
        self.selection = cursor..cursor;
    }

    pub fn backspace(&mut self) -> Result<(), ()> {
        let Range { start, end } = self.selection;

        if start != end {
            // TODO What to do here?
            return Err(());
        }

        let start = self.rope.grapheme().before(end);

        if start != end {
            self.rope
                .remove(self.rope.byte_to_char(start.index)..self.rope.byte_to_char(end.index));

            self.edit_tree(start, end, start);
            self.selection = start..start;
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
