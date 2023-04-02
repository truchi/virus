use ropey::Rope;
use std::{borrow::Cow, ops::Range};
use swash::CacheKey;
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
        if self.rope.is_instance(document.rope())
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
    }
}
