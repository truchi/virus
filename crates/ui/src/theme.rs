use crate::{syntax::SyntaxTheme, tween::Tween};
use std::time::Duration;
use virus_graphics::{
    text::{Advance, FontFamilyKey, FontSize, LineHeight},
    types::{Rgb, Rgba, Size},
};

#[derive(Copy, Clone, Debug)]
pub struct UiTheme {
    pub syntax: SyntaxTheme,
    pub background_color: Rgb,
    pub inactive_foreground_color: Rgba,

    pub family: FontFamilyKey,
    pub font_size: FontSize,
    pub line_height: LineHeight,
    pub advance: Advance,

    pub scroll_duration: Duration,
    pub scroll_tween: Tween,
    pub scrollbar_color: Rgb,

    pub normal_mode_color: Rgb,
    pub insert_mode_color: Rgb,

    pub caret_width: u32,
}

impl UiTheme {
    pub fn cells_and_pixels(&self, size: Size) -> (Size, Size) {
        let cells = Size {
            width: (size.width as f32 / self.advance).floor() as u32,
            height: size.height / self.line_height,
        };
        let pixels = Size {
            width: (cells.width as f32 * self.advance).ceil() as u32,
            height: cells.height * self.line_height,
        };

        (cells, pixels)
    }
}
