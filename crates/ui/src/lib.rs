pub mod theme;
pub mod tween;
pub mod ui;
pub mod views {
    mod document;
    mod files;

    pub use document::*;
    pub use files::*;
}

// For convenience.
pub use virus_graphics::Catppuccin;
