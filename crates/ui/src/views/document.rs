use crate::LineColumn;
use ropey::Rope;
use virus_editor::{document::Document, syntax::Theme};
use virus_graphics::{
    text::{
        Advance, Context, FontFamilyKey, FontSize, FontStyle, FontWeight, Line, LineHeight, Styles,
    },
    types::{Position, Rectangle, Rgba},
    wgpu::Layer,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          DocumentView                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentView {
    family: FontFamilyKey,
    theme: Theme,
    font_size: FontSize,
    line_height: LineHeight,
    rope: Rope,
}

impl DocumentView {
    pub fn new(
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
        line_height: LineHeight,
    ) -> Self {
        Self {
            family,
            theme,
            font_size,
            line_height,
            rope: Default::default(),
        }
    }

    pub fn family(&self) -> FontFamilyKey {
        self.family
    }

    pub fn font_size(&self) -> FontSize {
        self.font_size
    }

    pub fn line_height(&self) -> LineHeight {
        self.line_height
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn render(
        &mut self,
        context: &mut Context,
        layer: &mut Layer,
        document: &mut Document,
        scroll_top: u32,
        show_selection_as_lines: bool,
        scrollbar_color: Rgba,
        outline_colors: &[Rgba],
        caret_color: Rgba,
        caret_width: u32,
        selection_color: Rgba,
    ) {
        // NOTE: I'd like this the be done outside this file (or even better outside this crate)

        self.rope = document.rope().clone();

        let rope_lines = document.rope().len_lines();
        let region_height_in_lines = layer.size().height as f32 / self.line_height as f32;
        let scroll_top_in_lines = scroll_top as f32 / self.line_height as f32;

        let (start_line, end_line) = {
            let start = scroll_top_in_lines.floor() as usize;
            let end = scroll_top_in_lines.ceil() as usize + region_height_in_lines.ceil() as usize;

            let end = end.min(rope_lines);
            let start = start.min(end);

            (start, end)
        };

        let advance = context
            .fonts()
            .get((self.family, FontWeight::Regular, FontStyle::Normal))
            .unwrap()
            .advance_for_size(self.font_size);

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

        let anchor = LineColumn {
            line: document.selection().anchor.line,
            column: document.selection().anchor.column,
        };
        let head = LineColumn {
            line: document.selection().head.line,
            column: document.selection().head.column,
        };

        let lines = document.shape(
            context,
            start_line..end_line,
            self.family,
            self.theme,
            self.font_size,
        );

        Renderer {
            context,
            layer,
            family: self.family,
            font_size: self.font_size,
            line_height: self.line_height,
            anchor,
            head,
            start_line,
            line_numbers_width: (advance * (rope_lines.ilog10() + 3) as Advance).round() as u32,
            line_numbers_color: self.theme.comment.foreground,
            lines: &lines[..],
            scroll_top,
            show_selection_as_lines,
            scrollbar_rectangle,
            scrollbar_color,
            outline_colors,
            caret_color,
            caret_width,
            selection_color,
        }
        .render()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'context, 'layer, 'graphics, 'lines, 'outline_colors> {
    context: &'context mut Context,
    layer: &'layer mut Layer<'graphics>,
    family: FontFamilyKey,
    font_size: FontSize,
    line_height: LineHeight,
    anchor: LineColumn,
    head: LineColumn,
    lines: &'lines [Line],
    start_line: usize,
    line_numbers_width: u32,
    line_numbers_color: Rgba,
    scroll_top: u32,
    show_selection_as_lines: bool,
    scrollbar_rectangle: Rectangle,
    scrollbar_color: Rgba,
    outline_colors: &'outline_colors [Rgba],
    caret_color: Rgba,
    caret_width: u32,
    selection_color: Rgba,
}

impl<'context, 'layer, 'graphics, 'lines, 'outline_colors>
    Renderer<'context, 'layer, 'graphics, 'lines, 'outline_colors>
{
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
            foreground: self.line_numbers_color,
            background: Rgba::TRANSPARENT,
            underline: false,
            strike: false,
        };

        for number in self.start_line..self.start_line + self.lines.len() {
            let line = Line::shaper(&format!("{} ", number + 1), 0, styles).shape(
                self.context,
                self.family,
                self.font_size,
                None,
                None,
            );
            let top = number as i32 * self.line_height as i32 - self.scroll_top as i32;
            let left = (self.line_numbers_width as Advance - line.advance()).round() as i32;

            self.layer.draw(None, 0).glyphs(
                self.context,
                Position { top, left },
                &line,
                self.line_height,
            );
        }
    }

    fn render_lines(&mut self) {
        let left = self.line_numbers_width as i32;

        for (index, line) in self.lines.iter().enumerate() {
            let top =
                (self.start_line + index) as i32 * self.line_height as i32 - self.scroll_top as i32;

            self.layer.draw(None, 0).glyphs(
                self.context,
                Position { top, left },
                &line,
                self.line_height as u32,
            );
        }
    }

    fn render_selection(&mut self) {
        let pos = |top, left| Position { top, left };
        let row = |LineColumn { line, .. }| {
            line as i32 * self.line_height as i32 - self.scroll_top as i32
        };
        let column = |LineColumn { line, column }| -> i32 {
            self.line_numbers_width as i32
                + if (self.start_line..self.start_line + self.lines.len()).contains(&line) {
                    let line = &self.lines[line - self.start_line];

                    line.glyphs()
                        .iter()
                        .find_map(|glyph| {
                            // TODO: consecutive glyphs may have same range!
                            (glyph.range.end as usize > column).then_some(glyph.offset)
                        })
                        .unwrap_or_else(|| line.advance())
                        .round() as i32
                } else {
                    0
                }
        };

        let layer = 1;
        let (selection, is_forward) = if self.anchor <= self.head {
            (self.anchor..self.head, true)
        } else {
            (self.head..self.anchor, false)
        };
        let (width, height) = (self.layer.size().width as i32, self.line_height as i32);
        let top = row(selection.start);
        let bottom = row(selection.end);
        let start = column(selection.start);
        let end = column(selection.end);

        let render_outline = |renderer: &mut Renderer, top, bottom, left, right| {
            for (i, color) in renderer.outline_colors.iter().copied().enumerate() {
                let i = i as i32;

                if let Some(top) = top {
                    let i = i + 1; // TODO Why?!
                    renderer
                        .layer
                        .draw(None, layer)
                        .polyline([(pos(top + i, left), color), (pos(top + i, right), color)]);
                }

                if let Some(bottom) = bottom {
                    renderer.layer.draw(None, layer).polyline([
                        (pos(bottom - i, left), color),
                        (pos(bottom - i, right), color),
                    ]);
                }
            }
        };
        let render_selection = |renderer: &mut Renderer, top, left, width, height| {
            renderer.layer.draw(None, layer).rectangle(
                Rectangle {
                    top,
                    left,
                    width: width as u32,
                    height: height as u32,
                },
                renderer.selection_color,
            );
        };
        let render_caret = |renderer: &mut Renderer, top, left| {
            renderer.layer.draw(None, layer).rectangle(
                Rectangle {
                    top,
                    left: left - renderer.caret_width as i32 / 2,
                    width: renderer.caret_width,
                    height: height as u32,
                },
                renderer.caret_color,
            );
        };

        // Caret
        if selection.start == selection.end {
            let bottom = top + height;

            if self.show_selection_as_lines {
                render_selection(self, top, 0, width, height);
            } else {
                render_outline(self, Some(top), Some(bottom), 0, width);
            }
            render_caret(self, top, start);
        }
        // Single line
        else if selection.start.line == selection.end.line {
            let bottom = top + height;

            if self.show_selection_as_lines {
                render_selection(self, top, 0, width, height);
            } else {
                render_outline(self, Some(top), Some(bottom), 0, start);
                render_outline(self, Some(top), Some(bottom), end, width);
                render_selection(self, top, start, end - start, height);
            }
            render_caret(self, top, if is_forward { end } else { start });
        }
        // Multiple lines
        else {
            let (top2, bottom2) = (top + height, bottom + height);

            if self.show_selection_as_lines {
                render_selection(self, top, 0, width, bottom2 - top);
            } else {
                render_outline(self, Some(top), None, 0, start);
                render_outline(self, None, Some(bottom2), end, width);
                render_selection(self, top, start, width - start, height);
                render_selection(self, top2, 0, width, bottom - top2);
                render_selection(self, bottom, 0, end, height);
            }
            render_caret(
                self,
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
