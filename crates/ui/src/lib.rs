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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LineColumn                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}
