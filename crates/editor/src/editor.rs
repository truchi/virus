use ignore::WalkBuilder;
use notify::{
    event::ModifyKind, recommended_watcher, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use virus_document::document::{Document, DocumentId, DocumentIds};

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub type WatcherEvent = notify::Event;

pub trait EventLoopProxy: 'static + Send {
    fn redraw(&self);

    fn watcher(&self, event: WatcherEvent);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Editor                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Editor {
    root: PathBuf,
    document_ids: DocumentIds,
    documents: HashMap<DocumentId, Document>,
    watcher: RecommendedWatcher,
    event_loop_proxy: Box<dyn EventLoopProxy>,
}

impl Editor {
    pub fn new(root: PathBuf, event_loop_proxy: impl EventLoopProxy + Clone) -> Self {
        Self {
            root,
            document_ids: Default::default(),
            documents: Default::default(),
            watcher: recommended_watcher({
                let event_loop_proxy = event_loop_proxy.clone();

                move |event| match event {
                    Ok(event) => event_loop_proxy.watcher(event),
                    Err(err) => debug_assert!(false, "{err:#?}"),
                }
            })
            .expect("recommended watcher"),
            event_loop_proxy: Box::new(event_loop_proxy),
        }
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn get_document(&self, document_id: DocumentId) -> Option<&Document> {
        self.documents.get(&document_id)
    }

    pub fn get_document_mut(&mut self, document_id: DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(&document_id)
    }

    pub fn open(&mut self, path: PathBuf) -> std::io::Result<DocumentId> {
        if let Some((document_id, _)) = self
            .documents
            .iter()
            .find(|(_, document)| document.path() == path)
        {
            Ok(*document_id)
        } else {
            let document_id = self.document_ids.id();

            let mut document = Document::open(document_id, path)?;
            document.parse();

            self.watcher
                .watch(document.path(), RecursiveMode::NonRecursive)
                .expect("watch");
            self.documents.insert(document_id, document);

            Ok(document_id)
        }
    }

    pub fn close(&mut self, document_id: DocumentId) {
        let Some(document) = self.documents.remove(&document_id) else {
            return;
        };

        self.watcher.unwatch(document.path()).expect("unwatch");
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

    pub fn handle_watcher_event(&mut self, event: WatcherEvent) {
        if !matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
            return;
        }

        for path in event.paths {
            if let Some(document) = self
                .documents
                .values_mut()
                .find(|document| document.path() == path)
            {
                if document.reload().unwrap() {
                    document.parse();
                    self.event_loop_proxy.redraw();
                }
            }
        }
    }

    pub fn find_git_root(path: &Path) -> Option<PathBuf> {
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
