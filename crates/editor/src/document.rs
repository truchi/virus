use crate::{highlights::Highlights, rope::RopeExt, walk::Walk};
use ropey::Rope;
use std::{fs::File, io::BufReader, ops::Range, usize};
use tree_sitter::{Node, Parser, Query, Tree};
use virus_common::Cursor;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Selection                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub enum Selection {
    Range { start: Cursor, end: Cursor },
    Ast { start: Cursor, end: Cursor },
}

impl Default for Selection {
    fn default() -> Self {
        Self::Range {
            start: Cursor::default(),
            end: Cursor::default(),
        }
    }
}

impl Selection {
    pub fn range(Range { start, end }: Range<Cursor>) -> Self {
        Self::Range { start, end }
    }

    pub fn ast(Range { start, end }: Range<Cursor>) -> Self {
        Self::Ast { start, end }
    }

    pub fn as_range(&self) -> Option<Range<Cursor>> {
        match *self {
            Self::Range { start, end } => Some(start..end),
            _ => None,
        }
    }

    pub fn as_ast(&self) -> Option<Range<Cursor>> {
        match *self {
            Self::Ast { start, end } => Some(start..end),
            _ => None,
        }
    }

    pub fn to_range(&self) -> Range<Cursor> {
        match *self {
            Self::Range { start, end } => start..end,
            Self::Ast { start, end } => start..end,
        }
    }

    pub fn move_up(&self, document: &Document) -> Option<Self> {
        match *self {
            Self::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.grapheme_above(start);
                Some(Self::range(cursor..cursor))
            }
            Self::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).prev_or_last_sibling().0))
                .map(Selection::ast),
        }
    }

    pub fn move_down(&self, document: &Document) -> Option<Self> {
        match *self {
            Self::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.grapheme_below(start);
                Some(Self::range(cursor..cursor))
            }
            Self::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).next_or_first_sibling().0))
                .map(Self::ast),
        }
    }

    pub fn move_left(&self, document: &Document) -> Option<Self> {
        match *self {
            Self::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.prev_grapheme(start);
                Some(Self::range(cursor..cursor))
            }
            Self::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).parent_or_node().0))
                .map(Self::ast),
        }
    }

    pub fn move_right(&self, document: &Document) -> Option<Self> {
        match *self {
            Self::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.next_grapheme(start);
                Some(Self::range(cursor..cursor))
            }
            Self::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).first_child_or_node().0))
                .map(Self::ast),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Document                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Document {
    rope: Rope,
    selection: Selection,
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
        Self::parse_with(&self.rope, &mut self.parser, Some(&self.tree));
    }
}

/// Getters.
impl Document {
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
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
        if let Some(selection) = self.selection.move_up(self) {
            self.selection = selection;
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selection) = self.selection.move_down(self) {
            self.selection = selection;
        }
    }

    pub fn move_left(&mut self) {
        if let Some(selection) = self.selection.move_left(self) {
            self.selection = selection;
        }
    }

    pub fn move_right(&mut self) {
        if let Some(selection) = self.selection.move_right(self) {
            self.selection = selection;
        }
    }
}

/// Edition.
impl Document {
    pub fn edit_str(&mut self, str: &str) {
        // TODO edit cursors and tree

        let Range { start, end } = self.selection.to_range();
        let start = start.index;
        let end = end.index;
        let start_char = self.rope.byte_to_char(start);

        if start != end {
            let end_char = self.rope.byte_to_char(end);
            self.rope.remove(start_char..end_char);
        }

        if !str.is_empty() {
            self.rope.insert(start_char, str);
        }
    }

    pub fn edit_char(&mut self, char: char) {
        let Range { start, end } = self.selection.to_range();

        self.rope.edit_char(start..end, char);

        let cursor = self.rope.cursor_at_index(start.index + char.len_utf8());
        self.edit_tree(start, end, cursor);
        self.selection = Selection::range(cursor..cursor);
    }

    pub fn backspace(&mut self) -> Result<(), ()> {
        let Range { start, end } = self.selection.to_range();

        if start != end {
            return Err(());
        }

        let start = self.rope.prev_grapheme(end);

        if start != end {
            self.rope
                .remove(self.rope.byte_to_char(start.index)..self.rope.byte_to_char(end.index));

            self.edit_tree(start, end, start);
            self.selection = Selection::range(start..start);
        }

        Ok(())
    }
}

/// Private.
impl Document {
    fn parse_with(rope: &Rope, parser: &mut Parser, tree: Option<&Tree>) -> Tree {
        // TODO We could skip this if document is not "dirty"

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

    fn find_node(&self, range: Range<Cursor>) -> Option<Node> {
        let mut node = self
            .tree
            .root_node()
            .descendant_for_point_range(range.start.into(), range.end.into())?;

        // Make sure parent's range != node's range,
        // or we are trapped in this node!
        while let Some(parent) = node
            .parent()
            .filter(|parent| parent.byte_range() == node.byte_range())
        {
            node = parent;
        }

        Some(node)
    }
}
