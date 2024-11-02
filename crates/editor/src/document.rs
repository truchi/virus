use crate::{
    history::History,
    rope::{CursorRef, Edit, Selection, Text},
};
use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
use tree_sitter::{Parser, Query, Tree};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           DocumentId                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct DocumentId(usize);

impl DocumentId {
    pub fn generate(&mut self) -> Self {
        let id = *self;
        self.0 += 1;
        id
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Document                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Document {
    id: DocumentId,
    path: PathBuf,
    rope: Rope,
    selection: Selection, // TODO Should we really have this here?
    highlights: Query,
    parser: Parser,
    tree: Tree,
    is_tree_dirty: bool,
    version: usize,
    history: History,
}

impl Document {
    // NOTE:
    // This function is convenient for now.
    // We will need to deal with unsupported languages later.
    pub fn open(id: DocumentId, path: PathBuf) -> std::io::Result<Self> {
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            panic!("File type not supported");
        }

        const HIGHLIGHTS_QUERY: &str = include_str!("../treesitter/rust/highlights.scm");
        let language = tree_sitter_rust::language();

        let rope = Rope::from_reader(&mut BufReader::new(File::open(&path)?))?;
        let highlights =
            Query::new(&language, HIGHLIGHTS_QUERY).expect("Cannot create highlights query");
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("Cannot set parser's language");
        let tree = Self::parse_with(&rope, &mut parser, None);

        let document = Self {
            id,
            path,
            rope,
            selection: Selection::default(),
            highlights,
            parser,
            tree,
            is_tree_dirty: false,
            version: 0,
            history: History::default(),
        };

        Ok(document)
    }

    // NOTE: good enough for now
    pub fn save(&mut self) -> std::io::Result<()> {
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.path)?,
        );

        for chunk in self.rope.chunks() {
            writer.write(chunk.as_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Reparses the AST.
    ///
    /// Call this function after your edits to the document to update the AST.
    pub fn parse(&mut self) {
        if self.is_tree_dirty {
            self.tree = Self::parse_with(&self.rope, &mut self.parser, Some(&self.tree));
            self.is_tree_dirty = false;
        }
    }
}

/// Getters.
impl Document {
    pub fn id(&self) -> DocumentId {
        self.id
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn highlights(&self) -> &Query {
        &self.highlights
    }

    pub fn version(&self) -> usize {
        self.version
    }
}

/// Movements.
impl Document {
    pub fn move_anchor_to_head(&mut self) {
        self.selection = self.selection.head.clone().into();
    }

    pub fn flip_anchor_and_head(&mut self) {
        self.selection.flip_mut();
    }

    pub fn move_up(&mut self, selection: bool, lines: usize) {
        let line = self.selection.head.line();
        let width = self.selection.head.width();
        let cursor = CursorRef::with(self.rope.slice(..));

        self.selection.move_to_mut(
            line.checked_sub(lines)
                .map(|line| cursor.at_line_width(line, width))
                .unwrap_or_else(|| cursor.at_start())
                .as_cursor(),
            selection,
        );
    }

    pub fn move_down(&mut self, selection: bool, lines: usize) {
        let line = self.selection.head.line();
        let width = self.selection.head.width();

        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_line_width(
                    self.rope.len_lines().saturating_sub(1).min(line + lines),
                    width,
                )
                .as_cursor(),
            selection,
        );
    }

    pub fn move_prev_grapheme(&mut self, selection: bool) {
        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_cursor(self.selection.head)
                .prev_grapheme()
                .map(|cursor| cursor.as_cursor())
                .unwrap_or(self.selection.head),
            selection,
        );
    }

    pub fn move_next_grapheme(&mut self, selection: bool) {
        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_cursor(self.selection.head)
                .next_grapheme()
                .map(|cursor| cursor.as_cursor())
                .unwrap_or(self.selection.head),
            selection,
        );
    }

    pub fn move_prev_start_of_word(&mut self, selection: bool) {
        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_cursor(self.selection.head)
                .prev_word_start()
                .map(|cursor| cursor.as_cursor())
                .unwrap_or(self.selection.head),
            selection,
        );
    }

    pub fn move_prev_end_of_word(&mut self, selection: bool) {
        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_cursor(self.selection.head)
                .prev_word_end()
                .map(|cursor| cursor.as_cursor())
                .unwrap_or(self.selection.head),
            selection,
        );
    }

    pub fn move_next_start_of_word(&mut self, selection: bool) {
        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_cursor(self.selection.head)
                .next_word_start()
                .map(|cursor| cursor.as_cursor())
                .unwrap_or(self.selection.head),
            selection,
        );
    }

    pub fn move_next_end_of_word(&mut self, selection: bool) {
        self.selection.move_to_mut(
            CursorRef::with(self.rope.slice(..))
                .at_cursor(self.selection.head)
                .next_word_end()
                .map(|cursor| cursor.as_cursor())
                .unwrap_or(self.selection.head),
            selection,
        );
    }
}

/// Edition.
impl Document {
    pub fn edit(&mut self, inserted: Text) {
        let edit = {
            let selection = self.selection();
            Edit::edit(&mut self.rope, selection.range(), inserted)
        };

        if edit.is_noop().unwrap_or_default() {
            return;
        }

        let ts_edit = edit.to_ts_edit_applied();

        self.selection = edit.inserted_end().into();
        self.version += 1;
        self.is_tree_dirty = true;
        self.history.push(edit);
        self.tree.edit(&ts_edit);
    }

    // TODO: convenient for now but does not feel good
    pub fn backspace(&mut self) {
        if self.selection.is_empty() {
            self.move_prev_grapheme(true);
        }

        self.edit(Text::default());
    }

    pub fn undo(&mut self) {
        let Some(edits) = self.history.undo() else {
            return;
        };
        let mut anchor = self.selection.anchor;
        let mut head = self.selection.head;

        for edit in edits {
            edit.unapply(&mut self.rope);
            self.tree.edit(&edit.to_ts_edit_applied());

            anchor = anchor
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
            head = head
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
        }

        self.selection = Selection::new(anchor, head);
        self.version += 1;
        self.is_tree_dirty = true;
    }

    pub fn redo(&mut self) {
        let Some(edits) = self.history.redo() else {
            return;
        };
        let mut anchor = self.selection.anchor;
        let mut head = self.selection.head;

        for edit in edits {
            edit.apply(&mut self.rope);
            self.tree.edit(&edit.to_ts_edit_applied());

            anchor = anchor
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
            head = head
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
        }

        self.selection = Selection::new(anchor, head);
        self.version += 1;
        self.is_tree_dirty = true;
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
}
