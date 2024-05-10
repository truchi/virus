use ropey::Rope;
use std::{borrow::Cow, ops::Range, time::Duration};
use virus_common::{Cursor, Position, Rectangle, Rgb, Rgba};
use virus_editor::{
    document::{Document, Selection},
    highlights::{Highlight, Highlights},
    theme::Theme,
};
use virus_graphics::{
    text::{
        Advance, Context, FontFamilyKey, FontSize, FontStyle, FontWeight, Line, LineHeight, Styles,
    },
    wgpu::new::Draw,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          DocumentView                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct DocumentView {
    query: String,
    family: FontFamilyKey,
    theme: Theme,
    font_size: FontSize,
    line_height: LineHeight,
    rope: Rope,
    is_animating: bool,
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
            is_animating: false,
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

    pub fn is_animating(&self) -> bool {
        self.is_animating
    }

    pub fn render(
        &mut self,
        context: &mut Context,
        draw: &mut Draw,
        document: &Document,
        scroll_top: u32,
        scrollbar_alpha: u8,
        time: Duration,
    ) {
        Renderer::new(
            self,
            context,
            draw,
            document,
            scroll_top,
            scrollbar_alpha,
            time,
        )
        .render();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'a, 'b, 'c, 'd, 'e> {
    view: &'a mut DocumentView,
    context: &'b mut Context,
    draw: &'c mut Draw<'d>,
    document: &'e Document,
    scroll_top: u32,
    scrollbar_alpha: u8,
    time: Duration,
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
        time: Duration,
    ) -> Self {
        let start = (scroll_top as f32 / view.line_height as f32).floor() as usize;
        let end = start + (draw.region().height as f32 / view.line_height as f32).ceil() as usize;
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
            time,
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
        self.render_selection(self.document.selection().to_range());
        self.render_ast_helpers();
        self.render_scrollbar();
    }
}

impl<'a, 'b, 'c, 'd, 'e> Renderer<'a, 'b, 'c, 'd, 'e> {
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
                Position { top, left },
                &line,
                self.view.line_height,
                // self.time, // TODO
            );
        }
    }

    fn render_document(&mut self) {
        let left = self.line_numbers_width.ceil() as i32;
        let mut is_animating = false;

        for (index, line) in self
            .view
            .lines
            .iter()
            .skip_while(|(index, _)| *index < self.start)
            .take_while(|(index, _)| *index <= self.end)
        {
            let top = *index as i32 * self.view.line_height as i32 - self.scroll_top as i32;
            self.draw.glyphs(
                self.context,
                Position { top, left },
                &line,
                self.view.line_height as u32,
                // self.time, // TODO
            );

            if line.has_animated_glyphs() {
                is_animating = true;
            }
        }

        self.view.is_animating = is_animating;
    }

    fn render_selection(&mut self, selection: Range<Cursor>) {
        fn render_outline(
            renderer: &mut Renderer,
            top: Option<i32>,
            bottom: Option<i32>,
            left: i32,
            right: i32,
        ) {
            // TODO
            // for (i, color) in Renderer::OUTLINES {
            //     if let Some(top) = top {
            //         renderer
            //             .draw
            //             .polyline([([top + i, left], color), ([top + i, right], color)]);
            //     }

            //     if let Some(bottom) = bottom {
            //         renderer
            //             .draw
            //             .polyline([([bottom - i, left], color), ([bottom - i, right], color)]);
            //     }
            // }
        }

        let width = self.draw.region().width as i32;
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
                // TODO
                // self.draw
                //     .polyline([([top, start - i], color), ([bottom, start - i], color)]);
                // self.draw
                //     .polyline([([top, start + i], color), ([bottom, start + i], color)]);
            }
        }
        // Single line
        else if selection.start.line == selection.end.line {
            let bottom = top + height;

            render_outline(self, Some(top), Some(bottom), 0, start);
            render_outline(self, Some(top), Some(bottom), end, width);

            for (i, color) in Self::CARETS {
                // TODO
                // self.draw.polyline([
                //     ([top + i, start + i], color),
                //     ([top + i, end - i], color),
                //     ([bottom - i, end - i], color),
                //     ([bottom - i, start + i], color),
                //     ([top + i, start + i], color),
                // ]);
            }
        }
        // Two non-overlapping lines
        else if selection.start.line + 1 == selection.end.line && start > end {
            let middle = top + height;
            let bottom = middle + height;

            render_outline(self, Some(top), None, 0, start);
            render_outline(self, None, Some(bottom), end, width);

            for (i, color) in Self::CARETS {
                // TODO
                // self.draw.polyline([
                //     ([top + i, width], color),
                //     ([top + i, start + i], color),
                //     ([middle - i, start + i], color),
                //     ([middle - i, width], color),
                // ]);
                // self.draw.polyline([
                //     ([middle + i, 0], color),
                //     ([middle + i, end - i], color),
                //     ([bottom - i, end - i], color),
                //     ([bottom - i, 0], color),
                // ]);
            }
        }
        // Two lines or more
        else {
            let (top1, top2, bottom1, bottom2) = (top, top + height, bottom, bottom + height);

            render_outline(self, Some(top1), None, 0, start);
            render_outline(self, None, Some(bottom2), end, width);

            for (i, color) in Self::CARETS {
                // TODO
                // self.draw.polyline([
                //     ([top2 + i, 0], color),
                //     ([top2 + i, start + i], color),
                //     ([top1 + i, start + i], color),
                //     ([top1 + i, width], color),
                // ]);
                // self.draw.polyline([
                //     ([bottom1 - i, width], color),
                //     ([bottom1 - i, end - i], color),
                //     ([bottom2 - i, end - i], color),
                //     ([bottom2 - i, 0], color),
                // ]);
            }
        }
    }

    fn render_ast_helpers(&mut self) {
        const UP_DOWN: &str = "↕";
        const UP: &str = "↑";
        const DOWN: &str = "↓";
        const LEFT: &str = "←";
        const RIGHT: &str = "→";
        const STYLES: Styles = Styles {
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            foreground: Rgba::WHITE,
            background: Rgba::TRANSPARENT,
            underline: false,
            strike: false,
        };

        let range = match self.document.selection().as_ast() {
            Some(range) => range,
            None => return,
        };
        let [up, down, left, right] = std::array::from_fn(|i| {
            [
                Selection::move_up,
                Selection::move_down,
                Selection::move_left,
                Selection::move_right,
            ][i](self.document.selection(), self.document)
            .and_then(|selection| selection.as_ast())
            .filter(|selection| *selection != range)
            .map(|range| range.start)
        });

        fn draw(renderer: &mut Renderer, cursor: Cursor, str: &str) {
            let line = {
                let mut shaper = Line::shaper(renderer.context, renderer.view.font_size);
                shaper.push(&str, renderer.view.family, STYLES);
                shaper.line()
            };
            let top = renderer.row(cursor);
            let left = renderer.column(cursor)
                - (str == LEFT)
                    .then(|| line.glyphs().first())
                    .flatten()
                    .map(|glyph| glyph.advance as i32)
                    .unwrap_or_default();

            renderer.draw.glyphs(
                renderer.context,
                Position { top, left },
                &line,
                0,
                // Default::default(), // TODO
            );
        }

        match (up, down) {
            (Some(up), Some(down)) if up == down => {
                draw(self, up, UP_DOWN);
            }
            _ => {
                if let Some(cursor) = up {
                    draw(self, cursor, UP);
                }

                if let Some(cursor) = down {
                    draw(self, cursor, DOWN);
                }
            }
        }

        if let Some(cursor) = left {
            draw(self, cursor, LEFT);
        }

        if let Some(cursor) = right {
            draw(self, cursor, RIGHT);
        }
    }

    fn render_scrollbar(&mut self) {
        let region_height_in_lines = self.draw.region().height / self.view.line_height();

        if self.rope_lines <= region_height_in_lines as usize {
            return;
        }

        let scroll_top_in_lines = self.scroll_top as f32 / self.view.line_height() as f32;
        let top = scroll_top_in_lines / self.rope_lines as f32;
        let height = region_height_in_lines as f32 / self.rope_lines as f32;
        let rectangle = Rectangle {
            top: (top * self.draw.region().height as f32).round() as i32,
            height: (height * self.draw.region().height as f32).round() as u32,
            left: (self.advance / 2.0).round() as i32,
            width: (self.advance / 4.0).round() as u32,
        };

        self.draw
            .rectangle(rectangle, Self::SCROLLBAR.transparent(self.scrollbar_alpha));
    }
}
