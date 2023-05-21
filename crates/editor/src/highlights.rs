use crate::theme::ThemeKey;
use ropey::Rope;
use std::ops::Range;
use tree_sitter::{Node, Query, QueryCursor};
use virus_common::Cursor;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlight                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`Cursor`] range with a [`ThemeKey`].
#[derive(Copy, Clone, Debug)]
pub struct Highlight {
    /// Start of the highlight.
    pub start: Cursor,
    /// End of the highlight.
    pub end: Cursor,
    /// Theme key of the highlight.
    pub key: ThemeKey,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlights                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `tree-sitter` capture (internal to [`Highlights`]).
#[derive(Copy, Clone, Debug)]
struct Capture {
    start: Cursor,
    end: Cursor,
    pattern: usize,
    key: ThemeKey,
}

/// Procuces an iterator of [`Highlight`]s.
///
/// Guaranties that `Highlight`s are in the requested line range, do not span multiple lines
/// and are not empty.
#[derive(Clone, Debug)]
pub struct Highlights<'rope> {
    rope: &'rope Rope,
    start: Cursor,
    end: Cursor,
    captures: Vec<Capture>,
}

impl<'rope> Highlights<'rope> {
    /// Creates a new [`Highlights`] for `rope` with `root`, clamped by `lines`, for `query`.
    pub fn new(rope: &'rope Rope, root: Node, lines: Range<usize>, query: &Query) -> Self {
        let Range { start, end } = Self::range(rope, lines);

        Self {
            rope,
            start,
            end,
            captures: Self::captures(rope, root, start, end, query),
        }
    }

    fn range(rope: &Rope, Range { start, end }: Range<usize>) -> Range<Cursor> {
        let lines = rope.len_lines();
        let end = end.min(lines);
        let start = start.min(end);

        let start = Cursor::new(rope.line_to_byte(start), start, 0);
        let end = if let Some(line) = end.checked_sub(1) {
            let bytes = rope.len_bytes();
            let start = rope.try_line_to_byte(line).unwrap();

            let col = {
                let end = if line == lines - 1 {
                    bytes
                } else {
                    rope.try_line_to_byte(line + 1).unwrap() - /* \n */ 1
                };
                end - start
            };

            Cursor::new(start + col, line, col)
        } else {
            Cursor::ZERO
        };

        start..end
    }

    /// Use `tree-sitter` to get a sorted list of captures for `query`.
    fn captures(
        rope: &'rope Rope,
        root: Node,
        start: Cursor,
        end: Cursor,
        query: &Query,
    ) -> Vec<Capture> {
        let mut captures = Vec::<Capture>::new();
        let mut cursor = {
            let mut cursor = QueryCursor::new();
            cursor.set_point_range(start.into()..end.into());
            cursor
        };
        let it = cursor
            .matches(query, root, |node: Node| {
                rope.get_byte_slice(node.byte_range())
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
                    key: ThemeKey::new(&query.capture_names()[capture.index as usize]),
                })
            })
            .flatten();

        for capture in it {
            // We want all captures ordered by start index,
            // favoring lower pattern index when captured multiple times.
            // This is what `Helix` does, and we use their queries.
            // It seems like patterns are written in that specific order.
            match captures.binary_search_by_key(&capture.start.index, |capture| capture.start.index)
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
                start,
                end,
                pattern: usize::MAX, // Whatever now
                key: ThemeKey::Default,
            }]
        } else {
            captures
        }
    }

    /// Returns an iterator of [`Highlight`]s.
    pub fn highlights(&self) -> impl '_ + Iterator<Item = Highlight> {
        let captures = self.captures.iter();
        let highlights = Self::step1_ensure_line_range(captures, self.start, self.end);
        let highlights = Self::step2_add_inbetweens(highlights, self.start, self.end);
        let highlights = Self::step3_slice_at_line_boundaries(highlights, self.rope);

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

    /// Filter on line range and crop overlapping captures.
    fn step1_ensure_line_range<'a>(
        captures: impl 'a + Iterator<Item = &'a Capture>,
        start: Cursor,
        end: Cursor,
    ) -> impl 'a + Iterator<Item = Highlight> {
        captures
            .map(|capture| Highlight {
                start: capture.start,
                end: capture.end,
                key: capture.key,
            })
            .filter(move |highlight| start.index < highlight.end.index)
            .filter(move |highlight| highlight.start.index < end.index)
            .map(move |highlight| Highlight {
                start: if start.index <= highlight.start.index {
                    highlight.start
                } else {
                    start
                },
                end: if highlight.end.index <= end.index {
                    highlight.end
                } else {
                    end
                },
                key: highlight.key,
            })
    }

    /// Intersperse with in-between selections.
    fn step2_add_inbetweens(
        highlights: impl Iterator<Item = Highlight>,
        start: Cursor,
        end: Cursor,
    ) -> impl Iterator<Item = Highlight> {
        let mut highlights = highlights.peekable();
        let mut prev = Highlight {
            start,
            end: start,
            key: ThemeKey::default(),
        };

        let mut highlights = std::iter::from_fn(move || {
            let next = highlights.peek()?;

            prev = if prev.end.index == next.start.index {
                highlights.next()?
            } else {
                Highlight {
                    start: prev.end,
                    end: next.start,
                    key: ThemeKey::default(),
                }
            };

            Some(prev)
        });

        // Add last highlight to end of text
        let mut last = None;

        std::iter::from_fn(move || {
            if let Some(highlight) = highlights.next() {
                last = Some(highlight.end);
                Some(highlight)
            } else {
                Some(Highlight {
                    start: last.take()?,
                    end,
                    key: ThemeKey::default(),
                })
            }
        })
    }

    /// Slice highlights to line boundaries.
    fn step3_slice_at_line_boundaries<'a>(
        mut highlights: impl 'a + Iterator<Item = Highlight>,
        rope: &'a Rope,
    ) -> impl 'a + Iterator<Item = Highlight> {
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
                let end = rope.try_line_to_byte(highlight.start.line + 1).unwrap();

                next = Some(Highlight {
                    start: Cursor::new(end, highlight.start.line + 1, 0),
                    end: highlight.end,
                    key: highlight.key,
                });
                Some(Highlight {
                    start: highlight.start,
                    end: Cursor::new(
                        end,
                        highlight.start.line,
                        highlight.start.column + (end - highlight.start.index),
                    ),
                    key: highlight.key,
                })
            }
        })
        .filter(|highlight| highlight.start.index != highlight.end.index)
    }
}
