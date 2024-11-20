use crate::{
    add_in_range,
    history::History,
    ids,
    rope::{CursorRef, Edit, Selection, Text},
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
    selection: Selection, // TODO Should we really have this here?
    highlights: Query,
    parser: Parser,
    tree: Tree,
    is_tree_dirty: bool,
    version: usize,
    history: History,
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

    pub fn movements(&mut self) -> DocumentMovements {
        DocumentMovements { document: self }
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
        collapse.then(|| self.document.selection.collapse_mut());
        self
    }

    pub fn flip(&mut self, flip: bool) -> &mut Self {
        flip.then(|| self.document.selection.flip_mut());
        self
    }

    pub fn top(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = CursorRef::with(self.document.rope.slice(..))
            .at_line_width(
                if blank {
                    0
                } else {
                    self.document
                        .rope
                        .lines()
                        .take_while(|line| line.len_bytes() == 0)
                        .count()
                },
                self.document.selection.head.width,
            )
            .as_cursor();
        self
    }

    pub fn bottom(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = CursorRef::with(self.document.rope.slice(..))
            .at_line_width(
                self.document.rope.len_lines().saturating_sub(
                    1 + if blank {
                        0
                    } else {
                        self.document
                            .rope
                            .lines_at(self.document.rope.len_lines())
                            .reversed()
                            .take_while(|line| line.len_bytes() == 0)
                            .count()
                    },
                ),
                self.document.selection.head.width,
            )
            .as_cursor();
        self
    }

    pub fn up(&mut self, lines: usize, wrap: bool) -> &mut Self {
        self.document.selection.head = CursorRef::with(self.document.rope.slice(..))
            .at_line_width(
                sub_in_range(
                    self.document.rope.len_lines(),
                    self.document.selection.head.line,
                    lines,
                    wrap,
                ),
                self.document.selection.head.width,
            )
            .as_cursor();

        self
    }

    pub fn down(&mut self, lines: usize, wrap: bool) -> &mut Self {
        self.document.selection.head = CursorRef::with(self.document.rope.slice(..))
            .at_line_width(
                add_in_range(
                    self.document.rope.len_lines(),
                    self.document.selection.head.line,
                    lines,
                    wrap,
                ),
                self.document.selection.head.width,
            )
            .as_cursor();

        self
    }

    pub fn start(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = CursorRef::with(self.document.rope.slice(..))
            .at_line_column(
                self.document.selection.head.line,
                if blank {
                    0
                } else {
                    self.document
                        .rope
                        .line(self.document.selection.head.line)
                        .chars()
                        .take_while(|char| char.is_whitespace())
                        .map(|char| char.len_utf8())
                        .sum()
                },
            )
            .as_cursor();

        self
    }

    pub fn end(&mut self, blank: bool) -> &mut Self {
        self.document.selection.head = CursorRef::with(self.document.rope.slice(..))
            .at_line_column(
                self.document.selection.head.line,
                self.document
                    .rope
                    .line(self.document.selection.head.line)
                    .len_bytes()
                    .saturating_sub(
                        1 + if blank {
                            0
                        } else {
                            self.document
                                .rope
                                .line(self.document.selection.head.line)
                                .chars_at(self.document.rope.len_chars())
                                .reversed()
                                .take_while(|char| char.is_whitespace())
                                .map(|char| char.len_utf8())
                                .sum()
                        },
                    ),
            )
            .as_cursor();

        self
    }

    pub fn prev_grapheme(&mut self, graphemes: usize, wrap: bool) -> &mut Self {
        let mut cursor_ref =
            CursorRef::with(self.document.rope.slice(..)).at_cursor(self.document.selection.head);

        for _ in 0..graphemes {
            if let Some(cursor) = cursor_ref.prev_grapheme() {
                cursor_ref = cursor;
            } else {
                if wrap {
                    cursor_ref = CursorRef::with(self.document.rope.slice(..)).at_end();
                }

                break;
            }
        }

        self.document.selection.head = cursor_ref.as_cursor();
        self
    }

    pub fn next_grapheme(&mut self, graphemes: usize, wrap: bool) -> &mut Self {
        let mut cursor_ref =
            CursorRef::with(self.document.rope.slice(..)).at_cursor(self.document.selection.head);

        for _ in 0..graphemes {
            if let Some(cursor) = cursor_ref.next_grapheme() {
                cursor_ref = cursor;
            } else {
                if wrap {
                    cursor_ref = CursorRef::with(self.document.rope.slice(..)).at_start();
                }

                break;
            }
        }

        self.document.selection.head = cursor_ref.as_cursor();
        self
    }

    pub fn prev_start_of_subword(&mut self, words: usize, wrap: bool) -> &mut Self {
        let mut cursor_ref =
            CursorRef::with(self.document.rope.slice(..)).at_cursor(self.document.selection.head);

        for _ in 0..words {
            if let Some(cursor) = cursor_ref.prev_start_of_subword() {
                cursor_ref = cursor;
            } else {
                if wrap {
                    cursor_ref = CursorRef::with(self.document.rope.slice(..)).at_end();
                }

                break;
            }
        }

        self.document.selection.head = cursor_ref.as_cursor();
        self
    }

    pub fn prev_end_of_subword(&mut self, words: usize, wrap: bool) -> &mut Self {
        let mut cursor_ref =
            CursorRef::with(self.document.rope.slice(..)).at_cursor(self.document.selection.head);

        for _ in 0..words {
            if let Some(cursor) = cursor_ref.prev_end_of_subword() {
                cursor_ref = cursor;
            } else {
                if wrap {
                    cursor_ref = CursorRef::with(self.document.rope.slice(..)).at_end();
                }

                break;
            }
        }

        self.document.selection.head = cursor_ref.as_cursor();
        self
    }

    pub fn next_start_of_subword(&mut self, words: usize, wrap: bool) -> &mut Self {
        let mut cursor_ref =
            CursorRef::with(self.document.rope.slice(..)).at_cursor(self.document.selection.head);

        for _ in 0..words {
            if let Some(cursor) = cursor_ref.next_start_of_subword() {
                cursor_ref = cursor;
            } else {
                if wrap {
                    cursor_ref = CursorRef::with(self.document.rope.slice(..)).at_start();
                }

                break;
            }
        }

        self.document.selection.head = cursor_ref.as_cursor();
        self
    }

    pub fn next_end_of_subword(&mut self, words: usize, wrap: bool) -> &mut Self {
        let mut cursor_ref =
            CursorRef::with(self.document.rope.slice(..)).at_cursor(self.document.selection.head);

        for _ in 0..words {
            if let Some(cursor) = cursor_ref.next_end_of_subword() {
                cursor_ref = cursor;
            } else {
                if wrap {
                    cursor_ref = CursorRef::with(self.document.rope.slice(..)).at_start();
                }

                break;
            }
        }

        self.document.selection.head = cursor_ref.as_cursor();
        self
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
            self.movements().prev_grapheme(1, false);
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
