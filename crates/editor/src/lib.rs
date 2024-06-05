pub mod document;
pub mod rope {
    pub use cursor::chunk::*;
    pub use cursor::grapheme::*;
    pub use cursor::word::*;
    pub use extension::*;

    mod cursor {
        pub mod chunk;
        pub mod grapheme;
        pub mod word;
    }
    mod extension;
}
pub mod syntax {
    pub use highlight::*;
    pub use theme::*;

    mod highlight;
    mod theme;
}
