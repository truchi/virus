use crate::tween::Tween;
use std::time::Duration;
use virus_editor::syntax::Theme as SyntaxTheme;
use virus_graphics::{
    text::{FontSize, LineHeight},
    types::{Rgb, Rgba},
};

pub struct Theme {
    pub syntax: SyntaxTheme,
    pub font_size: FontSize,
    pub line_height: LineHeight,
    pub scrollbar_color: Rgb,
    pub scroll_duration: Duration,
    pub scroll_tween: Tween,
    pub outline_normal_mode_colors: Vec<Rgba>,
    pub outline_select_mode_colors: Vec<Rgba>,
    pub outline_insert_mode_colors: Vec<Rgba>,
    pub caret_normal_mode_color: Rgba,
    pub caret_select_mode_color: Rgba,
    pub caret_insert_mode_color: Rgba,
    pub caret_normal_mode_width: u32,
    pub caret_select_mode_width: u32,
    pub caret_insert_mode_width: u32,
    pub selection_select_mode_color: Rgba,
    pub selection_insert_mode_color: Rgba,
}
