use crate::{cursor::Cursor, rope::RopeExt, syntax::ThemeKey};
use ropey::Rope;
use std::ops::Range;
use tree_sitter::{Node, Query, QueryCursor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Capture                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `tree-sitter` capture (internal to [`Highlights`]).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Capture {
    pub start: Cursor,
    pub end: Cursor,
    pub pattern: usize,
    pub key: ThemeKey,
}

impl Capture {
    /// Creates a new [`Highlights`] for `rope` with `root`, clamped by `lines`, with `query`.
    pub fn captures(rope: &Rope, root: Node, lines: Range<usize>, query: &Query) -> Vec<Self> {
        debug_assert!(lines.start <= lines.end);
        debug_assert!(lines.end <= rope.len_lines());

        let (start, end) = (
            rope.cursor().line(lines.start),
            rope.cursor().line(lines.end),
        );

        {
            let mut cursor = QueryCursor::new();
            cursor.set_point_range(start.into()..end.into());
            cursor
        }
        .matches(query, root, |node: Node| {
            rope.byte_slice(node.byte_range())
                .chunks()
                .map(|chunk| chunk.as_bytes())
        })
        .map(|captures| {
            captures.captures.into_iter().map(move |capture| Capture {
                start: Cursor::new(
                    capture.node.start_byte(),
                    capture.node.start_position().row,
                    capture.node.start_position().column,
                ),
                end: Cursor::new(
                    capture.node.end_byte(),
                    capture.node.end_position().row,
                    capture.node.end_position().column,
                ),
                pattern: captures.pattern_index,
                key: ThemeKey::new(&query.capture_names()[capture.index as usize]),
            })
        })
        .flatten()
        .filter(|capture| start < capture.end)
        .filter(|capture| capture.start < end)
        .map(|capture| Capture {
            start: if start <= capture.start {
                capture.start
            } else {
                start
            },
            end: if capture.end <= end { capture.end } else { end },
            pattern: capture.pattern,
            key: capture.key,
        })
        .collect()
    }
}
