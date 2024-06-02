use ropey::Rope;
use std::{borrow::Cow, ops::Range};
use virus_common::{Cursor, Position, Rectangle, Rgb, Rgba};
use virus_editor::{document::Document, highlights::Highlight, theme::Theme};
use virus_graphics::{
    text::{
        Advance, Context, FontFamilyKey, FontSize, FontStyle, FontWeight, Line, LineHeight, Styles,
    },
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
    range: Range<usize>,
    lines: Vec<(usize, Line)>,
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
            range: 0..0,
            lines: Vec::default(),
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
        document: &Document,
        scroll_top: u32,
        scrollbar_alpha: u8,
    ) {
        Renderer::new(self, context, layer, document, scroll_top, scrollbar_alpha).render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// Catppuccin latte
const SURFACE1: Rgb = Rgb {
    r: 188,
    g: 192,
    b: 204,
};

fn pos(top: i32, left: i32) -> Position {
    Position { top, left }
}

struct Renderer<'view, 'context, 'layer, 'graphics, 'document> {
    view: &'view mut DocumentView,
    context: &'context mut Context,
    layer: &'layer mut Layer<'graphics>,
    document: &'document Document,
    scroll_top: u32,
    scrollbar_alpha: u8,
    start: usize,
    end: usize,
    rope_lines: usize,
    advance: Advance,
    line_numbers_width: Advance,
}

impl<'view, 'context, 'layer, 'graphics, 'document>
    Renderer<'view, 'context, 'layer, 'graphics, 'document>
{
    const CARETS: [(i32, Rgba); 2] = [
        (0, SURFACE1.transparent(255 / 1)),
        (1, SURFACE1.transparent(255 / 1)),
    ];
    const OUTLINES: [(i32, Rgba); 4] = [
        (0, SURFACE1.transparent(255 / 4)),
        (1, SURFACE1.transparent(255 / 6)),
        (2, SURFACE1.transparent(255 / 8)),
        (3, SURFACE1.transparent(255 / 10)),
    ];
    const SCROLLBAR: Rgb = SURFACE1;

    fn new(
        view: &'view mut DocumentView,
        context: &'context mut Context,
        layer: &'layer mut Layer<'graphics>,
        document: &'document Document,
        scroll_top: u32,
        scrollbar_alpha: u8,
    ) -> Self {
        let start = (scroll_top as f32 / view.line_height as f32).floor() as usize;
        let end = start + (layer.size().height as f32 / view.line_height as f32).ceil() as usize;
        let rope_lines = document.rope().len_lines() - 1;
        let advance = context
            .fonts()
            .get((view.family, FontWeight::Regular, FontStyle::Normal))
            .unwrap()
            .advance_for_size(view.font_size);
        let line_numbers_width = advance
            * if rope_lines == 0 {
                1.0
            } else {
                (rope_lines.ilog10() + 3) as Advance
            };

        Self {
            view,
            context,
            layer,
            document,
            scroll_top,
            scrollbar_alpha,
            start,
            end,
            rope_lines,
            advance,
            line_numbers_width,
        }
    }

    fn render(&mut self) {
        self.highlights();
        self.render_line_numbers();
        self.render_document();
        self.render_selection(self.document.selection());
        self.render_scrollbar();
    }
}

impl<'view, 'context, 'draw, 'graphics, 'document>
    Renderer<'view, 'context, 'draw, 'graphics, 'document>
{
    fn row(&self, cursor: Cursor) -> i32 {
        1 + cursor.line as i32 * self.view.line_height as i32 - self.scroll_top as i32
    }

    fn column(&self, cursor: Cursor) -> i32 {
        1 + self.line_numbers_width.ceil() as i32
            + (if let Some(line) = self
                .view
                .lines
                .iter()
                .find_map(|(index, line)| (*index == cursor.line).then_some(line))
            {
                line.glyphs()
                    .iter()
                    .find_map(|glyph| {
                        // TODO: consecutive glyphs may have same range!
                        (glyph.range.end as usize > cursor.column).then_some(glyph.offset)
                    })
                    .unwrap_or_else(|| line.advance())
                    .round() as i32
            } else {
                0
            })
    }

    fn highlights(&mut self) {
        // No need to prepare highlights if same rope and similar range
        if (self.view.rope.is_instance(self.document.rope())
            || self.view.rope == *self.document.rope())
            && self.view.range.contains(&self.start)
            && self.view.range.contains(&(self.end - 1))
        {
            return;
        }

        // Apply margins: half that line range above and below
        let margin = (self.end - self.start) / 2;
        let start = self.start.saturating_sub(margin);
        let end = (self.end + margin).min(self.rope_lines + 1);

        self.view.lines.clear();
        self.view.range = start..end;
        self.view.rope = self.document.rope().clone();

        // Compute lines
        let highlights = self.document.highlights(start..end);
        let mut shaper = Line::shaper(self.context, self.view.font_size);
        let mut prev_line = None;

        for Highlight { start, end, key } in highlights.highlights() {
            debug_assert!(start.line == end.line);
            let line = start.line;

            // New line
            if prev_line != Some(line) {
                if let Some(line) = prev_line {
                    self.view.lines.push((line, shaper.line()));
                }

                shaper = Line::shaper(self.context, self.view.font_size);
                prev_line = Some(line);
            }

            shaper.push(
                // We cow to make sure ligatures are not split between rope chunks
                &Cow::from(
                    self.document
                        .rope()
                        .get_byte_slice(start.index..end.index)
                        .unwrap(),
                ),
                self.view.family,
                self.view.theme[key],
            );
        }

        if let Some(line) = prev_line {
            // Last line
            self.view.lines.push((line, shaper.line()));
        }
    }

    fn render_line_numbers(&mut self) {
        let (family, foreground) = (self.view.family, self.view.theme.comment.foreground);
        let styles = Styles {
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            foreground: foreground.solid().transparent(127),
            background: Rgba::TRANSPARENT,
            underline: false,
            strike: false,
        };

        for number in self.start..=self.end.min(self.rope_lines.saturating_sub(1)) {
            let line = {
                let mut shaper = Line::shaper(self.context, self.view.font_size);
                shaper.push(&(number + 1).to_string(), family, styles);
                shaper.line()
            };

            let top = number as i32 * self.view.line_height as i32 - self.scroll_top as i32;
            let left = (self.line_numbers_width - self.advance - line.advance()).round() as i32;

            self.layer.draw(0).glyphs(
                self.context,
                Position { top, left },
                &line,
                self.view.line_height,
            );
        }
    }

    fn render_document(&mut self) {
        let left = self.line_numbers_width.ceil() as i32;

        for (index, line) in self
            .view
            .lines
            .iter()
            .skip_while(|(index, _)| *index < self.start)
            .take_while(|(index, _)| *index <= self.end)
        {
            let top = *index as i32 * self.view.line_height as i32 - self.scroll_top as i32;
            self.layer.draw(0).glyphs(
                self.context,
                Position { top, left },
                &line,
                self.view.line_height as u32,
            );
        }
    }

    fn render_selection(&mut self, selection: Range<Cursor>) {
        fn render_outline(
            renderer: &mut Renderer,
            top: Option<i32>,
            bottom: Option<i32>,
            left: i32,
            right: i32,
        ) {
            for (i, color) in Renderer::OUTLINES {
                if let Some(top) = top {
                    renderer
                        .layer
                        .draw(0)
                        .polyline([(pos(top + i, left), color), (pos(top + i, right), color)]);
                }

                if let Some(bottom) = bottom {
                    renderer.layer.draw(0).polyline([
                        (pos(bottom - i, left), color),
                        (pos(bottom - i, right), color),
                    ]);
                }
            }
        }

        let width = self.layer.size().width as i32;
        let height = self.view.line_height as i32;
        let top = self.row(selection.start);
        let bottom = self.row(selection.end);
        let start = self.column(selection.start);
        let end = self.column(selection.end);

        // Caret
        if selection.start.index == selection.end.index {
            let bottom = top + height;

            render_outline(self, Some(top), Some(bottom), 0, width);

            for (i, color) in Self::CARETS {
                self.layer.draw(0).polyline([
                    (pos(top, start - i), color),
                    (pos(bottom, start - i), color),
                ]);
                self.layer.draw(0).polyline([
                    (pos(top, start + i), color),
                    (pos(bottom, start + i), color),
                ]);
            }
        }
        // Single line
        else if selection.start.line == selection.end.line {
            let bottom = top + height;

            render_outline(self, Some(top), Some(bottom), 0, start);
            render_outline(self, Some(top), Some(bottom), end, width);

            for (i, color) in Self::CARETS {
                self.layer.draw(0).polygon([
                    (pos(top + i, start + i), color),
                    (pos(top + i, end - i), color),
                    (pos(bottom - i, end - i), color),
                    (pos(bottom - i, start + i), color),
                ]);
            }
        }
        // Two non-overlapping lines
        else if selection.start.line + 1 == selection.end.line && start > end {
            let middle = top + height;
            let bottom = middle + height;

            render_outline(self, Some(top), None, 0, start);
            render_outline(self, None, Some(bottom), end, width);

            for (i, color) in Self::CARETS {
                self.layer.draw(0).polyline([
                    (pos(top + i, width), color),
                    (pos(top + i, start + i), color),
                    (pos(middle - i, start + i), color),
                    (pos(middle - i, width), color),
                ]);
                self.layer.draw(0).polyline([
                    (pos(middle + i, 0), color),
                    (pos(middle + i, end - i), color),
                    (pos(bottom - i, end - i), color),
                    (pos(bottom - i, 0), color),
                ]);
            }
        }
        // Two lines or more
        else {
            let (top1, top2, bottom1, bottom2) = (top, top + height, bottom, bottom + height);

            render_outline(self, Some(top1), None, 0, start);
            render_outline(self, None, Some(bottom2), end, width);

            for (i, color) in Self::CARETS {
                self.layer.draw(0).polyline([
                    (pos(top2 + i, 0), color),
                    (pos(top2 + i, start + i), color),
                    (pos(top1 + i, start + i), color),
                    (pos(top1 + i, width), color),
                ]);
                self.layer.draw(0).polyline([
                    (pos(bottom1 - i, width), color),
                    (pos(bottom1 - i, end - i), color),
                    (pos(bottom2 - i, end - i), color),
                    (pos(bottom2 - i, 0), color),
                ]);
            }
        }
    }

    fn render_scrollbar(&mut self) {
        let region_height_in_lines = self.layer.size().height / self.view.line_height();

        if self.rope_lines <= region_height_in_lines as usize {
            return;
        }

        let scroll_top_in_lines = self.scroll_top as f32 / self.view.line_height() as f32;
        let top = scroll_top_in_lines / self.rope_lines as f32;
        let height = region_height_in_lines as f32 / self.rope_lines as f32;
        let rectangle = Rectangle {
            top: (top * self.layer.size().height as f32).round() as i32,
            height: (height * self.layer.size().height as f32).round() as u32,
            left: (self.advance / 2.0).round() as i32,
            width: (self.advance / 4.0).round() as u32,
        };

        self.layer
            .draw(0)
            .rectangle(rectangle, Self::SCROLLBAR.transparent(self.scrollbar_alpha));
    }
}
