use ropey::Rope;
use std::{borrow::Cow, ops::Range};
use virus_common::Cursor;
use virus_editor::{
    document::Document,
    highlights::{Highlight, Highlights},
    theme::Theme,
};
use virus_graphics::{
    colors::{Rgb, Rgba},
    text::{
        Advance, Context, FontFamilyKey, FontSize, FontStyle, FontWeight, Line, LineHeight, Styles,
    },
    wgpu::Draw,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                View                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentView {
    query: String,
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
        query: String,
        family: FontFamilyKey,
        theme: Theme,
        font_size: FontSize,
        line_height: LineHeight,
    ) -> Self {
        Self {
            query,
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
        draw: &mut Draw,
        document: &Document,
        scroll_top: u32,
        scrollbar_alpha: u8,
    ) {
        Renderer::new(self, context, draw, document, scroll_top, scrollbar_alpha).render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Renderer                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a, 'b, 'c, 'd, 'e> {
    view: &'a mut DocumentView,
    context: &'b mut Context,
    draw: &'c mut Draw<'d>,
    document: &'e Document,
    scroll_top: u32,
    scrollbar_alpha: u8,
    start: usize,
    end: usize,
    rope_lines: usize,
    advance: Advance,
    line_numbers_width: Advance,
}

impl<'a, 'b, 'c, 'd, 'e> Renderer<'a, 'b, 'c, 'd, 'e> {
    const CARETS: [(i32, Rgba); 3] = [
        (0, Rgb::WHITE.transparent(255)),
        (1, Rgb::WHITE.transparent(255 / 8)),
        (2, Rgb::WHITE.transparent(255 / 32)),
    ];
    const OUTLINES: [(i32, Rgba); 3] = [
        (0, Rgb::WHITE.transparent(255 / 16)),
        (1, Rgb::WHITE.transparent(255 / 32)),
        (2, Rgb::WHITE.transparent(255 / 64)),
    ];
    const SCROLLBAR: Rgb = Rgb::WHITE;

    fn new(
        view: &'a mut DocumentView,
        context: &'b mut Context,
        draw: &'c mut Draw<'d>,
        document: &'e Document,
        scroll_top: u32,
        scrollbar_alpha: u8,
    ) -> Self {
        let start = (scroll_top as f32 / view.line_height as f32).floor() as usize;
        let end = start + (draw.height() as f32 / view.line_height as f32).ceil() as usize;
        let rope_lines = document.rope().len_lines() - 1;
        let advance = context
            .fonts()
            .get((view.family, FontWeight::Regular, FontStyle::Normal))
            .unwrap()
            .advance_for_size(view.font_size);
        let line_numbers_width = advance * (rope_lines.ilog10() + 3) as Advance;

        Self {
            view,
            context,
            draw,
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
        let end = (self.end + margin).max(self.rope_lines + 1);

        self.view.lines.clear();
        self.view.range = start..end;
        self.view.rope = self.document.rope().clone();

        // Compute lines
        let highlights = Highlights::new(
            self.document.rope(),
            self.document.tree().unwrap().root_node(),
            start..end,
            &self.document.query(&self.view.query).unwrap(),
        );

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

        // Last line
        if let Some(line) = prev_line {
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

            self.draw.glyphs(
                self.context,
                [top, left],
                &line,
                self.view.line_height as u32,
            );
        }
    }

    fn render_document(&mut self) {
        let left = self.line_numbers_width.ceil() as i32;

        for (index, line) in &self.view.lines {
            let top = *index as i32 * self.view.line_height as i32 - self.scroll_top as i32;
            self.draw.glyphs(
                self.context,
                [top, left],
                &line,
                self.view.line_height as u32,
            );
        }
    }

    fn render_selection(&mut self, selection: Range<Cursor>) {
        let width = self.draw.width() as i32;
        let height = self.view.line_height as i32;

        let row = |cursor: Cursor| 1 + cursor.line as i32 * height - self.scroll_top as i32;
        let column = |cursor: Cursor| {
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
                            (glyph.range.end as usize > cursor.column).then_some(glyph.offset)
                        })
                        .unwrap_or_else(|| line.advance())
                        .round() as i32
                } else {
                    0
                })
        };

        let top = row(selection.start);
        let bottom = row(selection.end);
        let start = column(selection.start);
        let end = column(selection.end);

        // Caret
        if selection.start.index == selection.end.index {
            let bottom = top + height;

            self.render_outline(Some(top), Some(bottom), 0, width);

            for (i, color) in Self::CARETS {
                self.draw
                    .polyline([([top, start - i], color), ([bottom, start - i], color)]);
                self.draw
                    .polyline([([top, start + i], color), ([bottom, start + i], color)]);
            }
        }
        // Single line
        else if selection.start.line == selection.end.line {
            let bottom = top + height;

            self.render_outline(Some(top), Some(bottom), 0, start);
            self.render_outline(Some(top), Some(bottom), end, width);

            for (i, color) in Self::CARETS {
                self.draw.polyline([
                    ([top + i, start + i], color),
                    ([top + i, end - i], color),
                    ([bottom - i, end - i], color),
                    ([bottom - i, start + i], color),
                    ([top + i, start + i], color),
                ]);
            }
        }
        // Two non-overlapping lines
        else if selection.start.line + 1 == selection.end.line && start > end {
            let middle = top + height;
            let bottom = middle + height;

            self.render_outline(Some(top), None, 0, start);
            self.render_outline(None, Some(bottom), end, width);

            for (i, color) in Self::CARETS {
                self.draw.polyline([
                    ([top + i, width], color),
                    ([top + i, start + i], color),
                    ([middle - i, start + i], color),
                    ([middle - i, width], color),
                ]);
                self.draw.polyline([
                    ([middle + i, 0], color),
                    ([middle + i, end - i], color),
                    ([bottom - i, end - i], color),
                    ([bottom - i, 0], color),
                ]);
            }
        }
        // Two lines or more
        else {
            let (top1, top2, bottom1, bottom2) = (top, top + height, bottom, bottom + height);

            self.render_outline(Some(top1), None, 0, start);
            self.render_outline(None, Some(bottom2), end, width);

            for (i, color) in Self::CARETS {
                self.draw.polyline([
                    ([top2 + i, 0], color),
                    ([top2 + i, start + i], color),
                    ([top1 + i, start + i], color),
                    ([top1 + i, width], color),
                ]);
                self.draw.polyline([
                    ([bottom1 - i, width], color),
                    ([bottom1 - i, end - i], color),
                    ([bottom2 - i, end - i], color),
                    ([bottom2 - i, 0], color),
                ]);
            }
        }
    }

    fn render_outline(&mut self, top: Option<i32>, bottom: Option<i32>, left: i32, right: i32) {
        for (i, color) in Self::OUTLINES {
            if let Some(top) = top {
                self.draw
                    .polyline([([top + i, left], color), ([top + i, right], color)]);
            }

            if let Some(bottom) = bottom {
                self.draw
                    .polyline([([bottom - i, left], color), ([bottom - i, right], color)]);
            }
        }
    }

    fn render_scrollbar(&mut self) {
        let region_height_in_lines = self.draw.height() / self.view.line_height();

        if self.rope_lines <= region_height_in_lines as usize {
            return;
        }

        let scroll_top_in_lines = self.scroll_top as f32 / self.view.line_height() as f32;
        let top = scroll_top_in_lines / self.rope_lines as f32;
        let height = region_height_in_lines as f32 / self.rope_lines as f32;

        let top = (top * self.draw.height() as f32).round() as i32;
        let height = (height * self.draw.height() as f32).round() as u32;
        let left = (self.advance / 2.0).round() as i32;
        let width = (self.advance / 4.0).round() as u32;

        self.draw.rectangle(
            ([top, left], [width, height]),
            Self::SCROLLBAR.transparent(self.scrollbar_alpha),
        );
    }
}
