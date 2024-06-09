use crate::{
    cursor::Cursor,
    rope::RopeExt,
    syntax::{Highlights, Theme},
};
use ropey::Rope;
use std::{borrow::Cow, fs::File, io::BufReader, ops::Range};
use tree_sitter::{Node, Parser, Query, Tree};
use virus_graphics::text::{Context, FontFamilyKey, FontSize, Line, LineShaper, Styles};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Selection                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Default, Debug)]
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

    pub fn selection(&self) -> Selection {
        self.selection
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
        // TODO edit cursors and tree

        let Range { start, end } = self.selection.range();
        self.rope.replace(start..end, str);

        let cursor = self.rope.cursor().index(start.index + str.len());
        self.edit_tree(start, end, cursor);
        self.selection = cursor.into();

        self.cached_shaping = None; // TODO review this
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
            self.cached_shaping = None;
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
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

struct CachedShaping {
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
            family,
            theme,
            font_size,
        );

        Self {
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
                family,
                theme,
                font_size,
            );
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
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
    ) -> Vec<Line> {
        fn shape(
            (shaper, rope): (&mut LineShaper, &Rope),
            (start, end): (&mut usize, usize),
            (family, styles): (FontFamilyKey, Styles),
        ) {
            if !(*start..end).is_empty() {
                shaper.push(&Cow::from(rope.byte_slice(*start..end)), family, styles);
                *start = end;
            }
        }

        let highlights = Highlights::new(rope, root, line_range.clone(), query);
        let mut highlights = highlights.highlights();
        let mut next = highlights.next();

        let mut line = line_range.start;
        let mut index = rope.line_to_byte(line);
        let mut start_of_line = index;

        let unligature1 = None;
        let unligature2 = None;
        let mut shaper = Line::shaper(context, font_size, unligature1.clone(), unligature2.clone());
        let mut lines = vec![];

        loop {
            match next.as_mut() {
                Some(highlight) => {
                    debug_assert!(index <= highlight.start.index);

                    if line == highlight.start.line {
                        // To start of highlight
                        shape(
                            (&mut shaper, rope),
                            (&mut index, highlight.start.index),
                            (family, theme.default),
                        );

                        if line == highlight.end.line {
                            // To end of highlight
                            shape(
                                (&mut shaper, rope),
                                (&mut index, highlight.end.index),
                                (family, theme[highlight.key]),
                            );

                            // Take next highlight
                            next = highlights.next();
                        } else {
                            // To end of line
                            shape(
                                (&mut shaper, rope),
                                (&mut index, start_of_line + rope.line(line).len_bytes()),
                                (family, theme[highlight.key]),
                            );
                            line += 1;
                            start_of_line = index;

                            // Flush line
                            lines.push(shaper.line());
                            shaper = Line::shaper(
                                context,
                                font_size,
                                unligature1.clone(),
                                unligature2.clone(),
                            );

                            // Crop highlight
                            highlight.start = Cursor::new(index, line, 0);
                            if highlight.start == highlight.end {
                                next = highlights.next();
                            }
                        }
                    } else {
                        // To end of line
                        shape(
                            (&mut shaper, rope),
                            (&mut index, start_of_line + rope.line(line).len_bytes()),
                            (family, theme.default),
                        );
                        line += 1;
                        start_of_line = index;

                        // Flush line
                        lines.push(shaper.line());
                        shaper = Line::shaper(
                            context,
                            font_size,
                            unligature1.clone(),
                            unligature2.clone(),
                        );

                        // Keep highlight as is (it is below)
                    }
                }
                None => {
                    // We may just have flushed
                    if line == line_range.end {
                        break;
                    }

                    // To end of line
                    shape(
                        (&mut shaper, rope),
                        (&mut index, start_of_line + rope.line(line).len_bytes()),
                        (family, theme.default),
                    );

                    // Flush line
                    lines.push(shaper.line());

                    // Flush last lines
                    for line in line + 1..line_range.end {
                        shaper = Line::shaper(
                            context,
                            font_size,
                            unligature1.clone(),
                            unligature2.clone(),
                        );
                        shaper.push(&Cow::from(rope.line(line)), family, theme.default);
                        lines.push(shaper.line());
                    }

                    break;
                }
            }
        }

        debug_assert!(lines.len() == line_range.len());
        lines
    }
}
