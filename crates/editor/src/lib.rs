use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
};
use tree_sitter::{Parser, Tree};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Cursor                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Default, Debug)]
pub struct Cursor {
    index: usize,
    line: usize,
    column: usize,
}

impl Cursor {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
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

    pub fn parser(&self) -> Option<Parser> {
        self.tree_sitter_language().map(|language| {
            let mut parser = Parser::new();
            parser.set_language(language).unwrap();
            parser
        })
    }
}

/// Private.
impl Language {
    fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::language()),
            Self::Yaml => None,
            Self::Markdown => None,
        }
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
                &mut |index, _| self.rope.chunk_at_byte(index).0,
                self.tree.as_ref(),
            );
        }

        self.tree.as_ref()
    }
}
