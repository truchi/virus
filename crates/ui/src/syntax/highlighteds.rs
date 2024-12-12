use crate::theme::UiTheme;
use std::ops::Range;
use virus_editor::{
    ast::Highlights,
    document::{Document, DocumentId},
};
use virus_graphics::text::{Context, Glyphs};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Highlighted                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO optimizations!?

#[derive(Debug)]
pub struct Highlighted {
    theme: UiTheme,
    document_id: DocumentId,
    version: usize,
    lines: Vec<Glyphs>,
}

impl Highlighted {
    pub fn new(theme: UiTheme, document_id: DocumentId) -> Self {
        Self {
            theme,
            document_id,
            version: Default::default(),
            lines: Default::default(),
        }
    }

    pub fn get(
        &mut self,
        context: &mut Context,
        theme: UiTheme,
        document: &Document,
        range: Range<usize>,
    ) -> &[Glyphs] {
        assert_eq!(self.document_id, document.id());

        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= document.rope().len_lines());

        if !(self.version == document.version() && self.theme == theme) {
            self.theme = theme;
            self.version = document.version();
            self.lines.clear();
            self.shape(context, document);
        }

        &self.lines[range]
    }
}

/// Private.
impl Highlighted {
    fn shape(&mut self, context: &mut Context, document: &Document) {
        let range = 0..document.rope().len_lines();

        let highlights = Highlights::new(
            document.rope(),
            document.tree().root_node(),
            range.clone(),
            document.highlights(),
        );

        let mut shaper = Glyphs::shaper(context, self.theme.family, self.theme.font_size);

        self.lines.clear();

        // TODO ensure line break detection across highlights
        for (str, tag) in highlights.highlights() {
            let styles = self.theme.syntax[tag];
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
