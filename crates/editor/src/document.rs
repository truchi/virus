use crate::{language::Language, rope::RopeExt};
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

    pub fn to_range(&self) -> Range<Cursor> {
        match *self {
            Self::Range { start, end } => start..end,
            Self::Ast { start, end } => start..end,
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
        match self.selection {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = self.rope.grapheme_above(start);
                self.selection = Selection::range(cursor..cursor);
            }
            Selection::Ast { start, end } => {
                if let Some(node) = self.find_node(start..end) {
                    let node = prev_or_last_sibling(node);
                    dbg!(node.kind());
                    self.selection = Selection::ast(Cursor::from_node(node));
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.selection {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = self.rope.grapheme_below(start);
                self.selection = Selection::range(cursor..cursor);
            }
            Selection::Ast { start, end } => {
                if let Some(node) = self.find_node(start..end) {
                    let node = next_or_first_sibling(node);
                    dbg!(node.kind());
                    self.selection = Selection::ast(Cursor::from_node(node));
                }
            }
        }
    }

    pub fn move_prev(&mut self) {
        match self.selection {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = self.rope.prev_grapheme(start);
                self.selection = Selection::range(cursor..cursor);
            }
            Selection::Ast { start, end } => {
                if let Some(node) = self.find_node(start..end) {
                    let node = node.parent().unwrap_or(node);
                    dbg!(node.kind());
                    self.selection = Selection::ast(Cursor::from_node(node));
                }
            }
        }
    }

    pub fn move_next(&mut self) {
        match self.selection {
            Selection::Range { start, .. } => {
                // TODO start != end?
                let cursor = self.rope.next_grapheme(start);
                self.selection = Selection::range(cursor..cursor);
            }
            Selection::Ast { start, end } => {
                if let Some(node) = self.find_node(start..end) {
                    let node = node.child(0).unwrap_or(node);
                    dbg!(node.kind());
                    self.selection = Selection::ast(Cursor::from_node(node));
                }
            }
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
        self.tree.as_ref().and_then(|tree| {
            tree.root_node()
                .descendant_for_point_range(range.start.into(), range.end.into())
        })
    }
}

fn prev_or_last_sibling(node: Node) -> Node {
    node.prev_sibling().unwrap_or_else(|| {
        node.parent()
            .and_then(|parent| parent.child(parent.child_count() - 1))
            .unwrap_or(node)
    })
}

fn next_or_first_sibling(node: Node) -> Node {
    node.next_sibling().unwrap_or_else(|| {
        node.parent()
            .and_then(|parent| parent.child(0))
            .unwrap_or(node)
    })
}
