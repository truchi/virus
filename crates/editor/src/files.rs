use crate::editor::Editor;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use virus_document::{add_in_range, sub_in_range};
use virus_fuzzy::{Config, Fuzzy, Match};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Files                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Files {
    selected: usize,
    fuzzy: Fuzzy,
}

impl Files {
    pub fn new() -> Self {
        Self {
            fuzzy: Fuzzy::new(Config::FILES),
            selected: 0,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          EditorFiles                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct EditorFiles<'editor> {
    editor: &'editor Editor,
}

impl<'editor> EditorFiles<'editor> {
    pub(crate) fn new(editor: &'editor Editor) -> Self {
        Self { editor }
    }

    pub fn selected_index(&self) -> usize {
        self.editor.files.selected
    }

    pub fn selected_file(&self) -> Option<PathBuf> {
        self.editor
            .files
            .fuzzy
            .matches()
            .get(self.editor.files.selected)
            .map(|m| self.editor.files.fuzzy.haystack()[m.index].as_str())
            .map(|path| self.editor.root.join(path))
    }

    pub fn needle(&self) -> &str {
        self.editor.files.fuzzy.needle()
    }

    pub fn haystack(&self) -> &[String] {
        self.editor.files.fuzzy.haystack()
    }

    pub fn matches(&self) -> &[Match] {
        self.editor.files.fuzzy.matches()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         EditorFilesMut                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct EditorFilesMut<'editor> {
    editor: &'editor mut Editor,
}

impl<'editor> EditorFilesMut<'editor> {
    pub(crate) fn new(editor: &'editor mut Editor) -> Self {
        Self { editor }
    }

    pub fn open(&mut self) {
        self.close();

        *self.editor.files.fuzzy.haystack_mut() = files(&self.editor.root, true, false).collect();
        self.editor.files.fuzzy.search();
    }

    pub fn close(&mut self) {
        self.editor.files.selected = 0;
        self.editor.files.fuzzy.clear();
    }

    pub fn update(&mut self, f: impl FnOnce(&mut String)) {
        f(self.editor.files.fuzzy.needle_mut());
        self.editor.files.selected = 0;
        self.editor.files.fuzzy.search();
    }

    pub fn top(&mut self) {
        self.editor.files.selected = 0;
    }

    pub fn bottom(&mut self) {
        self.editor.files.selected = self.editor.files.fuzzy.matches().len().saturating_sub(1);
    }

    pub fn up(&mut self, lines: usize, wrap: bool) {
        self.editor.files.selected = sub_in_range(
            self.editor.files.fuzzy.matches().len(),
            self.editor.files.selected,
            lines,
            wrap,
        );
    }

    pub fn down(&mut self, lines: usize, wrap: bool) {
        self.editor.files.selected = add_in_range(
            self.editor.files.fuzzy.matches().len(),
            self.editor.files.selected,
            lines,
            wrap,
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn files(root: &Path, hidden: bool, ignored: bool) -> impl '_ + Iterator<Item = String> {
    WalkBuilder::new(root)
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
        .build()
        .filter_map(Result::ok)
        .filter(|entry| matches!(entry.file_type(), Some(ty) if ty.is_file()))
        .filter_map(move |entry| {
            entry
                .into_path()
                .strip_prefix(root)
                .map(|path| path.to_owned())
                .ok()
        })
        .filter_map(|file| file.as_os_str().to_str().map(|file| file.to_owned()))
}
