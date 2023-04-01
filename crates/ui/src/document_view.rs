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
            lines: Vec::default(),
        }
    }

    pub fn prepare(
        &mut self,
        surface: &mut Surface,
        context: &mut Context,
        document: &Document,
        lines: Range<usize>,
    ) {
        self.lines.clear();

        let highlights = Highlights::new(
            document.rope(),
            document.tree().unwrap().root_node(),
            lines,
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
        let line_height = self.line_height as f32;
        let start = (scroll_top as f32 / line_height).floor() as usize;
        let end = 1 + start + (surface.height() as f32 / line_height).ceil() as usize;

        self.prepare(surface, context, document, start..end);

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
