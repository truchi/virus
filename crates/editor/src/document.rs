use crate::{
    add_in_range,
    history::History,
    ids,
    rope::{
        Boundaries, Cursor, Edit, GraphemeCategory, GraphemesBackward, GraphemesForward,
        Segmentation, Selection, Text,
    },
    sub_in_range,
};
use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
use tree_sitter::{Parser, Query, Tree};

ids!(
    /// [`DocumentId`] generator.
    pub DocumentIds,
    /// [`Document`] id.
    pub DocumentId,
);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Document                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Document {
    id: DocumentId,
    path: PathBuf,
    rope: Rope,
    selection: Selection,
    anchor_segmentation: Segmentation,
    head_segmentation: Segmentation,
    highlights: Query,
    parser: Parser,
    tree: Tree,
    is_tree_dirty: bool,
    version: usize,
    history: History,
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
        let anchor_segmentation = Segmentation::new(rope.clone(), 0, 0, 0);
        let head_segmentation = anchor_segmentation.clone();
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
            anchor_segmentation,
            head_segmentation,
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

    pub fn movements(&mut self) -> DocumentMovements {
        debug_assert!(
            self.selection.anchor == self.anchor_segmentation.cursor()
                && self.selection.head == self.head_segmentation.cursor()
        );

        DocumentMovements { document: self }
    }

    pub fn edition(&mut self) -> DocumentEdition {
        DocumentEdition { document: self }
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

    fn leading_blank_lines(&self) -> usize {
        GraphemesForward::new(self.rope.slice(..))
            .take_while(|grapheme| matches!(grapheme.as_str().into(), GraphemeCategory::Break))
            .count()
    }

    fn trailing_blank_lines(&self) -> usize {
        GraphemesBackward::new(self.rope.slice(..))
            .take_while(|grapheme| matches!(grapheme.as_str().into(), GraphemeCategory::Break))
            .count()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       DocumentMovements                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentMovements<'document> {
    document: &'document mut Document,
}

impl<'document> DocumentMovements<'document> {
    pub fn collapse(&mut self, collapse: bool) -> &mut Self {
        if collapse {
            self.document.selection.collapse_mut();
            self.document.anchor_segmentation = self.document.head_segmentation.clone();
        }

        self
    }

    pub fn flip(&mut self, flip: bool) -> &mut Self {
        if flip {
            self.document.selection.flip_mut();
            std::mem::swap(
                &mut self.document.anchor_segmentation,
                &mut self.document.head_segmentation,
            );
        }

        self
    }

    pub fn anchor(&mut self, cursor: Cursor, update: bool) {
        self.document.selection.anchor = cursor;
        self.document.anchor_segmentation.to_cursor(cursor);

        if update {
            self.document
                .anchor_segmentation
                .update(self.document.rope.clone());
        }
    }

    pub fn head(&mut self, cursor: Cursor, update: bool) {
        self.document.selection.head = cursor;
        self.document.head_segmentation.to_cursor(cursor);

        if update {
            self.document
                .head_segmentation
                .update(self.document.rope.clone());
        }
    }

    pub fn selection(&mut self, selection: Selection, update: bool) {
        self.anchor(selection.anchor, update);
        self.head(selection.head, update);
    }

    pub fn top(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = Cursor::builder(self.document.rope.slice(..)).at_width(
            blank
                .then_some(0)
                .unwrap_or_else(|| self.document.leading_blank_lines()),
            self.document.selection.head.width,
        );

        self.head(self.document.selection.head, false);
        self
    }

    pub fn bottom(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = Cursor::builder(self.document.rope.slice(..)).at_width(
            self.document.rope.len_lines().saturating_sub(
                1 + blank
                    .then_some(0)
                    .unwrap_or_else(|| self.document.trailing_blank_lines()),
            ),
            self.document.selection.head.width,
        );

        self.head(self.document.selection.head, false);
        self
    }

    pub fn up(&mut self, lines: usize, wrap: bool) -> &mut Self {
        self.document.selection.head = Cursor::builder(self.document.rope.slice(..)).at_width(
            sub_in_range(
                self.document.rope.len_lines(),
                self.document.selection.head.line,
                lines,
                wrap,
            ),
            self.document.selection.head.width,
        );

        self.head(self.document.selection.head, false);
        self
    }

    pub fn down(&mut self, lines: usize, wrap: bool) -> &mut Self {
        self.document.selection.head = Cursor::builder(self.document.rope.slice(..)).at_width(
            add_in_range(
                self.document.rope.len_lines(),
                self.document.selection.head.line,
                lines,
                wrap,
            ),
            self.document.selection.head.width,
        );

        self.head(self.document.selection.head, false);
        self
    }

    pub fn start(&mut self, blank: bool) -> &mut Self {
        if blank {
            self.document.head_segmentation.to_line_start();
        } else {
            self.document.head_segmentation.to_line_first();
        }

        self.head(self.document.head_segmentation.cursor(), false);
        self
    }

    pub fn end(&mut self, blank: bool) -> &mut Self {
        if blank {
            self.document.head_segmentation.to_line_end();
        } else {
            self.document.head_segmentation.to_line_last();
        }

        self.head(self.document.head_segmentation.cursor(), false);
        self
    }

    pub fn left(&mut self, boundaries: Boundaries, repeat: usize, wrap: bool) -> &mut Self {
        for _ in 0..repeat {
            if !self.document.head_segmentation.prev(boundaries) && wrap {
                self.document.head_segmentation.to_end();
            }
        }

        self.head(self.document.head_segmentation.cursor(), false);
        self
    }

    pub fn right(&mut self, boundaries: Boundaries, repeat: usize, wrap: bool) -> &mut Self {
        for _ in 0..repeat {
            if !self.document.head_segmentation.next(boundaries) && wrap {
                self.document.head_segmentation.to_start();
            }
        }

        self.head(self.document.head_segmentation.cursor(), false);
        self
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        DocumentEdition                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentEdition<'document> {
    document: &'document mut Document,
}

impl<'document> DocumentEdition<'document> {
    pub fn edit(&mut self, inserted: Text) -> Option<Edit> {
        let edit = {
            let selection = self.document.selection();
            Edit::edit(&mut self.document.rope, selection.range(), inserted)
        };

        if edit.is_noop().unwrap_or_default() {
            return None;
        }

        self.document
            .movements()
            .selection(edit.inserted_end().into(), true);
        self.document.version += 1;
        self.document.is_tree_dirty = true;
        self.document.history.push(edit.clone());
        self.document.tree.edit(&edit.to_ts_edit_applied());

        Some(edit)
    }

    // TODO: convenient for now but does not feel good
    pub fn backspace(&mut self) -> Option<Edit> {
        if self.document.selection.is_empty() {
            self.document
                .movements()
                .left(Boundaries::GRAPHEME, 1, false);
        }

        self.edit(Text::default())
    }

    pub fn undo(&mut self) {
        let Some(edits) = self.document.history.undo() else {
            return;
        };
        let mut anchor = self.document.selection.anchor;
        let mut head = self.document.selection.head;

        for edit in edits {
            edit.unapply(&mut self.document.rope);
            self.document.tree.edit(&edit.to_ts_edit_applied());

            anchor = anchor
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
            head = head
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
        }

        self.document
            .movements()
            .selection(Selection::new(anchor, head), true);
        self.document.version += 1;
        self.document.is_tree_dirty = true;
    }

    pub fn redo(&mut self) {
        let Some(edits) = self.document.history.redo() else {
            return;
        };
        let mut anchor = self.document.selection.anchor;
        let mut head = self.document.selection.head;

        for edit in edits {
            edit.apply(&mut self.document.rope);
            self.document.tree.edit(&edit.to_ts_edit_applied());

            anchor = anchor
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
            head = head
                .edit(edit.start(), edit.removed_end(), edit.inserted_end())
                .unwrap_or(edit.start());
        }

        self.document
            .movements()
            .selection(Selection::new(anchor, head), true);
        self.document.version += 1;
        self.document.is_tree_dirty = true;
    }
}
