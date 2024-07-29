use crate::{rope::CursorRef, syntax::ThemeKey};
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

        let (start, end) = {
            let cursor = CursorRef::with(rope.slice(..));

            (
                cursor.at_line(lines.start),
                if lines.end == rope.len_lines() {
                    cursor.at_end()
                } else {
                    cursor.at_line(lines.end)
                },
            )
        };

        {
            let mut cursor = QueryCursor::new();
            cursor.set_point_range(Range {
                start: Point {
                    row: start.line(),
                    column: start.column(),
                },
                end: Point {
                    row: end.line(),
                    column: end.column(),
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
        .filter(|capture| start.index() < capture.end_index)
        .filter(|capture| capture.start_index < end.index())
        .map(|capture| {
            let (start_index, start_line, start_column) = if start.index() <= capture.start_index {
                (
                    capture.start_index,
                    capture.start_line,
                    capture.start_column,
                )
            } else {
                (start.index(), start.line(), start.column())
            };
            let (end_index, end_line, end_column) = if capture.end_index <= end.index() {
                (capture.end_index, capture.end_line, capture.end_column)
            } else {
                (end.index(), end.line(), end.column())
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
