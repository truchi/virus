use crate::{
    add_in_range,
    ast::RUST_HIGHLIGHTS,
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
    time::SystemTime,
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
    on_disk_content: Rope,
    on_disk_modified: SystemTime,
    on_disk_is_dirty: bool,
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

    pub fn lines_selection(&self) -> Selection {
        let mut range_selection = self.selection;

        if !self.selection.is_forward() {
            range_selection.flip_mut();
        }

        let anchor = range_selection.anchor;
        let head = range_selection.head;
        let is_last_line = head.line == self.rope.slice(..).len_lines() - 1;

        let mut lines_selection = Selection {
            anchor: Cursor::build(self.rope.slice(..)).at_column(anchor.line, 0),
            head: if is_last_line {
                Cursor::build(self.rope.slice(..)).at_end()
            } else {
                Cursor::build(self.rope.slice(..)).at_column(head.line + 1, 0)
            },
        };

        if !self.selection.is_forward() {
            lines_selection.flip_mut();
        }

        lines_selection
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

    pub fn on_disk_content(&self) -> &Rope {
        &self.on_disk_content
    }

    pub fn on_disk_modified(&self) -> SystemTime {
        self.on_disk_modified
    }

    pub fn on_disk_is_dirty(&self) -> bool {
        self.on_disk_is_dirty
    }
}

impl Document {
    pub fn leading_blank_lines(&self) -> usize {
        GraphemesForward::new(self.rope.slice(..))
            .take_while(|grapheme| matches!(grapheme.as_str().into(), GraphemeCategory::Break))
            .count()
    }

    pub fn trailing_blank_lines(&self) -> usize {
        GraphemesBackward::new(self.rope.slice(..))
            .take_while(|grapheme| matches!(grapheme.as_str().into(), GraphemeCategory::Break))
            .count()
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

        let language = tree_sitter_rust::language();
        let file = File::open(&path)?;
        let rope = Rope::from_reader(&mut BufReader::new(&file))?;
        let anchor_segmentation = Segmentation::new(rope.clone(), 0, 0, 0);
        let head_segmentation = anchor_segmentation.clone();
        let highlights =
            Query::new(&language, RUST_HIGHLIGHTS).expect("Cannot create highlights query");
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("Cannot set parser's language");
        let tree = Self::parse_with(&rope, &mut parser, None);

        let document = Self {
            id,
            path,
            rope: rope.clone(),
            selection: Selection::default(),
            anchor_segmentation,
            head_segmentation,
            highlights,
            parser,
            tree,
            is_tree_dirty: false,
            version: 0,
            history: History::default(),
            on_disk_modified: file
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or_else(|_| SystemTime::now()),
            on_disk_is_dirty: false,
            on_disk_content: rope,
        };

        Ok(document)
    }

    // NOTE: good enough for now
    // TODO: atomic write, links, async?, ...
    // https://github.com/helix-editor/helix/blob/e14c346ee74a44051b2c07c2255a6ab80142dbe7/helix-view/src/document.rs#L858
    pub fn save(&mut self) -> std::io::Result<()> {
        if !self.on_disk_is_dirty {
            return Ok(());
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(&file);

        for chunk in self.rope.chunks() {
            writer.write(chunk.as_bytes())?;
        }

        writer.flush()?;
        file.sync_all()?;

        self.on_disk_modified = file
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or_else(|_| SystemTime::now());
        self.on_disk_is_dirty = false;

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

    pub fn anchor(&mut self, cursor: Cursor, update: bool) -> &mut Self {
        self.document.selection.anchor = cursor;

        if update {
            self.document
                .anchor_segmentation
                .update(self.document.rope.clone(), cursor);
        } else {
            self.document.anchor_segmentation.to_cursor(cursor);
        }

        self
    }

    pub fn head(&mut self, cursor: Cursor, update: bool) -> &mut Self {
        self.document.selection.head = cursor;

        if update {
            self.document
                .head_segmentation
                .update(self.document.rope.clone(), cursor);
        } else {
            self.document.head_segmentation.to_cursor(cursor);
        }

        self
    }

    pub fn selection(&mut self, selection: Selection, update: bool) -> &mut Self {
        self.anchor(selection.anchor, update);
        self.head(selection.head, update);
        self
    }

    pub fn top(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = Cursor::build(self.document.rope.slice(..)).at_width(
            blank
                .then_some(0)
                .unwrap_or_else(|| self.document.leading_blank_lines()),
            self.document.selection.head.width,
        );

        self.head(self.document.selection.head, false);
        self
    }

    pub fn bottom(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = Cursor::build(self.document.rope.slice(..)).at_width(
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
        self.document.selection.head = Cursor::build(self.document.rope.slice(..)).at_width(
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
        self.document.selection.head = Cursor::build(self.document.rope.slice(..)).at_width(
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

    pub fn prev(&mut self, boundaries: Boundaries, repeat: usize, wrap: bool) -> &mut Self {
        for _ in 0..repeat {
            if !self.document.head_segmentation.prev(boundaries) {
                if wrap {
                    self.document.head_segmentation.to_text_end();
                } else {
                    break;
                }
            }
        }

        self.head(self.document.head_segmentation.cursor(), false);
        self
    }

    pub fn next(&mut self, boundaries: Boundaries, repeat: usize, wrap: bool) -> &mut Self {
        for _ in 0..repeat {
            if !self.document.head_segmentation.next(boundaries) {
                if wrap {
                    self.document.head_segmentation.to_text_start();
                } else {
                    break;
                }
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
    pub fn edit(&mut self, inserted: Text, reselect: bool) -> Option<Edit> {
        let selection = self.document.selection();
        let edit = Edit::edit(&mut self.document.rope, selection.range(), inserted);

        if edit.is_noop().unwrap_or_default() {
            return None;
        }

        self.document
            .movements()
            .selection(
                Selection {
                    anchor: edit.start(),
                    head: edit.inserted_end(),
                },
                true,
            )
            .flip(reselect && !selection.is_forward())
            .collapse(!reselect);

        self.document.version += 1;
        self.document.is_tree_dirty = true;
        self.document.on_disk_is_dirty = true;
        self.document.history.push(edit.clone());
        self.document.tree.edit(&edit.to_ts_edit_applied());

        Some(edit)
    }

    // TODO: convenient for now but does not feel good
    pub fn backspace(&mut self) -> Option<Edit> {
        if self.document.selection.is_empty() {
            self.document
                .movements()
                .prev(Boundaries::GRAPHEME, 1, false);
        }

        self.edit(Text::default(), false)
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
        self.document.on_disk_is_dirty = true;
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
        self.document.on_disk_is_dirty = true;
    }
}
