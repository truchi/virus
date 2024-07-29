use crate::{
    history::History,
    lsp::Lsp,
    rope::{CursorRef, Edit, Selection, Text, WordClass, WordCursor},
    syntax::{Capture, Theme},
};
use ropey::Rope;
use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
    path::{Path, PathBuf},
};
use tree_sitter::{Node, Parser, Query, Tree};
use virus_graphics::text::{Cluster, Context, FontFamilyKey, FontSize, Line};

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
    cached_shaping: Option<CachedShaping>,
    lsp: Lsp,
}

impl Document {
    // NOTE:
    // This function is convenient for now.
    // We will need to deal with unsupported languages later.
    pub fn open(id: DocumentId, path: PathBuf, lsp: Lsp) -> std::io::Result<Self> {
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
            cached_shaping: None,
            lsp: lsp.clone(),
        };

        lsp.open_document(&document);

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
        let lsp_edit = edit.to_lsp_edit_applied();

        self.selection = edit.inserted_end().into();
        self.version += 1;
        self.is_tree_dirty = true;
        self.cached_shaping = None;
        self.history.push(edit);
        self.tree.edit(&ts_edit);
        self.lsp.change_document(&self, [lsp_edit]);
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
        let mut lsp_edits = Vec::with_capacity(edits.len());

        for edit in edits {
            edit.unapply(&mut self.rope);
            self.tree.edit(&edit.to_ts_edit_applied());
            lsp_edits.push(edit.to_lsp_edit_unapplied());

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
        self.cached_shaping = None;
        self.lsp.change_document(&self, lsp_edits);
    }

    pub fn redo(&mut self) {
        let Some(edits) = self.history.redo() else {
            return;
        };
        let mut anchor = self.selection.anchor;
        let mut head = self.selection.head;
        let mut lsp_edits = Vec::with_capacity(edits.len());

        for edit in edits {
            edit.apply(&mut self.rope);
            self.tree.edit(&edit.to_ts_edit_applied());
            lsp_edits.push(edit.to_lsp_edit_applied());

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
        self.cached_shaping = None;
        self.lsp.change_document(&self, lsp_edits);
    }
}

///
impl Document {
    pub fn shape(
        &mut self,
        context: &mut Context,
        lines: Range<usize>,
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
    ) -> &[Line] {
        debug_assert!(lines.start <= lines.end);
        debug_assert!(lines.end <= self.rope.len_lines());

        if self.cached_shaping.is_none() {
            self.cached_shaping = Some(CachedShaping::new(
                context,
                &self.rope,
                self.tree.root_node(),
                &self.highlights,
                lines.clone(),
                self.selection.anchor.line(),
                self.selection.anchor.column(),
                self.selection.head.line(),
                self.selection.head.column(),
                family,
                theme,
                font_size,
            ));
        }

        self.cached_shaping.as_mut().expect("Just created it").get(
            context,
            &self.rope,
            self.tree.root_node(),
            &self.highlights,
            lines,
            self.selection.anchor.line(),
            self.selection.anchor.column(),
            self.selection.head.line(),
            self.selection.head.column(),
            family,
            theme,
            font_size,
        )
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

// ────────────────────────────────────────────────────────────────────────────────────────────── //

struct CachedShaping {
    anchor_line: usize,
    anchor_column: usize,
    head_line: usize,
    head_column: usize,
    family: FontFamilyKey,
    theme: Theme,
    font_size: FontSize,
    line_range: Range<usize>,
    lines: Vec<Line>,
}

impl CachedShaping {
    fn new(
        context: &mut Context,
        rope: &Rope,
        root: Node,
        query: &Query,
        line_range: Range<usize>,
        anchor_line: usize,
        anchor_column: usize,
        head_line: usize,
        head_column: usize,
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
    ) -> Self {
        let line_range = {
            let margin = line_range.len() / 2;
            let start = line_range.start.saturating_sub(margin);
            let end = (line_range.end + margin).min(rope.len_lines());
            start..end
        };
        let lines = Self::shape(
            context,
            rope,
            root,
            query,
            line_range.clone(),
            anchor_line,
            anchor_column,
            head_line,
            head_column,
            family,
            theme,
            font_size,
        );

        Self {
            anchor_line,
            anchor_column,
            head_line,
            head_column,
            family,
            theme,
            font_size,
            line_range,
            lines,
        }
    }

    fn get(
        &mut self,
        context: &mut Context,
        rope: &Rope,
        root: Node,
        query: &Query,
        line_range: Range<usize>,
        anchor_line: usize,
        anchor_column: usize,
        head_line: usize,
        head_column: usize,
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
    ) -> &[Line] {
        let in_cache = self.family == family
            && self.theme == theme
            && self.font_size == font_size
            && self.line_range.start <= line_range.start
            && line_range.end <= self.line_range.end;

        if !in_cache {
            *self = Self::new(
                context,
                rope,
                root,
                query,
                line_range.clone(),
                anchor_line,
                anchor_column,
                head_line,
                head_column,
                family,
                theme,
                font_size,
            );
        } else if (
            self.anchor_line,
            self.anchor_column,
            self.head_line,
            self.head_column,
        ) != (anchor_line, anchor_column, head_line, head_column)
        {
            let mut lines = vec![self.anchor_line, self.head_line, anchor_line, head_line];
            lines.sort();
            lines.dedup();

            for line in lines {
                if self.line_range.contains(&line) {
                    let lines = Self::shape(
                        context,
                        rope,
                        root,
                        query,
                        line..line + 1,
                        anchor_line,
                        anchor_column,
                        head_line,
                        head_column,
                        family,
                        theme,
                        font_size,
                    );

                    self.lines[line - self.line_range.start] = lines.into_iter().next().unwrap();
                }
            }

            self.anchor_line = anchor_line;
            self.anchor_column = anchor_column;
            self.head_line = head_line;
            self.head_column = head_column;
        }

        let start = line_range.start - self.line_range.start;
        let end = start + line_range.len();
        &self.lines[start..end]
    }

    fn shape(
        context: &mut Context,
        rope: &Rope,
        root: Node,
        query: &Query,
        line_range: Range<usize>,
        anchor_line: usize,
        anchor_column: usize,
        head_line: usize,
        head_column: usize,
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
    ) -> Vec<Line> {
        let mut lines = rope
            .lines_at(line_range.start)
            .take(line_range.len())
            .map(|slice| {
                (
                    slice,
                    Line::shaper(&Cow::from(slice), usize::MAX, theme.default),
                )
            })
            .collect::<Vec<_>>();

        debug_assert!(lines.len() == line_range.len());

        for capture in Capture::captures(rope, root, line_range.clone(), query) {
            let find = |clusters: &[Cluster], column| {
                clusters
                    .iter()
                    .position(|cluster| cluster.range().contains(&column))
            };
            let update = |clusters: &mut [Cluster]| {
                for cluster in clusters {
                    if capture.pattern < cluster.pattern() {
                        *cluster.pattern_mut() = capture.pattern;
                        *cluster.styles_mut() = theme[capture.key];
                    }
                }
            };

            let start_line = capture.start_line - line_range.start;
            let end_line = capture.end_line - line_range.start;

            if start_line == end_line {
                let (_, line) = &mut lines[start_line];
                let Some(start) = find(line.clusters(), capture.start_column) else {
                    debug_assert!(false, "Cannot find highlight in line");
                    continue;
                };
                let Some(end) = find(&line.clusters()[start..], capture.end_column) else {
                    debug_assert!(false, "Cannot find highlight in line");
                    continue;
                };

                update(&mut line.clusters_mut()[start..][..end]);
            } else {
                let (_, line) = &mut lines[start_line];
                let Some(start) = find(line.clusters(), capture.start_column) else {
                    debug_assert!(false, "Cannot find highlight in line");
                    continue;
                };

                update(&mut line.clusters_mut()[start..]);

                for (_, line) in &mut lines[start_line..end_line - 1] {
                    update(line.clusters_mut());
                }

                if let Some((_, line)) = lines.get_mut(end_line) {
                    let Some(end) = find(line.clusters(), capture.end_column) else {
                        debug_assert!(false, "Cannot find highlight in line");
                        continue;
                    };

                    update(&mut line.clusters_mut()[..end]);
                } else {
                    debug_assert!(capture.end_column == 0);
                }
            }
        }

        lines
            .into_iter()
            .enumerate()
            .map(|(i, (slice, line))| {
                let i = line_range.start + i;
                let word = |index| {
                    let mut cursor = WordCursor::new(slice, index);
                    let mut start = index;
                    let mut end = index;

                    while let Some((range, WordClass::Punctuation(_))) = cursor.prev() {
                        start = range.start;
                    }

                    cursor.set_index(index);

                    while let Some((range, WordClass::Punctuation(_))) = cursor.next() {
                        end = range.end;
                    }

                    start..=end
                };

                line.shape(
                    context,
                    family,
                    font_size,
                    (i == anchor_line).then(|| word(anchor_column)),
                    (i == head_line).then(|| word(head_column)),
                )
            })
            .collect()
    }
}
