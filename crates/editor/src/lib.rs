pub mod async_actor;
pub mod cursor;
pub mod document;
pub mod editor;
pub mod fuzzy;
pub mod lsp;
pub mod rope {
    pub use cursor::chunk::*;
    pub use cursor::grapheme::*;
    pub use cursor::word::*;
    pub use extension::*;
    pub use text::*;

    mod cursor {
        pub mod chunk;
        pub mod grapheme;
        pub mod word;
    }
    mod extension;
    mod text;
}
pub mod syntax {
    pub use capture::*;
    pub use theme::*;

    mod capture;
    mod theme;
}
