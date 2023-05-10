use crate::theme::Theme;
use ropey::Rope;
use std::ops::Range;
use tree_sitter::{Node, Point, Query, QueryCursor};
use virus_common::{Cursor, Style};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlight                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct Highlight {
    pub start: Cursor,
    pub end: Cursor,
    pub style: Style,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlights                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Procuces an iterator of [`Highlight`]s.
pub struct Highlights<'tree, 'rope> {
    rope: &'rope Rope,
    root: Node<'tree>,
    start: Cursor,
    end: Cursor,
    query: Query,
    theme: Theme,
}

impl<'tree, 'rope> Highlights<'tree, 'rope> {
    /// Creates a new [`Highlights`] for `rope` with `root`,
    /// clamped by `lines`, for `query` with `theme`.
    pub fn new(
        rope: &'rope Rope,
        root: Node<'tree>,
        lines: Range<usize>,
        query: Query,
        theme: Theme,
    ) -> Self {
        let Range { start, end } = lines;

        let lines = rope.len_lines();
        let end = end.min(lines);
        let start = start.min(end);

        let start = Cursor::new(rope.line_to_byte(start), start, 0);
        let end = Cursor::new(rope.line_to_byte(end), end, 0);

        Self {
            rope,
            root,
            query,
            start,
            end,
            theme,
        }
    }

    /// Returns an iterator of [`Highlight`]s.
    pub fn iter(&self) -> impl '_ + Iterator<Item = Highlight> {
        // Use `tree-sitter` to get a sorted list of catpures for `self.query`
        let captures = if (self.start.line..self.end.line).is_empty() {
            vec![]
        } else {
            struct Capture<'a> {
                start: Cursor,
                end: Cursor,
                pattern: usize,
                name: &'a str,
            }

            let mut captures = Vec::<Capture>::new();
            let mut cursor = {
                let start = Point::new(self.start.line, 0);
                let end = Point::new(self.end.line, 0);
                let mut cursor = QueryCursor::new();
                cursor.set_point_range(start..end);
                cursor
            };
            let it = cursor
                .matches(&self.query, self.root, |node: Node| {
                    self.rope
                        .get_byte_slice(node.byte_range())
                        .unwrap()
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
                        name: &self.query.capture_names()[capture.index as usize],
                    })
                })
                .flatten();

            for capture in it {
                // We want all captures ordered by start index,
                // favoring lower pattern index when captured multiple times.
                // This is what `Helix` does, and we use their queries.
                // It seems like patterns are written in that specific order.
                match captures
                    .binary_search_by_key(&capture.start.index, |capture| capture.start.index)
                {
                    Ok(index) => {
                        // Favoring lower index pattern
                        if captures[index].pattern > capture.pattern {
                            captures[index] = capture
                        }
                    }
                    Err(index) => {
                        // Captures must not overlap, otherwise what can we do?
                        debug_assert!(
                            index
                                .checked_sub(1)
                                .map(|prev| captures[prev].end.index <= capture.start.index)
                                .unwrap_or(true),
                            "overlapping capture",
                        );

                        captures.insert(index, capture);
                    }
                }
            }

            if captures.is_empty() {
                vec![Capture {
                    start: self.start,
                    end: self.end,
                    pattern: usize::MAX,
                    name: "",
                }]
            } else {
                captures
            }
        };

        // Filter on line range and crop overlapping captures
        let highlights = captures
            .into_iter()
            .map(|highlight| Highlight {
                start: highlight.start,
                end: highlight.end,
                style: *self.theme.get(highlight.name),
            })
            .filter(|highlight| self.start.index < highlight.end.index)
            .filter(|highlight| highlight.start.index < self.end.index)
            .map(|highlight| Highlight {
                start: if self.start.index < highlight.start.index {
                    highlight.start
                } else {
                    self.start
                },
                end: if highlight.end.index < self.end.index {
                    highlight.end
                } else {
                    self.end
                },
                style: highlight.style,
            });

        // Intersperse with in-between selections
        let highlights = {
            let mut highlights = highlights.peekable();
            let mut prev = Highlight {
                start: self.start,
                end: self.start,
                style: *self.theme.default(),
            };

            std::iter::from_fn(move || {
                let next = highlights.peek()?;

                prev = if prev.end.index == next.start.index {
                    highlights.next()?
                } else {
                    Highlight {
                        start: prev.end,
                        end: next.start,
                        style: *self.theme.default(),
                    }
                };

                Some(prev)
            })
        };

        // Slice highlights to line boundaries
        let highlights = {
            let mut highlights = highlights;
            let mut next = highlights.next();

            std::iter::from_fn(move || {
                let highlight = next?;

                if highlight.start.line == highlight.end.line {
                    next = highlights.next();
                    Some(highlight)
                } else {
                    // NOTE: this does not take line breaks into account!
                    // It could be nice to remove the line break (if any) from the item,
                    // but this should not be a real issue.
                    let end = self
                        .rope
                        .try_line_to_byte(highlight.start.line + 1)
                        .unwrap();

                    next = Some(Highlight {
                        start: Cursor::new(end, highlight.start.line + 1, 0),
                        end: highlight.end,
                        style: highlight.style,
                    });
                    Some(Highlight {
                        start: highlight.start,
                        end: Cursor::new(
                            end,
                            highlight.start.line,
                            highlight.start.column + (end - highlight.start.index),
                        ),
                        style: highlight.style,
                    })
                }
            })
            .filter(|highlight| highlight.start.index != highlight.end.index)
        };

        // That was hard! Let's make sure we made it right:
        highlights.inspect(|highlight| {
            // In the requested line range
            debug_assert!((self.start.line..self.end.line).contains(&highlight.start.line));
            debug_assert!((self.start.line..self.end.line).contains(&highlight.end.line));
            // One-line
            debug_assert!(highlight.start.line == highlight.end.line);
            // Not empty
            debug_assert!(highlight.start.index != highlight.end.index);
            debug_assert!(highlight.end.column != 0);
        })
    }
}
