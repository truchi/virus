use crate::{syntax::SyntaxTheme, tween::Tween};
use std::time::Duration;
use virus_graphics::{
    text::{Context, FontFamilyKey, FontSize, LineHeight},
    types::{Rgb, Rgba},
};

pub struct UiTheme<T = FontFamilyKey> {
    pub syntax: SyntaxTheme,
    pub family: T,
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

impl UiTheme<&str> {
    pub fn resolve(self, context: &Context) -> UiTheme {
        UiTheme {
            syntax: self.syntax,
            family: context.fonts().get(self.family).unwrap().key(),
            font_size: self.font_size,
            line_height: self.line_height,
            scrollbar_color: self.scrollbar_color,
            scroll_duration: self.scroll_duration,
            scroll_tween: self.scroll_tween,
            outline_normal_mode_colors: self.outline_normal_mode_colors,
            outline_select_mode_colors: self.outline_select_mode_colors,
            outline_insert_mode_colors: self.outline_insert_mode_colors,
            caret_normal_mode_color: self.caret_normal_mode_color,
            caret_select_mode_color: self.caret_select_mode_color,
            caret_insert_mode_color: self.caret_insert_mode_color,
            caret_normal_mode_width: self.caret_normal_mode_width,
            caret_select_mode_width: self.caret_select_mode_width,
            caret_insert_mode_width: self.caret_insert_mode_width,
            selection_select_mode_color: self.selection_select_mode_color,
            selection_insert_mode_color: self.selection_insert_mode_color,
        }
    }
}
