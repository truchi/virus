use crate::{language::Language, rope::RopeExt, walk::Walk};
use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
};
use tree_sitter::{Node, Parser, Query, Tree};
use virus_common::Cursor;

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
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.grapheme_above(start);
                Some(Self::range(cursor..cursor))
            }
            Selection::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).prev_or_last_sibling().0))
                .map(Selection::ast),
        }
    }

    pub fn move_down(&mut self, document: &Document) -> Option<Self> {
        match *self {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.grapheme_below(start);
                Some(Selection::range(cursor..cursor))
            }
            Selection::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).next_or_first_sibling().0))
                .map(Self::ast),
        }
    }

    pub fn move_left(&mut self, document: &Document) -> Option<Self> {
        match *self {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.prev_grapheme(start);
                Some(Selection::range(cursor..cursor))
            }
            Selection::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).parent_or_node().0))
                .map(Self::ast),
        }
    }

    pub fn move_right(&mut self, document: &Document) -> Option<Self> {
        match *self {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = document.rope.next_grapheme(start);
                Some(Selection::range(cursor..cursor))
            }
            Selection::Ast { start, end } => document
                .find_node(start..end)
                .map(|node| Cursor::from_node(Walk(node).first_child_or_node().0))
                .map(Self::ast),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Document                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Default)]
pub struct Document {
    path: Option<String>,
    rope: Rope,
    selection: Selection,
    language: Option<Language>,
    parser: Option<Parser>,
    tree: Option<Tree>,
    dirty: bool,
}

impl Document {
    pub fn empty(language: Option<Language>) -> Self {
        Self {
            path: None,
            rope: Default::default(),
            selection: Default::default(),
            language,
            parser: language.map(|language| language.parser()).flatten(),
            tree: None,
            dirty: false,
        }
    }

    pub fn open(path: String) -> std::io::Result<Self> {
        let rope = Rope::from_reader(&mut BufReader::new(File::open(&path)?))?;
        let (language, parser) = if let Ok(language) = Language::try_from(path.as_str()) {
            (Some(language), language.parser())
        } else {
            (None, None)
        };

        Ok(Self {
            path: Some(path),
            rope,
            selection: Default::default(),
            language,
            parser,
            tree: None,
            dirty: false,
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = self.path.as_ref() {
            if self.dirty {
                let mut writer =
                    BufWriter::new(OpenOptions::new().write(true).truncate(true).open(path)?);

                for chunk in self.rope.chunks() {
                    writer.write(chunk.as_bytes())?;
                }
            }

            self.dirty = false;
        }

        Ok(())
    }

    pub fn parse(&mut self) -> Option<&Tree> {
        // TODO is_tree_dirty(/is_file_dirty)
        if let Some(parser) = self.parser.as_mut() {
            self.tree = parser.parse_with(
                &mut |index, _| {
                    let (chunk, chunk_index, ..) = self.rope.chunk_at_byte(index);
                    &chunk[index - chunk_index..]
                },
                self.tree.as_ref(),
            );
        }

        self.tree.as_ref()
    }
}

/// Getters.
impl Document {
    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|path| path.as_str())
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    pub fn language(&self) -> Option<Language> {
        self.language
    }

    pub fn parser(&self) -> Option<&Parser> {
        self.parser.as_ref()
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn query(&self, query: &str) -> Option<Query> {
        self.language
            .map(|language| language.language())
            .flatten()
            .map(|language| Query::new(language, query).ok())
            .flatten()
    }
}

/// Movements.
impl Document {
    pub fn move_up(&mut self) {
        if let Some(selection) = self.selection().move_up(self) {
            self.selection = selection;
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selection) = self.selection().move_down(self) {
            self.selection = selection;
        }
    }

    pub fn move_left(&mut self) {
        if let Some(selection) = self.selection().move_left(self) {
            self.selection = selection;
        }
    }

    pub fn move_right(&mut self) {
        if let Some(selection) = self.selection().move_right(self) {
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
            self.dirty = true;
        }

        if !str.is_empty() {
            self.rope.insert(start_char, str);
            self.dirty = true;
        }
    }

    pub fn edit_char(&mut self, char: char) {
        let Range { start, end } = self.selection.to_range();

        self.rope.edit_char(start..end, char);
        self.dirty = true;

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
            self.dirty = true;

            self.edit_tree(start, end, start);
            self.selection = Selection::range(start..start);
        }

        Ok(())
    }
}

/// Private.
impl Document {
    fn edit_tree(&mut self, start: Cursor, old_end: Cursor, new_end: Cursor) {
        if let Some(tree) = &mut self.tree {
            tree.edit(&Cursor::into_input_edit(start, old_end, new_end));
        }
    }

    fn find_node(&self, range: Range<Cursor>) -> Option<Node> {
        let mut node = self
            .tree
            .as_ref()?
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
