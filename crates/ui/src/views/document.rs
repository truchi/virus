use crate::{
    syntax::Highlighted,
    theme::UiTheme,
    tween::{Tween, Tweened},
    Context,
};
use std::{fmt::Write, time::Duration};
use swash::{scale::ScaleContext, shape::ShapeContext};
use virus_editor::{
    document::{Document, DocumentId},
    mode::{Mode, Select},
    rope::{Cursor, Selection},
};
use virus_graphics::{
    text::{Advance, FontStyle, FontWeight, Fonts, Glyphs, Styles},
    types::{Position, Rectangle, Rgba, Size},
    wgpu::{Draw, Layer},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          DocumentView                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentView {
    document_id: DocumentId,
    size: Size,
    scroll_top: Tweened<u32>,
    scrollbar_alpha: Tweened<u8>,
}

impl std::fmt::Debug for DocumentView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('…')
    }
}

impl DocumentView {
    pub const GUTTER_COLUMNS: u32 = 5;

    pub fn new(document_id: DocumentId) -> Self {
        Self {
            document_id,
            size: Default::default(),
            scroll_top: Default::default(),
            scrollbar_alpha: Default::default(),
        }
    }

    pub fn is_animating(&self) -> bool {
        self.scroll_top.is_animating() || self.scrollbar_alpha.is_animating()
    }

    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn scroll_top(&self) -> Tweened<u32> {
        self.scroll_top
    }

    pub fn scroll(&mut self, top: u32, tween: Tween, duration: Duration) {
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
        layer: Layer,
        document: &Document,
        mode: Mode,
        is_active: bool,
    ) {
        debug_assert!(match mode {
            Mode::Normal { select } | Mode::Insert { select } if select == Select::None =>
                document.selection().is_empty(),
            _ => true,
        });

        self.size = layer.size();

        let context = context.as_mut();
        let theme = *context.theme;
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
            .fonts
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
        let highlighted = context
            .highlighteds
            .entry(document.id())
            .or_insert_with(|| Highlighted::new(theme, document.id()));
        let highlighted = highlighted.get(
            context.fonts,
            context.shape,
            theme,
            document,
            start_line..end_line,
        );

        Renderer {
            fonts: context.fonts,
            shape: context.shape,
            scale: context.scale,
            theme: context.theme,
            layer,
            selection: document.selection(),
            highlighted,
            start_line,
            gutter_width: (advance * Self::GUTTER_COLUMNS as Advance).round() as u32,
            scroll_top,
            scrollbar_rectangle,
            scrollbar_color,
            mode,
            is_active,
        }
        .scrollbar()
        .gutter()
        .text()
        .selection()
        .foreground();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a> {
    fonts: &'a Fonts,
    shape: &'a mut ShapeContext,
    scale: &'a mut ScaleContext,
    theme: &'a UiTheme,
    layer: Layer<'a>,
    selection: Selection,
    highlighted: &'a [Glyphs],
    start_line: usize,
    gutter_width: u32,
    scroll_top: u32,
    scrollbar_rectangle: Rectangle,
    scrollbar_color: Rgba,
    mode: Mode,
    is_active: bool,
}

impl<'a> Renderer<'a> {
    fn scrollbar(&mut self) -> &mut Self {
        self.layer
            .draw(None, 0)
            .rectangle(self.scrollbar_rectangle, self.scrollbar_color);

        self
    }

    fn gutter(&mut self) -> &mut Self {
        let styles = Styles {
            foreground: self.theme.syntax.comment.foreground,
            ..Default::default()
        };

        for number in self.start_line..self.start_line + self.highlighted.len() {
            let glyphs = Glyphs::shaper(
                self.fonts,
                self.shape,
                self.theme.family,
                self.theme.font_size,
            )
            .push(&format!("{} ", number + 1), styles)
            .glyphs();

            self.layer.draw(None, 0).glyphs(
                self.fonts,
                self.scale,
                Position {
                    top: number as i32 * self.theme.line_height as i32 - self.scroll_top as i32,
                    left: (self.gutter_width as Advance - glyphs.advance()).round() as i32,
                },
                self.theme.line_height,
                &glyphs,
            );
        }

        self
    }

    fn text(&mut self) -> &mut Self {
        let left = self.gutter_width as i32;

        for (index, glyphs) in self.highlighted.iter().enumerate() {
            let top = (self.start_line + index) as i32 * self.theme.line_height as i32
                - self.scroll_top as i32;

            self.layer.draw(None, 0).glyphs(
                self.fonts,
                self.scale,
                Position { top, left },
                self.theme.line_height as u32,
                glyphs,
            );
        }

        self
    }

    fn selection(&mut self) -> &mut Self {
        let pos = |top, left| Position { top, left };
        let row = |cursor: Cursor| {
            cursor.line as i32 * self.theme.line_height as i32 - self.scroll_top as i32
        };
        let column = |cursor: Cursor| -> i32 {
            self.gutter_width as i32
                + if (self.start_line..self.start_line + self.highlighted.len())
                    .contains(&cursor.line)
                {
                    let glyphs = &self.highlighted[cursor.line - self.start_line];

                    glyphs
                        .glyphs()
                        .iter()
                        .find_map(|glyph| {
                            // TODO consecutive glyphs may have same range!
                            (glyph.end as usize > cursor.column).then_some(glyph.offset)
                        })
                        .unwrap_or_else(|| glyphs.advance())
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

        self
    }

    fn foreground(&mut self) -> &mut Self {
        if !self.is_active {
            let size = self.layer.size();
            self.layer.draw(None, 2).rectangle(
                Rectangle::from((Position::default(), size)),
                self.theme.inactive_foreground_color,
            );
        }

        self
    }
}
