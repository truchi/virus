use crate::theme::UiTheme;
use std::ops::Range;
use swash::shape::ShapeContext;
use virus_document::{
    document::{Document, DocumentId},
    highlights::Highlights,
};
use virus_graphics::text::{Fonts, Glyphs};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Highlighted                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO optimizations!?

#[derive(Default, Debug)]
pub struct Highlighted {
    theme: Option<UiTheme>,
    document_id: Option<DocumentId>,
    version: usize,
    lines: Vec<Glyphs>,
}

impl Highlighted {
    pub fn get(
        &mut self,
        fonts: &Fonts,
        shape: &mut ShapeContext,
        theme: UiTheme,
        document: &Document,
        range: Range<usize>,
    ) -> &[Glyphs] {
        assert!(
            self.document_id
                .map(|document_id| document_id == document.id())
                .unwrap_or(true),
            "Use the same document here",
        );
        self.document_id = Some(document.id());

        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= document.rope().len_lines());

        if !(self.version == document.version() && self.theme == Some(theme)) {
            self.theme = Some(theme);
            self.version = document.version();
            self.lines.clear();
            self.shape(fonts, shape, document);
        }

        &self.lines[range]
    }
}

/// Private.
impl Highlighted {
    fn shape(&mut self, fonts: &Fonts, shape: &mut ShapeContext, document: &Document) {
        let theme = self.theme.unwrap();
        let range = 0..document.rope().len_lines();

        let highlights = Highlights::new(
            document.rope(),
            document.tree().root_node(),
            range.clone(),
            document.highlights(),
        );

        let mut shaper = Glyphs::shaper(fonts, shape, theme.family, theme.font_size, theme.advance);

        self.lines.clear();

        // TODO ensure line break detection across highlights
        for (str, tag) in highlights.highlights() {
            let styles = theme.syntax[tag];
            let mut str = str.as_str();

            loop {
                let (before, after) = {
                    match str.find(['\r', '\n']) {
                        Some(index) => {
                            let (left, right) = str.split_at(index);

                            match right.as_bytes()[0] {
                                b'\r' => {
                                    if right.as_bytes().get(1) == Some(&b'\n') {
                                        (left, Some(&right[2..]))
                                    } else {
                                        (left, Some(&right[1..]))
                                    }
                                }
                                b'\n' => (left, Some(&right[1..])),
                                _ => unreachable!(),
                            }
                        }
                        None => (str, None),
                    }
                };

                shaper.push(before, styles);

                if let Some(after) = after {
                    self.lines.push(shaper.glyphs());
                    str = after;
                } else {
                    break;
                }
            }
        }

        self.lines.push(shaper.glyphs());
    }
}
