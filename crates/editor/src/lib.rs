use ropey::Rope;
use std::{
    borrow::Cow,
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

        let start = self.selection.start.index();
        let end = self.selection.end.index();
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

        let start = self.selection.start.index();
        let end = self.selection.end.index();
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

#[derive(Debug)]
pub struct Highlights<'tree, 'rope, 'theme> {
    root: Node<'tree>,
    rope: &'rope Rope,
    lines: Range<usize>,
    query: Query,
    theme: &'theme Theme,
}

impl<'tree, 'rope, 'theme> Highlights<'tree, 'rope, 'theme> {
    pub fn new(
        root: Node<'tree>,
        rope: &'rope Rope,
        lines: Range<usize>,
        query: Query,
        theme: &'theme Theme,
    ) -> Self {
        Self {
            root,
            rope,
            query,
            lines,
            theme,
        }
    }

    pub fn iter(
        &self,
    ) -> impl '_ + Iterator<Item = (Range<Cursor>, Cow<'rope, str>, Option<&'theme Style>)> {
        let items = {
            struct Item<'a> {
                node: Node<'a>,
                pattern: usize,
                capture: usize,
            }

            // We want all captures ordered by start index,
            // favoring lower pattern index when captured multiple times.
            // This is what Helix does, and we use their queries.
            // Seems like pattern are writen in that specific order.
            let mut items = Vec::<Item>::new();

            let mut cursor = QueryCursor::new();
            cursor.set_point_range(Point::new(self.lines.start, 0)..Point::new(self.lines.end, 0));

            let matches = cursor.matches(&self.query, self.root, |node: Node| {
                self.rope
                    .get_byte_slice(node.byte_range())
                    .unwrap()
                    .chunks()
                    .map(|chunk| chunk.as_bytes())
            });

            for captures in matches {
                let pattern = captures.pattern_index;

                for capture in captures.captures {
                    let node = capture.node;
                    let capture = capture.index as usize;

                    match items
                        .binary_search_by_key(&node.start_byte(), |item| item.node.start_byte())
                    {
                        Ok(index) => {
                            // Favoring lower index pattern.
                            if items[index].pattern > pattern {
                                items[index].capture = capture
                            }
                        }
                        Err(index) => {
                            // Captures must not overlap, otherwise what can we do?
                            debug_assert!(
                                if let Some(prev) = index.checked_sub(1) {
                                    if let Some(prev) = items.get(prev) {
                                        prev.node.end_byte() <= node.start_byte()
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                },
                                "overlapping capture",
                            );

                            items.insert(
                                index,
                                Item {
                                    node,
                                    pattern,
                                    capture,
                                },
                            );
                        }
                    }
                }
            }

            items
        };

        let mut cursor = Cursor::default();
        let mut items = items.into_iter().peekable();

        std::iter::from_fn(move || {
            if let Some(item) = items.peek() {
                let start = Cursor::new(
                    item.node.start_byte(),
                    item.node.start_position().row,
                    item.node.start_position().column,
                );
                let end = Cursor::new(
                    item.node.end_byte(),
                    item.node.end_position().row,
                    item.node.end_position().column,
                );
                assert!(cursor.index() <= start.index());

                // Before item
                if cursor.index() != start.index() {
                    let selection = cursor..start;
                    cursor = start;

                    Some((selection.clone(), self.cow(selection), None))
                }
                // Item
                else {
                    let selection = start..end;
                    cursor = end;

                    Some((
                        selection.clone(),
                        self.cow(selection),
                        self.style(items.next().unwrap().capture),
                    ))
                }
            } else {
                let end = Cursor::new(self.rope.len_bytes(), self.rope.len_lines(), 0);

                // After last item
                if cursor.index() < end.index() {
                    let selection = cursor..end;
                    cursor = end;

                    Some((selection.clone(), self.cow(selection), None))
                }
                // Done
                else {
                    None
                }
            }
        })
    }

    fn cow(&self, selection: Range<Cursor>) -> Cow<'rope, str> {
        Cow::from(
            self.rope
                .get_byte_slice(selection.start.index()..selection.end.index())
                .unwrap(),
        )
    }

    fn style(&self, capture: usize) -> Option<&'theme Style> {
        self.theme.get(&self.query.capture_names()[capture])
    }
}
