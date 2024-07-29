pub mod async_actor;
pub mod document;
pub mod editor;
pub mod fuzzy;
pub mod lsp;
pub mod rope {
    pub use cursor::*;
    pub use cursors::chunk::*;
    pub use cursors::grapheme::*;
    pub use cursors::word::*;
    pub use edit::*;
    pub use text::*;

    mod cursor;
    mod cursors {
        pub mod chunk;
        pub mod grapheme;
        pub mod word;
    }
    mod edit;
    mod text;
}
pub mod syntax {
    pub use capture::*;
    pub use theme::*;

    mod capture;
    mod theme;
}
pub mod history;
