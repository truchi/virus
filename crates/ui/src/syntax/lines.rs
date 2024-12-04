use crate::{syntax::Capture, theme::UiTheme};
use std::{borrow::Cow, cell::RefCell, ops::Range, rc::Weak};
use virus_editor::document::{Document, DocumentId};
use virus_graphics::text::{Cluster, Context, Line};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Lines                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO optimize: have a list of shaped ranges

#[derive(Debug)]
pub struct Lines {
    document_id: Option<DocumentId>,
    version: usize,
    lines: Vec<Line>,
    theme: Weak<RefCell<UiTheme>>,
}

impl Lines {
    pub fn new(theme: Weak<RefCell<UiTheme>>) -> Self {
        Self {
            document_id: Default::default(),
            version: Default::default(),
            lines: Default::default(),
            theme,
        }
    }

    pub fn lines(
        &mut self,
        context: &mut Context,
        document: &Document,
        range: Range<usize>,
    ) -> &[Line] {
        debug_assert!(range.start <= range.end);

        if !(self.document_id == Some(document.id()) && self.version == document.version()) {
            self.document_id = Some(document.id());
            self.version = document.version();
            self.lines.clear();
            self.shape(context, document);
        }

        let start = range.start.min(document.rope().len_lines());
        let end = range.end.min(document.rope().len_lines());
        &self.lines[start..end]
    }
}

/// Private.
impl Lines {
    fn shape(&mut self, context: &mut Context, document: &Document) {
        let theme = self.theme.upgrade().unwrap();
        let theme = theme.borrow();
        let range = 0..document.rope().len_lines();

        let mut shapers = document
            .rope()
            .lines()
            .map(|slice| Line::shaper(&Cow::from(slice), usize::MAX, theme.syntax.default))
            .collect::<Vec<_>>();

        for capture in Capture::captures(
            document.rope(),
            document.tree().root_node(),
            range.clone(),
            document.highlights(),
        ) {
            let find = |clusters: &[Cluster], column| {
                clusters
                    .iter()
                    .position(|cluster| cluster.range().contains(&column))
                    .expect("Cannot find highlight in line")
            };
            let update = |clusters: &mut [Cluster]| {
                for cluster in clusters {
                    if capture.pattern < cluster.pattern() {
                        *cluster.pattern_mut() = capture.pattern;
                        *cluster.styles_mut() = theme.syntax[capture.key];
                    }
                }
            };

            let start_line = capture.start_line - range.start;
            let end_line = capture.end_line - range.start;

            if start_line == end_line {
                let shaper = &mut shapers[start_line];
                let start = find(shaper.clusters(), capture.start_column);
                let end = find(&shaper.clusters()[start..], capture.end_column);

                update(&mut shaper.clusters_mut()[start..][..end]);
            } else {
                let shaper = &mut shapers[start_line];
                let start = find(shaper.clusters(), capture.start_column);

                update(&mut shaper.clusters_mut()[start..]);

                for shaper in &mut shapers[start_line..end_line - 1] {
                    update(shaper.clusters_mut());
                }

                if let Some(shaper) = shapers.get_mut(end_line) {
                    let end = find(shaper.clusters(), capture.end_column);

                    update(&mut shaper.clusters_mut()[..end]);
                } else {
                    debug_assert!(capture.end_column == 0);
                }
            }
        }

        self.lines = shapers
            .into_iter()
            .map(|line| line.shape(context, theme.family, theme.font_size))
            .collect();
    }
}
