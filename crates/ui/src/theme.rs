use crate::{syntax::SyntaxTheme, tween::Tween};
use std::time::Duration;
use virus_graphics::{
    text::{Advance, FontFamilyKey, FontSize, LineHeight},
    types::{Rgb, Rgba, Size},
};

#[derive(Copy, Clone, PartialEq, Debug)]
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
    /// Returns the size in cells according to advance and line height.
    pub fn cells(&self, size: Size) -> Size {
        Size {
            width: (size.width as f32 / self.advance).floor() as u32,
            height: size.height / self.line_height,
        }
    }

    /// Returns the cells size in pixels according to advance and line height.
    pub fn pixels(&self, size: Size) -> Size {
        let cells = self.cells(size);
        Size {
            width: (cells.width as f32 * self.advance).ceil() as u32,
            height: cells.height * self.line_height,
        }
    }
}
