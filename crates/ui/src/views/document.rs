use crate::{syntax::Lines, theme::UiTheme, tween::Tweened, ui::LinesCache};
use std::{cell::RefCell, fmt::Write, rc::Weak, time::Duration};
use virus_editor::{
    document::Document,
    mode::{Mode, Select},
    rope::{Cursor, Selection},
};
use virus_graphics::{
    text::{Advance, Context, FontStyle, FontWeight, Line, Styles},
    types::{Position, Rectangle, Rgba},
    wgpu::{Draw, Layer},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          DocumentView                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentView {
    scroll_top: Tweened<u32>,
    scrollbar_alpha: Tweened<u8>,
    theme: Weak<RefCell<UiTheme>>,
    lines_cache: Weak<RefCell<LinesCache>>,
}

impl std::fmt::Debug for DocumentView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('…')
    }
}

impl DocumentView {
    pub fn new(theme: Weak<RefCell<UiTheme>>, lines_cache: Weak<RefCell<LinesCache>>) -> Self {
        Self {
            scroll_top: Default::default(),
            scrollbar_alpha: Default::default(),
            theme,
            lines_cache,
        }
    }

    pub fn scroll_top(&self) -> u32 {
        self.scroll_top.end()
    }

    pub fn is_animating(&self) -> bool {
        self.scroll_top.is_animating() || self.scrollbar_alpha.is_animating()
    }

    pub fn scroll_to(&mut self, top: u32) {
        let (duration, tween) = {
            let theme = *self.theme.upgrade().unwrap().borrow();
            (theme.scroll_duration, theme.scroll_tween)
        };

        self.scroll_top.to(top, duration, tween);
        self.scrollbar_alpha = Tweened::with_animation(255, 0, duration, tween);
    }

    pub fn update(&mut self, delta: Duration) {
        self.scroll_top.step(delta);
        self.scrollbar_alpha.step(delta);
    }

    pub fn render(
        &mut self,
        context: &mut Context,
        layer: &mut Layer,
        document: &Document,
        mode: Mode,
    ) {
        debug_assert!(match mode {
            Mode::Normal { select } | Mode::Insert { select } if select == Select::None =>
                document.selection().is_empty(),
            _ => true,
        });

        let theme = *self.theme.upgrade().unwrap().borrow();
        let scroll_top = self.scroll_top.current();
        let scrollbar_color = theme
            .scrollbar_color
            .transparent(self.scrollbar_alpha.current());

        let rope_lines = document.rope().len_lines();
        let region_height_in_lines = layer.size().height as f32 / theme.line_height as f32;
        let scroll_top_in_lines = scroll_top as f32 / theme.line_height as f32;
        let (start_line, end_line) = {
            let start = scroll_top_in_lines.floor() as usize;
            let end = scroll_top_in_lines.ceil() as usize + region_height_in_lines.ceil() as usize;

            let end = end.min(rope_lines);
            let start = start.min(end);

            (start, end)
        };
        let advance = context
            .fonts()
            .get((theme.family, FontWeight::Regular, FontStyle::Normal))
            .unwrap()
            .advance_for_size(theme.font_size);
        let scrollbar_rectangle = if rope_lines <= region_height_in_lines as usize {
            Rectangle::default()
        } else {
            let top = scroll_top_in_lines / rope_lines as f32;
            let height = region_height_in_lines / rope_lines as f32;
            let region_height = layer.size().height as f32;

            Rectangle {
                top: (top * region_height).round() as i32,
                height: (height * region_height).round() as u32,
                left: (advance / 2.0).round() as i32,
                width: (advance / 4.0).round() as u32,
            }
        };
        let lines_cache = self.lines_cache.upgrade().unwrap();
        let mut lines_cache = lines_cache.borrow_mut();
        let lines = lines_cache
            .entry(document.id())
            .or_insert_with(|| Lines::new(self.theme.clone()));
        let lines = lines.lines(context, document, start_line..end_line);

        Renderer {
            context,
            layer,
            theme,
            selection: document.selection(),
            lines: &lines[..],
            start_line,
            line_numbers_width: (advance * (rope_lines.ilog10() + 3) as Advance).round() as u32,
            scroll_top,
            scrollbar_rectangle,
            scrollbar_color,
            mode,
        }
        .render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'context, 'layer, 'graphics, 'lines> {
    context: &'context mut Context,
    layer: &'layer mut Layer<'graphics>,
    theme: UiTheme,
    selection: Selection,
    lines: &'lines [Line],
    start_line: usize,
    line_numbers_width: u32,
    scroll_top: u32,
    scrollbar_rectangle: Rectangle,
    scrollbar_color: Rgba,
    mode: Mode,
}

impl<'context, 'layer, 'graphics, 'lines> Renderer<'context, 'layer, 'graphics, 'lines> {
    fn render(&mut self) {
        self.render_line_numbers();
        self.render_lines();
        self.render_selection();
        self.render_scrollbar();
    }

    fn render_line_numbers(&mut self) {
        let styles = Styles {
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            foreground: self.theme.syntax.comment.foreground,
            background: Rgba::TRANSPARENT,
            underline: false,
            strike: false,
        };

        for number in self.start_line..self.start_line + self.lines.len() {
            let line = Line::shaper(&format!("{} ", number + 1), 0, styles).shape(
                self.context,
                self.theme.family,
                self.theme.font_size,
            );
            let top = number as i32 * self.theme.line_height as i32 - self.scroll_top as i32;
            let left = (self.line_numbers_width as Advance - line.advance()).round() as i32;

            self.layer.draw(None, 0).glyphs(
                self.context,
                Position { top, left },
                &line,
                self.theme.line_height,
            );
        }
    }

    fn render_lines(&mut self) {
        let left = self.line_numbers_width as i32;

        for (index, line) in self.lines.iter().enumerate() {
            let top = (self.start_line + index) as i32 * self.theme.line_height as i32
                - self.scroll_top as i32;

            self.layer.draw(None, 0).glyphs(
                self.context,
                Position { top, left },
                &line,
                self.theme.line_height as u32,
            );
        }
    }

    fn render_selection(&mut self) {
        let pos = |top, left| Position { top, left };
        let row = |cursor: Cursor| {
            cursor.line as i32 * self.theme.line_height as i32 - self.scroll_top as i32
        };
        let column = |cursor: Cursor| -> i32 {
            self.line_numbers_width as i32
                + if (self.start_line..self.start_line + self.lines.len()).contains(&cursor.line) {
                    let line = &self.lines[cursor.line - self.start_line];

                    line.glyphs()
                        .iter()
                        .find_map(|glyph| {
                            // TODO consecutive glyphs may have same range!
                            (glyph.range.end as usize > cursor.column).then_some(glyph.offset)
                        })
                        .unwrap_or_else(|| line.advance())
                        .round() as i32
                } else {
                    0
                }
        };

        let (selection, is_forward) = (self.selection.range(), self.selection.is_forward());
        let (width, height) = (
            self.layer.size().width as i32,
            self.theme.line_height as i32,
        );
        let top = row(selection.start);
        let bottom = row(selection.end);
        let start = column(selection.start);
        let end = column(selection.end);
        let draw = &mut self.layer.draw(None, 1);
        let caret_width = self.theme.caret_width;
        let color = match self.mode {
            Mode::Normal { .. } => self.theme.normal_mode_color,
            Mode::Insert { .. } => self.theme.insert_mode_color,
            Mode::Files => Default::default(),
        };
        let select = match self.mode {
            Mode::Normal { select } | Mode::Insert { select } => select,
            Mode::Files => Select::None,
        };
        let outline_colors = &[
            color.transparent(255 / 4),
            color.transparent(255 / 6),
            color.transparent(255 / 8),
            color.transparent(255 / 10),
        ];

        let render_outline = |draw: &mut Draw, top, bottom, left, right| {
            for (i, color) in outline_colors.iter().copied().enumerate() {
                let i = i as i32;

                if let Some(top) = top {
                    let i = i + 1; // TODO Why?!
                    draw.polyline([(pos(top + i, left), color), (pos(top + i, right), color)]);
                }

                if let Some(bottom) = bottom {
                    draw.polyline([
                        (pos(bottom - i, left), color),
                        (pos(bottom - i, right), color),
                    ]);
                }
            }
        };
        let render_selection = |draw: &mut Draw, top, left, width, height| {
            draw.rectangle(
                Rectangle {
                    top,
                    left,
                    width: width as u32,
                    height: height as u32,
                },
                color.transparent(255 / 4),
            );
        };
        let render_caret = |draw: &mut Draw, top, left| {
            draw.rectangle(
                Rectangle {
                    top,
                    left: left - caret_width as i32 / 2,
                    width: caret_width,
                    height: height as u32,
                },
                color.transparent(255),
            );
        };

        // Caret
        if selection.start == selection.end {
            let bottom = top + height;

            if select == Select::Lines {
                render_selection(draw, top, 0, width, height);
            } else {
                render_outline(draw, Some(top), Some(bottom), 0, width);
            }
            render_caret(draw, top, start);
        }
        // Single line
        else if selection.start.line == selection.end.line {
            let bottom = top + height;

            if select == Select::Lines {
                render_selection(draw, top, 0, width, height);
            } else {
                render_outline(draw, Some(top), Some(bottom), 0, start);
                render_outline(draw, Some(top), Some(bottom), end, width);
                render_selection(draw, top, start, end - start, height);
            }
            render_caret(draw, top, if is_forward { end } else { start });
        }
        // Multiple lines
        else {
            let (top2, bottom2) = (top + height, bottom + height);

            if select == Select::Lines {
                render_selection(draw, top, 0, width, bottom2 - top);
            } else {
                render_outline(draw, Some(top), None, 0, start);
                render_outline(draw, None, Some(bottom2), end, width);
                render_selection(draw, top, start, width - start, height);
                render_selection(draw, top2, 0, width, bottom - top2);
                render_selection(draw, bottom, 0, end, height);
            }
            render_caret(
                draw,
                if is_forward { bottom } else { top },
                if is_forward { end } else { start },
            );
        }
    }

    fn render_scrollbar(&mut self) {
        self.layer
            .draw(None, 0)
            .rectangle(self.scrollbar_rectangle, self.scrollbar_color);
    }
}
