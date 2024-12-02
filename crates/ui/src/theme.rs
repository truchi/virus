use crate::{syntax::SyntaxTheme, tween::Tween};
use std::time::Duration;
use virus_graphics::{
    text::{FontFamilyKey, FontSize, LineHeight},
    types::Rgb,
};

#[derive(Clone, Debug)]
pub struct UiTheme {
    pub syntax: SyntaxTheme,

    pub family: FontFamilyKey,
    pub font_size: FontSize,
    pub line_height: LineHeight,

    pub scroll_duration: Duration,
    pub scroll_tween: Tween,
    pub scrollbar_color: Rgb,

    pub normal_mode_color: Rgb,
    pub insert_mode_color: Rgb,

    pub caret_width: u32,
}
