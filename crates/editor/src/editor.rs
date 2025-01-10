use crate::{
    files::{EditorFiles, EditorFilesMut, Files},
    mode::{Mode, Select},
};
use notify::{
    event::ModifyKind, recommended_watcher, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use virus_document::{
    document::{Document, DocumentId, DocumentIds},
    edit::Text,
};

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub type WatcherEvent = notify::Event;

pub trait EventLoopProxy: 'static + Send {
    fn redraw(&self);

    fn watcher(&self, event: WatcherEvent);
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Clone, Debug)]
pub struct Clipboard {
    pub select: Select,
    pub text: Text,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Editor                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Editor {
    pub(crate) root: PathBuf,
    pub(crate) document_ids: DocumentIds,
    pub(crate) documents: HashMap<DocumentId, Document>,
    pub(crate) mode: Mode,
    pub(crate) files: Files,
    pub(crate) clipboard: Option<Clipboard>,
    pub(crate) watcher: RecommendedWatcher,
    pub(crate) event_loop_proxy: Box<dyn EventLoopProxy>,
}

impl Editor {
    pub fn new(root: PathBuf, event_loop_proxy: impl EventLoopProxy + Clone) -> Self {
        Self {
            root,
            document_ids: Default::default(),
            documents: Default::default(),
            mode: Default::default(),
            files: Files::new(),
            clipboard: Default::default(),
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

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn mode_mut(&mut self) -> &mut Mode {
        &mut self.mode
    }

    pub fn clipboard(&self) -> &Option<Clipboard> {
        &self.clipboard
    }

    pub fn clipboard_mut(&mut self) -> &mut Option<Clipboard> {
        &mut self.clipboard
    }

    pub fn files(&self) -> EditorFiles {
        EditorFiles::new(self)
    }

    pub fn files_mut(&mut self) -> EditorFilesMut {
        EditorFilesMut::new(self)
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
