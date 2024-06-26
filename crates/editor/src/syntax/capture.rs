use crate::{rope::RopeExt, syntax::ThemeKey};
use ropey::Rope;
use std::ops::Range;
use tree_sitter::{Node, Point, Query, QueryCursor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Capture                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `tree-sitter` capture (internal to [`Highlights`]).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Capture {
    pub start_index: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_index: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub pattern: usize,
    pub key: ThemeKey,
}

impl Capture {
    /// Creates a new [`Highlights`] for `rope` with `root`, clamped by `lines`, with `query`.
    pub fn captures(rope: &Rope, root: Node, lines: Range<usize>, query: &Query) -> Vec<Self> {
        debug_assert!(lines.start <= lines.end);
        debug_assert!(lines.end <= rope.len_lines());

        let (start, end) = (
            rope.cursor().at_line(lines.start),
            if lines.end == rope.len_lines() {
                rope.cursor().at_end()
            } else {
                rope.cursor().at_line(lines.end)
            },
        );
        let start_index = start.index();
        let start_line = start.line(rope);
        let start_column = start.column(rope);
        let end_index = end.index();
        let end_line = end.line(rope);
        let end_column = end.column(rope);

        {
            let mut cursor = QueryCursor::new();
            cursor.set_point_range(Range {
                start: Point {
                    row: start_line,
                    column: start_column,
                },
                end: Point {
                    row: end_line,
                    column: end_column,
                },
            });
            cursor
        }
        .matches(query, root, |node: Node| {
            rope.byte_slice(node.byte_range())
                .chunks()
                .map(|chunk| chunk.as_bytes())
        })
        .map(|captures| {
            captures.captures.into_iter().map(move |capture| Capture {
                start_index: capture.node.start_byte(),
                start_line: capture.node.start_position().row,
                start_column: capture.node.start_position().column,
                end_index: capture.node.end_byte(),
                end_line: capture.node.end_position().row,
                end_column: capture.node.end_position().column,
                pattern: captures.pattern_index,
                key: ThemeKey::new(&query.capture_names()[capture.index as usize]),
            })
        })
        .flatten()
        .filter(|capture| start_index < capture.end_index)
        .filter(|capture| capture.start_index < end_index)
        .map(|capture| {
            let (start_index, start_line, start_column) = if start_index <= capture.start_index {
                (
                    capture.start_index,
                    capture.start_line,
                    capture.start_column,
                )
            } else {
                (start_index, start_line, start_column)
            };
            let (end_index, end_line, end_column) = if capture.end_index <= end_index {
                (capture.end_index, capture.end_line, capture.end_column)
            } else {
                (end_index, end_line, end_column)
            };

            Capture {
                start_index,
                start_line,
                start_column,
                end_index,
                end_line,
                end_column,
                pattern: capture.pattern,
                key: capture.key,
            }
        })
        .collect()
    }
}
