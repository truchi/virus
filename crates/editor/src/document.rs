use crate::language::Language;
use ropey::Rope;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    ops::Range,
};
use tree_sitter::{Parser, Query, Tree};
use virus_common::Cursor;

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
            selection: Cursor::ZERO..Cursor::ZERO,
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
