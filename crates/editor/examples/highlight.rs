use editor::Document;
use ropey::Rope;
use std::{borrow::Cow, ops::Range};
use tree_sitter::{Node, Point, Query, QueryCursor};
use virus_common::{Cursor, Rgba};

const HIGHLIGHT_QUERY: &str = include_str!("../treesitter/rust/highlights.scm");

fn main() {
    let mut document =
        Document::open("/home/romain/perso/virus/crates/virus/src/main.rs".into()).unwrap();
    dbg!(document.language());
    document.parse();

    let rope = document.rope();
    let language = document.language().unwrap();
    let parser = document.parser().unwrap();
    let tree = document.tree().unwrap();

    let theme = Default::default();
    let highlights = Highlights::new(
        tree.root_node(),
        &rope,
        0..10000000,
        Query::new(language.language().unwrap(), HIGHLIGHT_QUERY).unwrap(),
        &theme,
    );

    for (_, cow, style) in highlights.iter() {
        if style.is_none() {
            print!("\x1B[0;31m{cow}\x1B[0m");
        } else {
            print!("\x1B[0;30m{cow}\x1B[0m");
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Style {
    foreground: Rgba,
    background: Rgba,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
}

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
