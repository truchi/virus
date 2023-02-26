use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
};
use tree_sitter::{Node, Parser, Point, Query, QueryCursor, Tree};
use virus_common::{Cursor, Style};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Theme                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Theme {
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
    pub fn get(&self, key: &str) -> Option<&Style> {
        match key {
            "attribute" => Some(&self.attribute),
            "comment" => Some(&self.comment),
            "constant" => Some(&self.constant),
            "constant.builtin.boolean" => Some(&self.constant_builtin_boolean),
            "constant.character" => Some(&self.constant_character),
            "constant.character.escape" => Some(&self.constant_character_escape),
            "constant.numeric.float" => Some(&self.constant_numeric_float),
            "constant.numeric.integer" => Some(&self.constant_numeric_integer),
            "constructor" => Some(&self.constructor),
            "function" => Some(&self.function),
            "function.macro" => Some(&self.function_macro),
            "function.method" => Some(&self.function_method),
            "keyword" => Some(&self.keyword),
            "keyword.control" => Some(&self.keyword_control),
            "keyword.control.conditional" => Some(&self.keyword_control_conditional),
            "keyword.control.import" => Some(&self.keyword_control_import),
            "keyword.control.repeat" => Some(&self.keyword_control_repeat),
            "keyword.control.return" => Some(&self.keyword_control_return),
            "keyword.function" => Some(&self.keyword_function),
            "keyword.operator" => Some(&self.keyword_operator),
            "keyword.special" => Some(&self.keyword_special),
            "keyword.storage" => Some(&self.keyword_storage),
            "keyword.storage.modifier" => Some(&self.keyword_storage_modifier),
            "keyword.storage.modifier.mut" => Some(&self.keyword_storage_modifier_mut),
            "keyword.storage.modifier.ref" => Some(&self.keyword_storage_modifier_ref),
            "keyword.storage.type" => Some(&self.keyword_storage_type),
            "label" => Some(&self.label),
            "namespace" => Some(&self.namespace),
            "operator" => Some(&self.operator),
            "punctuation.bracket" => Some(&self.punctuation_bracket),
            "punctuation.delimiter" => Some(&self.punctuation_delimiter),
            "special" => Some(&self.special),
            "string" => Some(&self.string),
            "type" => Some(&self.r#type),
            "type.builtin" => Some(&self.type_builtin),
            "type.enum.variant" => Some(&self.type_enum_variant),
            "variable" => Some(&self.variable),
            "variable.builtin" => Some(&self.variable_builtin),
            "variable.other.member" => Some(&self.variable_other_member),
            "variable.parameter" => Some(&self.variable_parameter),
            _ => None,
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
    selection: Range<Cursor>,
    language: Option<Language>,
    parser: Option<Parser>,
    tree: Option<Tree>,
    dirty: bool,
}

impl Document {
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

    pub fn edit_str(&mut self, str: &str) {
        // TODO edit cursors and tree

        let start = self.selection.start.index;
        let end = self.selection.end.index;
        let start_char = self.rope.byte_to_char(start);
        let mut dirty = false;

        if start != end {
            let end_char = self.rope.byte_to_char(end);
            self.rope.remove(start_char..end_char);
            dirty = true;
        }

        if !str.is_empty() {
            self.rope.insert(start_char, str);
            dirty = true;
        }

        if dirty {
            self.dirty = true;
        }
    }

    pub fn edit_char(&mut self, char: char) {
        // TODO edit cursors and tree

        let start = self.selection.start.index;
        let end = self.selection.end.index;
        let start_char = self.rope.byte_to_char(start);
        let mut dirty = false;

        if start != end {
            let end_char = self.rope.byte_to_char(end);
            self.rope.remove(start_char..end_char);
            dirty = true;
        }

        self.rope.insert_char(start_char, char);

        if dirty {
            self.dirty = true;
        }
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlights                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct Highlight {
    pub start: Cursor,
    pub end: Cursor,
    pub style: Option<Style>,
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
        let start = Cursor::new(rope.try_line_to_byte(start).unwrap(), start, 0);
        let end = Cursor::new(rope.try_line_to_byte(end).unwrap(), end, 0);

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
            #[derive(Debug)] // TODO remove
            struct Capture {
                start: Cursor,
                end: Cursor,
                pattern: usize,
                capture: usize,
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
                        capture: capture.index as usize,
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
                            captures[index].capture = capture.capture
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

            captures
        };

        // Intersperse with in-between selections
        let highlights = {
            let mut prev = Option::<Highlight>::None;
            let mut highlights = captures
                .into_iter()
                .map(|highlight| Highlight {
                    start: highlight.start,
                    end: highlight.end,
                    style: self
                        .theme
                        .get(&self.query.capture_names()[highlight.capture])
                        .copied(),
                })
                .peekable();

            std::iter::from_fn(move || {
                if let Some(prev) = prev.take() {
                    let next = highlights.peek()?;

                    Some(Highlight {
                        start: prev.end,
                        end: next.start,
                        style: None,
                    })
                } else {
                    prev = highlights.next();
                    prev
                }
            })
            .filter(|highlight| highlight.start.index != highlight.end.index)
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
                    // FIXME: this supposes `\n` boundaries!
                    let index = self
                        .rope
                        .try_line_to_byte(highlight.start.line + 1)
                        .unwrap()
                        - /* eol */ 1;

                    next = Some(Highlight {
                        start: Cursor::new(index + /* eol */ 1, highlight.start.line + 1, 0),
                        end: highlight.end,
                        style: highlight.style,
                    });
                    Some(Highlight {
                        start: highlight.start,
                        end: Cursor::new(
                            index,
                            highlight.start.line,
                            highlight.start.column + (index - highlight.start.index),
                        ),
                        style: highlight.style,
                    })
                }
            })
            .filter(|highlight| highlight.start.index != highlight.end.index)
        };

        // Finally, filter out highlights not in the requested line range
        highlights
            .filter(|highlight| (self.start.line..self.end.line).contains(&highlight.start.line))
    }
}
