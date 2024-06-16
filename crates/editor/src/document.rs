use crate::{
    cursor::Cursor,
    rope::{RopeExt, WordClass, WordCursor},
    syntax::{Capture, Theme},
};
use ropey::Rope;
use std::{borrow::Cow, fs::File, io::BufReader, ops::Range};
use tree_sitter::{Node, Parser, Query, Tree};
use virus_graphics::text::{Cluster, Context, FontFamilyKey, FontSize, Line};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Selection {
    pub anchor: Cursor,
    pub head: Cursor,
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
    cached_shaping: Option<CachedShaping>,
    is_tree_dirty: bool,
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
            cached_shaping: None,
            is_tree_dirty: false,
        })
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
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn selection(&self) -> Selection {
        self.selection
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
                self.selection,
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
            self.selection,
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

    fn edit_tree(&mut self, start: Cursor, old_end: Cursor, new_end: Cursor) {
        self.tree
            .edit(&Cursor::into_input_edit(start, old_end, new_end));
        self.is_tree_dirty = true;
        self.cached_shaping = None;
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

struct CachedShaping {
    selection: Selection,
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
        selection: Selection,
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
            selection,
            family,
            theme,
            font_size,
        );

        Self {
            selection,
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
        selection: Selection,
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
                selection,
                family,
                theme,
                font_size,
            );
        } else if self.selection != selection {
            let mut lines = vec![
                self.selection.anchor.line,
                self.selection.head.line,
                selection.anchor.line,
                selection.head.line,
            ];
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
                        selection,
                        family,
                        theme,
                        font_size,
                    );

                    self.lines[line - self.line_range.start] = lines.into_iter().next().unwrap();
                }
            }

            self.selection = selection;
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
        selection: Selection,
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

            let start_line = capture.start.line - line_range.start;
            let end_line = capture.end.line - line_range.start;

            if start_line == end_line {
                let (_, line) = &mut lines[start_line];
                let Some(start) = find(line.clusters(), capture.start.column) else {
                    debug_assert!(false, "Cannot find highlight in line");
                    continue;
                };
                let Some(end) = find(&line.clusters()[start..], capture.end.column) else {
                    debug_assert!(false, "Cannot find highlight in line");
                    continue;
                };

                update(&mut line.clusters_mut()[start..][..end]);
            } else {
                let (_, line) = &mut lines[start_line];
                let Some(start) = find(line.clusters(), capture.start.column) else {
                    debug_assert!(false, "Cannot find highlight in line");
                    continue;
                };

                update(&mut line.clusters_mut()[start..]);

                for (_, line) in &mut lines[start_line..end_line - 1] {
                    update(line.clusters_mut());
                }

                if let Some((_, line)) = lines.get_mut(end_line) {
                    let Some(end) = find(line.clusters(), capture.end.column) else {
                        debug_assert!(false, "Cannot find highlight in line");
                        continue;
                    };

                    update(&mut line.clusters_mut()[..end]);
                } else {
                    debug_assert!(capture.end.column == 0);
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
                    (i == selection.anchor.line).then(|| word(selection.anchor.column)),
                    (i == selection.head.line).then(|| word(selection.head.column)),
                )
            })
            .collect()
    }
}
