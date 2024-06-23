use ignore::WalkBuilder;

use crate::document::Document;
use std::path::PathBuf;

pub struct Editor {
    root: PathBuf,
    documents: Vec<Document>,
}

impl Editor {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            documents: Default::default(),
        }
    }

    pub fn active_document(&self) -> &Document {
        &self.documents[0]
    }

    pub fn active_document_mut(&mut self) -> &mut Document {
        &mut self.documents[0]
    }

    pub fn open(&mut self, path: PathBuf) -> std::io::Result<()> {
        let mut document = Document::open(path)?;
        document.parse();

        self.documents.push(document);

        Ok(())
    }

    pub fn files(&self, hidden: bool, ignored: bool) -> impl '_ + Iterator<Item = PathBuf> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(!hidden)
            .parents(!ignored)
            .ignore(!ignored)
            .git_ignore(!ignored)
            .git_global(!ignored)
            .git_exclude(!ignored)
            .filter_entry(|entry| {
                entry
                    .path()
                    .file_name()
                    .map(|name| name != ".git")
                    .unwrap_or_default()
            })
            .sort_by_file_path(|a, b| match (a.is_dir(), b.is_dir()) {
                (true, true) | (false, false) => a.cmp(b),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
            })
            .build();

        walker
            .filter_map(Result::ok)
            .filter(|entry| matches!(entry.file_type(), Some(ty) if ty.is_file()))
            .filter_map(|entry| {
                entry
                    .into_path()
                    .strip_prefix(&self.root)
                    .map(|path| path.to_owned())
                    .ok()
            })
    }
}
