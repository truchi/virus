pub mod pixels_mut;
pub mod text;

/// Re-exports.
pub mod reexports {
    pub mod swash {
        pub use swash::{
            scale::{image::Image, ScaleContext},
            shape::ShapeContext,
            text::cluster::SourceRange,
            CacheKey, FontRef, GlyphId,
        };
    }

    pub mod lru {
        pub use lru::LruCache;
    }
}

/// A RGB color.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const GREY: Self = Self::grey(127);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn grey(grey: u8) -> Self {
        Self::new(grey, grey, grey)
    }
}

/// A RGBA color.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
