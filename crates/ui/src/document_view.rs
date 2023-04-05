use ropey::Rope;
use std::{borrow::Cow, ops::Range};
use swash::CacheKey;
use virus_common::Rgba;
use virus_editor::{Document, Highlight, Highlights, Theme};
use virus_graphics::{
    pixels_mut::Surface,
    text::{Context, FontSize, Line},
};

/// LineHeight unit (`u32`).
pub type LineHeight = u32;

pub struct DocumentView {
    query: String,
    theme: Theme,
    font: CacheKey,
    font_size: FontSize,
    line_height: LineHeight,
    rope: Rope,
    range: Range<usize>,
    lines: Vec<(usize, Line)>,
}

impl DocumentView {
    pub fn new(
        query: String,
        theme: Theme,
        font: CacheKey,
        font_size: FontSize,
        line_height: LineHeight,
    ) -> Self {
        Self {
            query,
            theme,
            font,
            font_size,
            line_height,
            rope: Default::default(),
            range: 0..0,
            lines: Vec::default(),
        }
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

        // FIXME: why?????!!!!!!!!! rrrrrrrrrrr
        // println!("prepare");
        // dbg!(self.rope.is_instance(document.rope()));
        // dbg!(self.rope == *document.rope());
        // dbg!(self.range.contains(&start));
        // dbg!(self.range.contains(&(end - 1)));

        // Apply margins
        let margin = (end - start) / 2;
        let start = start.saturating_sub(margin);
        let end = (end + margin).max(self.rope.len_lines());

        self.lines.clear();
        self.range = start..end;

        let highlights = Highlights::new(
            document.rope(),
            document.tree().unwrap().root_node(),
            start..end,
            document.query(&self.query).unwrap(),
            self.theme,
        );

        let mut prev_line = None;
        let mut shaper = Line::shaper(context, self.font_size);

        for Highlight { start, end, style } in highlights.iter() {
            let line = start.line;
            debug_assert!(start.line == end.line);

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
                self.font,
                style,
            );
        }

        if let Some(line) = prev_line {
            self.lines.push((line, shaper.line()));
        }
    }

    pub fn render(
        &mut self,
        surface: &mut Surface,
        context: &mut Context,
        document: &Document,
        scroll_top: u32,
    ) {
        let start = (scroll_top as f32 / self.line_height as f32).floor() as usize;
        let len = (surface.height() as f32 / self.line_height as f32).ceil() as usize;

        self.prepare(context, document, start..start + len + 1);

        for (index, line) in &self.lines {
            surface.draw_line(
                context,
                *index as i32 * self.line_height as i32 - scroll_top as i32,
                0,
                &line,
                self.line_height as u32,
            );
        }

        self.render_selection(surface, document, scroll_top);
    }

    pub fn render_selection(
        &mut self,
        surface: &mut Surface,
        document: &Document,
        scroll_top: u32,
    ) {
        let color = Rgba::WHITE;
        let selection = document.selection();

        // Not sure what is best when glyph ranges and cursor columns don't align...!
        // This includes overlapping glyphs on both ends.
        // Note that we could be more clever when looking for end position,
        // by starting from the end when it's closer.
        let get_range = |line| {
            // TODO: what if not in there (empty line)?
            let glyphs = self
                .lines
                .iter()
                .find(|(index, _)| *index == line)
                .unwrap()
                .1
                .glyphs();

            // TODO: buggy at line boundaries
            let start = || {
                glyphs
                    .iter()
                    .find(|glyph| glyph.range.end as usize > selection.start.column)
                    .map(|glyph| glyph.offset)
                    .unwrap_or(0.)
            };
            let end = || {
                glyphs
                    .iter()
                    .find(|glyph| glyph.range.end as usize >= selection.end.column)
                    .map(|glyph| glyph.offset + glyph.advance)
                    .unwrap_or(0.)
            };
            let last = || {
                glyphs
                    .last()
                    .map(|glyph| glyph.offset + glyph.advance)
                    .unwrap_or(0.)
            };

            if line == selection.start.line {
                if line == selection.end.line {
                    // Single line
                    (start(), end())
                } else {
                    // First line
                    (start(), last())
                }
            } else if line == selection.end.line {
                // Last line
                (0., if selection.end.column == 0 { 0. } else { end() })
            } else {
                // Middle line
                (0., last())
            }
        };

        let line_pos = |line| line as i32 * self.line_height as i32 - scroll_top as i32;

        let mut prev = None;

        for line in selection.start.line..=selection.end.line {
            let top = line_pos(line);
            let bottom = line_pos(line + 1);
            let (start, end) = get_range(line);
            let left = start as i32;
            let right = end as i32;

            surface.draw_vertical_line(top..bottom, left, color);
            surface.draw_vertical_line(top..bottom, right, color);

            if let Some((_, prev_left, prev_right)) = prev {
                debug_assert!(left <= prev_left);

                if right < prev_left {
                    surface.draw_horizontal_line(top, left..right, color);
                    surface.draw_horizontal_line(top, prev_left..prev_right, color);
                } else if right == prev_left {
                    surface.draw_horizontal_line(top, left..prev_right, color);
                } else if right <= prev_right {
                    surface.draw_horizontal_line(top, left..prev_left, color);
                    surface.draw_horizontal_line(top, right..prev_right, color);
                } else {
                    surface.draw_horizontal_line(top, left..prev_left, color);
                    surface.draw_horizontal_line(top, prev_right..right, color);
                }
            } else {
                surface.draw_horizontal_line(top, left..right, color);
            }

            prev = Some((line, left, right));
        }

        if let Some((line, left, right)) = prev {
            let bottom = line_pos(line + 1);
            surface.draw_horizontal_line(bottom, left..right, color);
        }
    }
}
