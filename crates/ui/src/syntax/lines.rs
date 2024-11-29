use crate::{syntax::Capture, theme::UiTheme};
use std::{borrow::Cow, ops::Range};
use virus_editor::{document::Document, rope::Selection};
use virus_graphics::text::{Cluster, Context, Line};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Lines                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Default, Debug)]
pub struct Lines {
    selection: Selection,
    version: usize,
    range: Range<usize>,
    lines: Vec<Line>,
}

impl Lines {
    pub fn lines(
        &mut self,
        context: &mut Context,
        document: &Document,
        range: Range<usize>,
        theme: &UiTheme,
    ) -> &[Line] {
        let in_cache = self.version == document.version()
            && self.range.start <= range.start
            && range.end <= self.range.end;

        if !in_cache {
            let range = {
                let margin = range.len() / 2;
                let start = range.start.saturating_sub(margin);
                let end = (range.end + margin).min(document.rope().len_lines());
                start..end
            };

            *self = Self {
                selection: document.selection(),
                version: document.version(),
                range: range.clone(),
                lines: Self::shape(context, document, range, theme),
            };
        } else if self.selection != document.selection() {
            let mut lines = vec![
                self.selection.anchor.line,
                self.selection.head.line,
                document.selection().anchor.line,
                document.selection().head.line,
            ];
            lines.sort();
            lines.dedup();

            for line in lines {
                if self.range.contains(&line) {
                    let mut lines =
                        Self::shape(context, document, line..line + 1, theme).into_iter();

                    self.lines[line - self.range.start] = lines.next().unwrap();
                    debug_assert!(lines.next().is_none());
                }
            }

            self.selection = document.selection();
        }

        let start = range.start - self.range.start;
        let end = start + range.len();
        &self.lines[start..end]
    }

    fn shape(
        context: &mut Context,
        document: &Document,
        range: Range<usize>,
        theme: &UiTheme,
    ) -> Vec<Line> {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= document.rope().len_lines());

        let mut lines = document
            .rope()
            .lines_at(range.start)
            .take(range.len())
            .map(|slice| Line::shaper(&Cow::from(slice), usize::MAX, theme.syntax.default))
            .collect::<Vec<_>>();

        debug_assert!(lines.len() == range.len());

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
                let line = &mut lines[start_line];
                let start = find(line.clusters(), capture.start_column);
                let end = find(&line.clusters()[start..], capture.end_column);

                update(&mut line.clusters_mut()[start..][..end]);
            } else {
                let line = &mut lines[start_line];
                let start = find(line.clusters(), capture.start_column);

                update(&mut line.clusters_mut()[start..]);

                for line in &mut lines[start_line..end_line - 1] {
                    update(line.clusters_mut());
                }

                if let Some(line) = lines.get_mut(end_line) {
                    let end = find(line.clusters(), capture.end_column);

                    update(&mut line.clusters_mut()[..end]);
                } else {
                    debug_assert!(capture.end_column == 0);
                }
            }
        }

        lines
            .into_iter()
            .map(|line| line.shape(context, theme.family, theme.font_size))
            .collect()
    }
}
