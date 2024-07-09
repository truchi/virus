use crate::{async_actor::AsyncActorSender, document::Document};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Debug)]
pub enum EventLoopMessage {}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub type EventLoopSender = Box<dyn Fn(EventLoopMessage) + Send>;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct Editor {
    root: PathBuf,
    documents: Vec<Document>,
    active_document: usize,
    async_actor_sender: AsyncActorSender,
    event_loop_sender: EventLoopSender,
}

impl Editor {
    pub fn new(
        root: PathBuf,
        async_actor_sender: AsyncActorSender,
        event_loop_sender: EventLoopSender,
    ) -> Self {
        Self {
            root,
            documents: Default::default(),
            active_document: 0,
            async_actor_sender,
            event_loop_sender,
        }
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn active_document(&self) -> &Document {
        self.documents.get(self.active_document).unwrap()
    }

    pub fn active_document_mut(&mut self) -> &mut Document {
        self.documents.get_mut(self.active_document).unwrap()
    }

    pub fn open(&mut self, path: PathBuf) -> std::io::Result<()> {
        if let Some(active_document) = self
            .documents
            .iter()
            .position(|document| document.path() == path)
        {
            self.active_document = active_document;
        } else {
            let mut document = Document::open(path)?;
            document.parse();

            self.active_document = self.documents.len();
            self.documents.push(document);
        }

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

    pub fn find_git_root(path: PathBuf) -> Option<PathBuf> {
        let mut current = match std::fs::canonicalize(path) {
            Ok(path) => Some(path),
            Err(_) => return None,
        };

        while let Some(path) = current {
            if path.join(".git").is_dir() {
                return Some(path);
            }

            current = path.parent().map(Into::into);
        }

        None
    }
}