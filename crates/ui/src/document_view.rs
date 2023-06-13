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
    text::{Context, FontFamilyKey, FontSize, Line, LineHeight},
    wgpu::Draw,
};

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

    pub fn prepare(
        &mut self,
        context: &mut Context,
        document: &Document,
        Range { start, end }: Range<usize>,
    ) {
        // No need to prepare if same rope and similar range
        if (self.rope.is_instance(document.rope()) || self.rope == *document.rope())
            && self.range.contains(&start)
            && self.range.contains(&(end - 1))
        {
            return;
        }

        // Apply margins
        let margin = (end - start) / 2;
        let start = start.saturating_sub(margin);
        let end = (end + margin).max(self.rope.len_lines());

        self.lines.clear();
        self.range = start..end;
        self.rope = document.rope().clone();

        let highlights = Highlights::new(
            document.rope(),
            document.tree().unwrap().root_node(),
            start..end,
            &document.query(&self.query).unwrap(),
        );

        let mut prev_line = None;
        let mut shaper = Line::shaper(context, self.font_size);

        for Highlight { start, end, key } in highlights.highlights() {
            debug_assert!(start.line == end.line);
            let line = start.line;

            if prev_line != Some(line) {
                if let Some(line) = prev_line {
                    self.lines.push((line, shaper.line()));
                }

                prev_line = Some(line);
                shaper = Line::shaper(context, self.font_size);
            }

            shaper.push(
                // We cow to make sure ligatures are not split between rope chunks
                &Cow::from(
                    document
                        .rope()
                        .get_byte_slice(start.index..end.index)
                        .unwrap(),
                ),
                self.family,
                self.theme[key],
            );
        }

        if let Some(line) = prev_line {
            self.lines.push((line, shaper.line()));
        }
    }

    pub fn render(
        &mut self,
        mut draw: Draw,
        context: &mut Context,
        document: &Document,
        scroll_top: u32,
    ) {
        let start = (scroll_top as f32 / self.line_height as f32).floor() as usize;
        let len = (draw.height() as f32 / self.line_height as f32).ceil() as usize;

        self.prepare(context, document, start..start + len + 1);

        for (index, line) in &self.lines {
            let top = *index as i32 * self.line_height as i32 - scroll_top as i32;
            draw.glyphs(context, [top, 0], &line, self.line_height as u32);
        }

        // TODO
        // self.render_selection(surface, document, scroll_top);
    }

    // TODO
    // pub fn render_selection(
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
