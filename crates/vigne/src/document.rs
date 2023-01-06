use crate::{cursor::Cursor, language::Language};
use ropey::Rope;
use std::{
    convert::TryFrom,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
};
use tree_sitter::{Parser, Tree};

#[derive(Default)]
pub struct Document {
    path: Option<String>,
    rope: Rope,
    selection: Range<Cursor>,
    parser: Option<Parser>,
    tree: Option<Tree>,
    language: Option<Language>,
    dirty: bool,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(path: String) -> std::io::Result<Self> {
        let rope = Rope::from_reader(&mut BufReader::new(File::open(&path)?))?;
        let cursor = Default::default();
        let language = Language::try_from(path.as_str()).ok();

        let (parser, tree) = if let Some(language) = language
            .map(|language| language.tree_sitter_language())
            .flatten()
        {
            let mut parser = Parser::new();
            parser.set_language(language).unwrap();

            let tree = parse(&mut parser, &rope, None);
            (Some(parser), tree)
        } else {
            (None, None)
        };

        Ok(Self {
            path: Some(path),
            rope,
            selection: cursor,
            tree,
            parser,
            language,
            dirty: false,
        })
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = self.path.as_ref() {
            let mut writer =
                BufWriter::new(OpenOptions::new().write(true).truncate(true).open(path)?);

            for chunk in self.rope.chunks() {
                writer.write(chunk.as_bytes())?;
            }
        }

        Ok(())
    }

    pub fn edit_str(&mut self, str: &str) {
        let start = self.selection.start.index();
        let end = self.selection.end.index();
        let start_char = self.rope.byte_to_char(start);

        if start != end {
            let end_char = self.rope.byte_to_char(end);
            self.rope.remove(start_char..end_char);
        }

        if !str.is_empty() {
            self.rope.insert(start_char, str);
        }

        self.parse();
    }

    pub fn edit_char(&mut self, char: char) {
        let start = self.selection.start.index();
        let end = self.selection.end.index();
        let start_char = self.rope.byte_to_char(start);

        if start != end {
            let end_char = self.rope.byte_to_char(end);
            self.rope.remove(start_char..end_char);
        }

        self.rope.insert_char(start_char, char);

        self.parse();
    }
}

/// Private.
impl Document {
    fn parse(&mut self) {
        if let Some(parser) = self.parser.as_mut() {
            self.tree = parse(parser, &mut self.rope, self.tree.as_ref());
        }
    }
}

fn parse(parser: &mut Parser, rope: &Rope, old_tree: Option<&Tree>) -> Option<Tree> {
    parser.parse_with(&mut |index, _| rope.chunk_at_byte(index).0, old_tree)
}
