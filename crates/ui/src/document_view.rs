//! DocumentView.
//!
//! # Selections
//!
//! Square when touching borders (final `\n` is selected), round otherwise.
//!
//! ```text
//!
//! ┌──────────────────────────────────────────────────────────────┐
//! │Lorem ipsum dolor sit amet, qui minim labore.                 │
//! │                                                              │
//! │Lorem ipsum dolor sit amet, qui minim labore.                 │
//! │                                                              │
//! │Lorem ipsum dolor sit amet, qui minim labore.                 │
//! ├──────────────────────────────────────────────────────────────┤
//! │Lorem ipsum dolor sit amet, qui minim labore.                 │ // Full line (\n), square
//! ├──────────────────────────────────────────────────────────────┤
//! │Lorem ipsum dolor sit amet, qui minim labore.                 │
//! │                                                              │
//! │Lorem ipsum dolor sit amet, qui minim labore.                 │
//! │                            ╭─────────────────╮               │
//! │Lorem ipsum dolor sit amet, │officia excepteur│ labore.       │ // Single line, round
//! │                            ╰─────────────────╯               │
//! │culpa sint ad nisi Lorem pariatur mollit ex esse amet.        │
//! │                                       ╭──────────────────────┤
//! │Nisi anim cupidatat excepteur officia. │Reprehenderit         │ // Two lines, disjoined
//! ├────────────────────────╮              ╰──────────────────────┤
//! │amet voluptate voluptate│ dolor minim nulla est proident.     │
//! ├────────────────────────╯                                     │
//! │Lorem ipsum dolor sit amet, qui minim labore adipisicing.     │
//! │                                  ╭───────────────────────────┤
//! │Sit irure elit esse ea nulla sunt │ex occaecat reprehenderit  │ // Two lines, joined
//! ├──────────────────────────────────╯            ╭──────────────┤
//! │Lorem duis laboris cupidatat officia voluptate.│              │
//! ├───────────────────────────────────────────────╯              │
//! │Lorem ipsum dolor sit amet, qui minim labore adipisicing.     │
//! │                                                       ╭──────┤
//! │Culpa proident adipisicing id nulla nisi laboris ex in │Lorem │ // Three lines
//! ├───────────────────────────────────────────────────────╯      │
//! │Aliqua reprehenderit commodo ex non excepteur duis sunt velit.│
//! │                                        ╭─────────────────────┤
//! │Voluptate laboris sint cupidatat ullamco│ ut ea consectetur.  │
//! ├────────────────────────────────────────╯                     │
//! │Lorem ipsum dolor sit amet, qui minim labore adipisicing.     │
//! │                                                              │
//! │Lorem ipsum dolor sit amet, qui minim labore adipisicing.     │
//! │                                                       ╭──────┤
//! │Culpa proident adipisicing id nulla nisi laboris ex in │Lorem │ // Three lines
//! ├───────────────────────────────────────────────────────╯      │
//! │Aliqua reprehenderit commodo ex non excepteur duis sunt velit.│
//! │                                                              │
//! │Voluptate laboris sint cupidatat ullamco ut ea consectetur.   │ // (\n)
//! └──────────────────────────────────────────────────────────────┘
//! ```

use ropey::Rope;
use std::{borrow::Cow, ops::Range};
use virus_editor::{
    document::Document,
    highlights::{Highlight, Highlights},
    theme::Theme,
};
use virus_graphics::{
    colors::Rgba,
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
        draw: &mut Draw,
        context: &mut Context,
        document: &Document,
        scroll_top: u32,
    ) {
        Renderer::new(self, context, draw, document, scroll_top).render();
    }

    // TODO
    // fn render_selection(
    //     &mut self,
    //     surface: &mut Surface,
    //     document: &Document,
    //     scroll_top: u32,
    // ) {
    //     use virus_graphics::pixels_mut::Quadrant::*;

    //     let color = Rgba::WHITE;
    //     let selection = document.selection();
    //     let surface_width = surface.width();

    //     let line_top = |line| line as i32 * self.line_height as i32 - scroll_top as i32;

    //     // Not sure what is best when glyph ranges and cursor columns don't align...!
    //     // This includes overlapping glyphs on both ends.
    //     let line_range = |line| {
    //         // TODO: what if not in there (empty line)?
    //         let glyphs = self
    //             .lines
    //             .iter()
    //             .find(|(index, _)| *index == line)
    //             .map(|(_, line)| line.glyphs())
    //             .unwrap_or_default();

    //         // NOTE: we could try to guess which end to start from
    //         // TODO: buggy at line boundaries?
    //         let start = || {
    //             glyphs
    //                 .iter()
    //                 .find(|glyph| glyph.range.end as usize > selection.start.column)
    //                 .map(|glyph| glyph.offset)
    //                 .unwrap_or(
    //                     glyphs
    //                         .last()
    //                         .map(|glyph| glyph.offset + glyph.advance)
    //                         .unwrap_or_default(),
    //                 ) as u32
    //         };
    //         let end = || {
    //             glyphs
    //                 .iter()
    //                 .find(|glyph| glyph.range.end as usize >= selection.end.column)
    //                 .map(|glyph| glyph.offset + glyph.advance)
    //                 .unwrap_or(
    //                     glyphs
    //                         .last()
    //                         .map(|glyph| glyph.offset + glyph.advance)
    //                         .unwrap_or_default(),
    //                 ) as u32
    //         };

    //         if line == selection.start.line {
    //             if line == selection.end.line {
    //                 // Single line
    //                 (start(), end())
    //             } else {
    //                 // First line
    //                 (start(), surface_width)
    //             }
    //         } else if line == selection.end.line {
    //             // Last line
    //             (0, if selection.end.column == 0 { 0 } else { end() })
    //         } else {
    //             // Middle line
    //             debug_assert!(false); // We don't do middle lines
    //             (0, surface_width)
    //         }
    //     };

    //     let start_line = selection.start.line;
    //     let end_line = selection.end.line;
    //     let full = self.line_height;
    //     let half = self.line_height / 2;
    //     let quarter = self.line_height / 4;

    //     // Single line
    //     if selection.start.line == selection.end.line {
    //         let top = line_top(start_line);
    //         let (start, end) = line_range(start_line);
    //         let width = end - start;
    //         let radius = quarter.min(width / 2);
    //         let left = start as i32;

    //         surface.stroke_rect(top, left, width, full, radius, color);

    //         return;
    //     }

    //     // Two non-overlapping lines
    //     if selection.start.line + 1 == selection.end.line
    //         && selection.start.column > selection.end.column
    //     {
    //         let top = line_top(start_line);
    //         let bottom = top + full as i32;
    //         let (start, end) = line_range(start_line);
    //         let width = end - start;
    //         let radius = quarter.min(width / 2);
    //         let left = start as i32;

    //         surface.stroke_corner(TopLeft, top, left, width, half, radius, color);
    //         surface.stroke_corner(BottomLeft, bottom, left, width, half, radius, color);

    //         let top = line_top(end_line);
    //         let bottom = top + full as i32;
    //         let (start, end) = line_range(end_line);
    //         let width = end - start;
    //         let radius = quarter.min(width / 2);
    //         let right = end as i32;

    //         surface.stroke_corner(TopRight, top, right, width, half, radius, color);
    //         surface.stroke_corner(BottomRight, bottom, right, width, half, radius, color);

    //         return;
    //     }

    //     // Two (overlapping) lines or more
    //     let top1 = line_top(start_line);
    //     let bottom1 = top1 + full as i32;
    //     let (start1, end1) = line_range(start_line);
    //     let left1 = start1 as i32;
    //     debug_assert!(end1 == surface_width);

    //     let top2 = line_top(end_line);
    //     let bottom2 = top2 + full as i32;
    //     let (start2, end2) = line_range(end_line);
    //     let right2 = end2 as i32;
    //     debug_assert!(start2 == 0);

    //     let width = surface_width - start1;
    //     let radius = quarter.min(width / 2);
    //     surface.stroke_corner(TopLeft, top1, left1, width, half, radius, color);

    //     let width = surface_width - end2;
    //     let radius = quarter.min(width / 2);
    //     surface.stroke_corner(TopLeft, top2, right2, width, half, radius, color);

    //     let width = start1;
    //     let radius = quarter.min(width / 2);
    //     surface.stroke_corner(BottomRight, bottom1, left1, width, half, radius, color);

    //     let width = end2;
    //     let radius = quarter.min(width / 2);
    //     surface.stroke_corner(BottomRight, bottom2, right2, width, half, radius, color);
    // }
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
    start: usize,
    end: usize,
    rope_lines: usize,
    advance: Advance,
    line_numbers_width: Advance,
}

impl<'a, 'b, 'c, 'd, 'e> Renderer<'a, 'b, 'c, 'd, 'e> {
    fn new(
        view: &'a mut DocumentView,
        context: &'b mut Context,
        draw: &'c mut Draw<'d>,
        document: &'e Document,
        scroll_top: u32,
    ) -> Self {
        let start = (scroll_top as f32 / view.line_height as f32).floor() as usize;
        let end = start + (draw.height() as f32 / view.line_height as f32).ceil() as usize;
        let rope_lines = document.rope().len_lines() - 1;
        let advance = {
            let font = context
                .fonts()
                .get((view.family, FontWeight::Regular, FontStyle::Normal))
                .unwrap();
            context.advance(font.key(), view.font_size).unwrap()
        };
        let line_numbers_width = advance * (rope_lines.ilog10() + 2) as Advance;

        Self {
            view,
            context,
            draw,
            document,
            scroll_top,
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
        self.render_document((self.line_numbers_width + self.advance).ceil() as i32);
        self.render_selection();
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

        for number in self.start..self.end.min(self.rope_lines) {
            let line = {
                let mut shaper = Line::shaper(self.context, self.view.font_size);
                shaper.push(&(number + 1).to_string(), family, styles);
                shaper.line()
            };

            let top = number as i32 * self.view.line_height as i32 - self.scroll_top as i32;
            let left = (self.line_numbers_width - line.width()).round() as i32;

            self.draw.glyphs(
                self.context,
                [top, left],
                &line,
                self.view.line_height as u32,
            );
        }
    }

    fn render_document(&mut self, left: i32) {
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

    fn render_selection(&mut self) {
        const COLOR: Rgba = Rgba::WHITE;

        let selection = self.document.selection();
        let draw_width = self.draw.width();

        let line_top = |line| line as i32 * self.view.line_height as i32 - self.scroll_top as i32;

        // Not sure what is best when glyph ranges and cursor columns don't align...!
        // This includes overlapping glyphs on both ends.
        let line_range = |line| {
            // TODO: what if not in there (empty line)?
            let glyphs = self
                .view
                .lines
                .iter()
                .find(|(index, _)| *index == line)
                .map(|(_, line)| line.glyphs())
                .unwrap_or_default();

            // NOTE: we could try to guess which end to start from!
            // TODO: buggy at line boundaries?
            let start = || {
                glyphs
                    .iter()
                    .find(|glyph| glyph.range.end as usize > selection.start.column)
                    .map(|glyph| glyph.offset)
                    .unwrap_or(
                        glyphs
                            .last()
                            .map(|glyph| glyph.offset + glyph.advance)
                            .unwrap_or_default(),
                    ) as u32
            };
            let end = || {
                glyphs
                    .iter()
                    .find(|glyph| glyph.range.end as usize >= selection.end.column)
                    .map(|glyph| glyph.offset + glyph.advance)
                    .unwrap_or(
                        glyphs
                            .last()
                            .map(|glyph| glyph.offset + glyph.advance)
                            .unwrap_or_default(),
                    ) as u32
            };

            if line == selection.start.line {
                if line == selection.end.line {
                    // Single line
                    (start(), end())
                } else {
                    // First line
                    (start(), draw_width)
                }
            } else if line == selection.end.line {
                // Last line
                (0, if selection.end.column == 0 { 0 } else { end() })
            } else {
                // Middle line
                debug_assert!(false); // We don't do middle lines
                (0, draw_width)
            }
        };

        let start_line = selection.start.line;
        let end_line = selection.end.line;
        let full = self.view.line_height;
        let half = self.view.line_height / 2;
        let quarter = self.view.line_height / 4;

        // Caret
        if selection.start.index == selection.end.index {
            return;
        }

        // Single line
        if start_line == end_line {
            let top = line_top(start_line);
            let (start, end) = line_range(start_line);
            let width = end - start;
            let radius = quarter.min(width / 2);
            let left = start as i32;

            // surface.stroke_rect(top, left, width, full, radius, COLOR);
            self.draw
                .rectangle([top, left], [width, full], 1, radius, COLOR);

            return;
        }

        // Two non-overlapping lines
        if selection.start.line + 1 == selection.end.line
            && selection.start.column > selection.end.column
        {
            let top = line_top(start_line);
            let bottom = top + full as i32;
            let (start, end) = line_range(start_line);
            let width = end - start;
            let radius = quarter.min(width / 2);
            let left = start as i32;

            // surface.stroke_corner(TopLeft, top, left, width, half, radius, COLOR);
            // surface.stroke_corner(BottomLeft, bottom, left, width, half, radius, COLOR);

            let top = line_top(end_line);
            let bottom = top + full as i32;
            let (start, end) = line_range(end_line);
            let width = end - start;
            let radius = quarter.min(width / 2);
            let right = end as i32;

            // surface.stroke_corner(TopRight, top, right, width, half, radius, COLOR);
            // surface.stroke_corner(BottomRight, bottom, right, width, half, radius, COLOR);

            return;
        }

        // Two (overlapping) lines or more
        let top1 = line_top(start_line);
        let bottom1 = top1 + full as i32;
        let (start1, end1) = line_range(start_line);
        let left1 = start1 as i32;
        debug_assert!(end1 == draw_width);

        let top2 = line_top(end_line);
        let bottom2 = top2 + full as i32;
        let (start2, end2) = line_range(end_line);
        let right2 = end2 as i32;
        debug_assert!(start2 == 0);

        let width = draw_width - start1;
        let radius = quarter.min(width / 2);
        // surface.stroke_corner(TopLeft, top1, left1, width, half, radius, COLOR);

        let width = draw_width - end2;
        let radius = quarter.min(width / 2);
        // surface.stroke_corner(TopLeft, top2, right2, width, half, radius, COLOR);

        let width = start1;
        let radius = quarter.min(width / 2);
        // surface.stroke_corner(BottomRight, bottom1, left1, width, half, radius, COLOR);

        let width = end2;
        let radius = quarter.min(width / 2);
        // surface.stroke_corner(BottomRight, bottom2, right2, width, half, radius, COLOR);
    }
}
