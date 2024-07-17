use crate::{
    async_actor::AsyncActorSender,
    document::{Document, DocumentId},
    lsp::Lsp,
    rope::Text,
};
use ignore::WalkBuilder;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{
    process::Command,
    sync::{mpsc::unbounded_channel, oneshot::Sender},
};
use virus_lsp::LspClients;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Debug)]
pub enum EventLoopMessage {}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub type EventLoopSender = Box<dyn Fn(EventLoopMessage) + Send>;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct Editor {
    root: PathBuf,
    document_id: DocumentId,
    documents: HashMap<DocumentId, Document>,
    active_document: Option<DocumentId>,
    clipboard: Text,
    pub(crate) lsps: LspClients,
    pub(crate) async_actor: AsyncActorSender,
    _event_loop: EventLoopSender,
}

impl Editor {
    pub fn new(
        root: PathBuf,
        (rust_lsp,): (Command,),
        async_actor: AsyncActorSender,
        event_loop: EventLoopSender,
    ) -> Self {
        let (rust_server_message_sender, rust_server_message_receiver) = unbounded_channel();
        let mut document_id = DocumentId::default();
        let active_document = Some(document_id.generate());

        let editor = Self {
            root,
            document_id,
            documents: Default::default(),
            active_document,
            clipboard: Text::default(),
            lsps: LspClients::new((rust_lsp, rust_server_message_sender)),
            async_actor,
            _event_loop: event_loop,
        };

        editor.lsp().init(rust_server_message_receiver);
        editor
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn get_document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    pub fn get_document_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(&id)
    }

    pub fn active_document(&self) -> Option<DocumentId> {
        self.active_document
    }

    pub fn get_active_document(&self) -> Option<&Document> {
        self.active_document.and_then(|id| self.get_document(id))
    }

    pub fn get_active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_document
            .and_then(|id| self.get_document_mut(id))
    }

    pub fn open(&mut self, path: PathBuf) -> std::io::Result<()> {
        if let Some((id, _)) = self
            .documents
            .iter()
            .find(|(_, document)| document.path() == path)
        {
            self.active_document = Some(*id);
        } else {
            let id = self.document_id.generate();

            let mut document = Document::open(id, path, self.lsp())?;
            document.parse();

            self.active_document = Some(id);
            self.documents.insert(id, document);
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

    pub fn copy(&mut self) {
        let document = self.get_active_document().unwrap();
        let range = document.selection().range();
        let slice = document
            .rope()
            .byte_slice(range.start.index()..range.end.index());

        self.clipboard = Text::from(slice);
    }

    pub fn paste(&mut self) {
        let edit = self.clipboard.clone();

        self.get_active_document_mut().unwrap().edit(edit);
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

    pub fn exit(&mut self, sender: Sender<()>) {
        self.lsp().exit(sender);
    }
}

/// Private.
impl Editor {
    fn lsp(&self) -> Lsp {
        Lsp::new(self.async_actor.clone())
    }
}
