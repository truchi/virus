use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
};
use tree_sitter::{Node, Parser, Point, Query, QueryCursor, Tree};
use virus_common::{Cursor, Rgba, Style};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Theme                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Theme {
    pub default: Style,
    pub attribute: Style,
    pub comment: Style,
    pub constant: Style,
    pub constant_builtin_boolean: Style,
    pub constant_character: Style,
    pub constant_character_escape: Style,
    pub constant_numeric_float: Style,
    pub constant_numeric_integer: Style,
    pub constructor: Style,
    pub function: Style,
    pub function_macro: Style,
    pub function_method: Style,
    pub keyword: Style,
    pub keyword_control: Style,
    pub keyword_control_conditional: Style,
    pub keyword_control_import: Style,
    pub keyword_control_repeat: Style,
    pub keyword_control_return: Style,
    pub keyword_function: Style,
    pub keyword_operator: Style,
    pub keyword_special: Style,
    pub keyword_storage: Style,
    pub keyword_storage_modifier: Style,
    pub keyword_storage_modifier_mut: Style,
    pub keyword_storage_modifier_ref: Style,
    pub keyword_storage_type: Style,
    pub label: Style,
    pub namespace: Style,
    pub operator: Style,
    pub punctuation_bracket: Style,
    pub punctuation_delimiter: Style,
    pub special: Style,
    pub string: Style,
    pub r#type: Style,
    pub type_builtin: Style,
    pub type_enum_variant: Style,
    pub variable: Style,
    pub variable_builtin: Style,
    pub variable_other_member: Style,
    pub variable_parameter: Style,
}

impl Theme {
    pub fn default(&self) -> &Style {
        &self.default
    }

    pub fn get(&self, key: &str) -> &Style {
        match key {
            "attribute" => &self.attribute,
            "comment" => &self.comment,
            "constant" => &self.constant,
            "constant.builtin.boolean" => &self.constant_builtin_boolean,
            "constant.character" => &self.constant_character,
            "constant.character.escape" => &self.constant_character_escape,
            "constant.numeric.float" => &self.constant_numeric_float,
            "constant.numeric.integer" => &self.constant_numeric_integer,
            "constructor" => &self.constructor,
            "function" => &self.function,
            "function.macro" => &self.function_macro,
            "function.method" => &self.function_method,
            "keyword" => &self.keyword,
            "keyword.control" => &self.keyword_control,
            "keyword.control.conditional" => &self.keyword_control_conditional,
            "keyword.control.import" => &self.keyword_control_import,
            "keyword.control.repeat" => &self.keyword_control_repeat,
            "keyword.control.return" => &self.keyword_control_return,
            "keyword.function" => &self.keyword_function,
            "keyword.operator" => &self.keyword_operator,
            "keyword.special" => &self.keyword_special,
            "keyword.storage" => &self.keyword_storage,
            "keyword.storage.modifier" => &self.keyword_storage_modifier,
            "keyword.storage.modifier.mut" => &self.keyword_storage_modifier_mut,
            "keyword.storage.modifier.ref" => &self.keyword_storage_modifier_ref,
            "keyword.storage.type" => &self.keyword_storage_type,
            "label" => &self.label,
            "namespace" => &self.namespace,
            "operator" => &self.operator,
            "punctuation.bracket" => &self.punctuation_bracket,
            "punctuation.delimiter" => &self.punctuation_delimiter,
            "special" => &self.special,
            "string" => &self.string,
            "type" => &self.r#type,
            "type.builtin" => &self.type_builtin,
            "type.enum.variant" => &self.type_enum_variant,
            "variable" => &self.variable,
            "variable.builtin" => &self.variable_builtin,
            "variable.other.member" => &self.variable_other_member,
            "variable.parameter" => &self.variable_parameter,
            _ => &self.default,
        }
    }

    pub fn dracula() -> Self {
        fn style(r: u8, g: u8, b: u8) -> Style {
            Style {
                foreground: Rgba {
                    r,
                    g,
                    b,
                    a: u8::MAX,
                },
                ..Default::default()
            }
        }

        let _background = style(40, 42, 54);
        let _current = style(68, 71, 90);
        let foreground = style(248, 248, 242);
        let comment = style(98, 114, 164);
        let cyan = style(139, 233, 253);
        let green = style(80, 250, 123);
        let orange = style(255, 184, 108);
        let pink = style(255, 121, 198);
        let purple = style(189, 147, 249);
        let red = style(255, 85, 85);
        let yellow = style(241, 250, 140);

        Theme {
            default: style(255, 255, 255),
            attribute: green,
            comment,
            constant: green,
            constant_builtin_boolean: purple,
            constant_character: purple,
            constant_character_escape: purple,
            constant_numeric_float: purple,
            constant_numeric_integer: purple,
            constructor: foreground,
            function: pink,
            function_macro: pink,
            function_method: pink,
            keyword: red,
            keyword_control: red,
            keyword_control_conditional: red,
            keyword_control_import: red,
            keyword_control_repeat: red,
            keyword_control_return: red,
            keyword_function: red,
            keyword_operator: red,
            keyword_special: red,
            keyword_storage: red,
            keyword_storage_modifier: red,
            keyword_storage_modifier_mut: red,
            keyword_storage_modifier_ref: red,
            keyword_storage_type: red,
            label: foreground,
            namespace: foreground,
            operator: foreground,
            punctuation_bracket: yellow,
            punctuation_delimiter: yellow,
            special: yellow,
            string: cyan,
            r#type: cyan,
            type_builtin: cyan,
            type_enum_variant: cyan,
            variable: orange,
            variable_builtin: orange,
            variable_other_member: orange,
            variable_parameter: orange,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Language                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub enum Language {
    Rust,
    Yaml,
    Markdown,
}

impl Language {
    pub fn iter() -> std::array::IntoIter<Self, 3> {
        [Self::Rust, Self::Yaml, Self::Markdown].into_iter()
    }

    pub fn extensions(&self) -> &[&str] {
        match self {
            Self::Rust => &[".rs"],
            Self::Yaml => &[".yml", ".yaml"],
            Self::Markdown => &[".md", ".markdown"],
        }
    }

    pub fn language(&self) -> Option<tree_sitter::Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::language()),
            Self::Yaml => None,
            Self::Markdown => None,
        }
    }

    pub fn parser(&self) -> Option<Parser> {
        self.language().map(|language| {
            let mut parser = Parser::new();
            parser.set_language(language).unwrap();
            parser
        })
    }
}

impl TryFrom<&str> for Language {
    type Error = ();

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        for language in Self::iter() {
            for extension in language.extensions() {
                if path.ends_with(extension) {
                    return Ok(language);
                }
            }
        }

        Err(())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Document                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Default)]
pub struct Document {
    path: Option<String>,
    rope: Rope,
    selection: Range<Cursor>, // Byte cursor, but ropey uses char indices...
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
            selection: Cursor::START..Cursor::START,
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

    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|path| path.as_str())
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn selection(&self) -> Range<Cursor> {
        self.selection.clone()
    }

    pub fn selection_mut(&mut self) -> &mut Range<Cursor> {
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

    pub fn edit_str(&mut self, str: &str) {
        // TODO edit cursors and tree

        let start = self.selection.start.index;
        let end = self.selection.end.index;
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
        let start = self.selection.start.index;
        let end = self.selection.end.index;
        let start_char = self.rope.byte_to_char(start);

        if start != end {
            let end_char = self.rope.byte_to_char(end);
            self.rope.remove(start_char..end_char);
            self.dirty = true;
        }

        self.rope.insert_char(start_char, char);

        let cursor = self.cursor_at_index(end + char.len_utf8()); // FIXME start instead of end?
        self.edit_tree(self.selection.start, self.selection.end, cursor);
        self.selection = cursor..cursor;
    }

    pub fn backspace(&mut self) -> Result<(), ()> {
        if self.selection.start != self.selection.end {
            return Err(());
        }

        // Disgusting: we have to go from byte to char index, then from char to byte index...

        let char_index = self.rope.byte_to_char(self.selection.start.index);

        if char_index == 0 {
            return Ok(());
        }

        if let Some(prev_char_index) = char_index.checked_sub(1) {
            self.rope.remove(prev_char_index..char_index);

            let cursor = self.cursor_at_index(self.rope.char_to_byte(prev_char_index));
            self.edit_tree(cursor, self.selection.end, cursor);
            self.selection = cursor..cursor;
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

    fn cursor_at_index(&self, index: usize) -> Cursor {
        let line = self.rope.byte_to_line(index);

        Cursor {
            index,
            line,
            column: index - self.rope.line_to_byte(line),
        }
    }

    fn edit_tree(&mut self, start: Cursor, old_end: Cursor, new_end: Cursor) {
        if let Some(tree) = &mut self.tree {
            tree.edit(&Cursor::input_edit(start, old_end, new_end));
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlights                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct Highlight {
    pub start: Cursor,
    pub end: Cursor,
    pub style: Style,
}

/// Procuces an iterator of [`Highlight`]s.
pub struct Highlights<'tree, 'rope> {
    rope: &'rope Rope,
    root: Node<'tree>,
    start: Cursor,
    end: Cursor,
    query: Query,
    theme: Theme,
}

impl<'tree, 'rope> Highlights<'tree, 'rope> {
    /// Creates a new [`Highlights`] for `rope` with `root`,
    /// clamped by `lines`, for `query` with `theme`.
    pub fn new(
        rope: &'rope Rope,
        root: Node<'tree>,
        lines: Range<usize>,
        query: Query,
        theme: Theme,
    ) -> Self {
        let Range { start, end } = lines;

        let lines = rope.len_lines();
        let end = end.min(lines);
        let start = start.min(end);

        let start = Cursor::new(rope.line_to_byte(start), start, 0);
        let end = Cursor::new(rope.line_to_byte(end), end, 0);

        Self {
            rope,
            root,
            query,
            start,
            end,
            theme,
        }
    }

    /// Returns an iterator of [`Highlight`]s.
    pub fn iter(&self) -> impl '_ + Iterator<Item = Highlight> {
        // Use `tree-sitter` to get a sorted list of catpures for `self.query`
        let captures = if (self.start.line..self.end.line).is_empty() {
            vec![]
        } else {
            struct Capture<'a> {
                start: Cursor,
                end: Cursor,
                pattern: usize,
                name: &'a str,
            }

            let mut captures = Vec::<Capture>::new();
            let mut cursor = {
                let start = Point::new(self.start.line, 0);
                let end = Point::new(self.end.line, 0);
                let mut cursor = QueryCursor::new();
                cursor.set_point_range(start..end);
                cursor
            };
            let it = cursor
                .matches(&self.query, self.root, |node: Node| {
                    self.rope
                        .get_byte_slice(node.byte_range())
                        .unwrap()
                        .chunks()
                        .map(|chunk| chunk.as_bytes())
                })
                .map(|captures| {
                    captures.captures.into_iter().map(move |capture| Capture {
                        start: Cursor::new(
                            capture.node.start_byte(),
                            capture.node.start_position().row,
                            capture.node.start_position().column,
                        ),
                        end: Cursor::new(
                            capture.node.end_byte(),
                            capture.node.end_position().row,
                            capture.node.end_position().column,
                        ),
                        pattern: captures.pattern_index,
                        name: &self.query.capture_names()[capture.index as usize],
                    })
                })
                .flatten();

            for capture in it {
                // We want all captures ordered by start index,
                // favoring lower pattern index when captured multiple times.
                // This is what `Helix` does, and we use their queries.
                // It seems like patterns are written in that specific order.
                match captures
                    .binary_search_by_key(&capture.start.index, |capture| capture.start.index)
                {
                    Ok(index) => {
                        // Favoring lower index pattern
                        if captures[index].pattern > capture.pattern {
                            captures[index] = capture
                        }
                    }
                    Err(index) => {
                        // Captures must not overlap, otherwise what can we do?
                        debug_assert!(
                            index
                                .checked_sub(1)
                                .map(|prev| captures[prev].end.index <= capture.start.index)
                                .unwrap_or(true),
                            "overlapping capture",
                        );

                        captures.insert(index, capture);
                    }
                }
            }

            if captures.is_empty() {
                vec![Capture {
                    start: self.start,
                    end: self.end,
                    pattern: usize::MAX,
                    name: "",
                }]
            } else {
                captures
            }
        };

        // Filter on line range and crop overlapping captures
        let highlights = captures
            .into_iter()
            .map(|highlight| Highlight {
                start: highlight.start,
                end: highlight.end,
                style: *self.theme.get(highlight.name),
            })
            .filter(|highlight| self.start.index < highlight.end.index)
            .filter(|highlight| highlight.start.index < self.end.index)
            .map(|highlight| Highlight {
                start: if self.start.index < highlight.start.index {
                    highlight.start
                } else {
                    self.start
                },
                end: if highlight.end.index < self.end.index {
                    highlight.end
                } else {
                    self.end
                },
                style: highlight.style,
            });

        // Intersperse with in-between selections
        let highlights = {
            let mut highlights = highlights.peekable();
            let mut prev = Highlight {
                start: self.start,
                end: self.start,
                style: *self.theme.default(),
            };

            std::iter::from_fn(move || {
                let next = highlights.peek()?;

                prev = if prev.end.index == next.start.index {
                    highlights.next()?
                } else {
                    Highlight {
                        start: prev.end,
                        end: next.start,
                        style: *self.theme.default(),
                    }
                };

                Some(prev)
            })
        };

        // Slice highlights to line boundaries
        let highlights = {
            let mut highlights = highlights;
            let mut next = highlights.next();

            std::iter::from_fn(move || {
                let highlight = next?;

                if highlight.start.line == highlight.end.line {
                    next = highlights.next();
                    Some(highlight)
                } else {
                    // NOTE: this does not take line breaks into account!
                    // It could be nice to remove the line break (if any) from the item,
                    // but this should not be a real issue.
                    let end = self
                        .rope
                        .try_line_to_byte(highlight.start.line + 1)
                        .unwrap();

                    next = Some(Highlight {
                        start: Cursor::new(end, highlight.start.line + 1, 0),
                        end: highlight.end,
                        style: highlight.style,
                    });
                    Some(Highlight {
                        start: highlight.start,
                        end: Cursor::new(
                            end,
                            highlight.start.line,
                            highlight.start.column + (end - highlight.start.index),
                        ),
                        style: highlight.style,
                    })
                }
            })
            .filter(|highlight| highlight.start.index != highlight.end.index)
        };

        // That was hard! Let's make sure we made it right:
        highlights.inspect(|highlight| {
            // In the requested line range
            debug_assert!((self.start.line..self.end.line).contains(&highlight.start.line));
            debug_assert!((self.start.line..self.end.line).contains(&highlight.end.line));
            // One-line
            debug_assert!(highlight.start.line == highlight.end.line);
            // Not empty
            debug_assert!(highlight.start.index != highlight.end.index);
            debug_assert!(highlight.end.column != 0);
        })
    }
}
