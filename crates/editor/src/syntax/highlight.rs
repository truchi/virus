use crate::{cursor::Cursor, rope::RopeExt, syntax::ThemeKey};
use ropey::Rope;
use std::ops::Range;
use tree_sitter::{Node, Query, QueryCursor};

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

/// Procuces an iterator of [`Highlight`]s.
///
/// Guaranties that `Highlight`s are in the requested line range, do not span multiple lines
/// and are not empty.
#[derive(Clone, Debug)]
pub struct Highlights {
    captures: Vec<Capture>,
}

impl Highlights {
    /// Creates a new [`Highlights`] for `rope` with `root`, clamped by `lines`, with `query`.
    pub fn new(rope: &Rope, root: Node, lines: Range<usize>, query: &Query) -> Self {
        debug_assert!(lines.start <= lines.end);
        debug_assert!(lines.end <= rope.len_lines());

        let (start, end) = (
            rope.cursor().line(lines.start),
            rope.cursor().line(lines.end),
        );
        let captures = Capture::collect(
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
            }),
        );

        Self { captures }
    }

    /// Returns an iterator of [`Highlight`]s.
    ///
    /// Guaranties that highlights:
    /// - do not overlap
    /// - are sorted (in text order)
    /// - are merged (different theme key if adjacent)
    pub fn highlights(&self) -> impl '_ + Iterator<Item = Highlight> {
        let mut highlights = self
            .captures
            .iter()
            .filter(|capture| capture.start < capture.end)
            .map(|capture| Highlight {
                start: capture.start,
                end: capture.end,
                key: capture.key,
            });
        let mut next = highlights.next();

        std::iter::from_fn(move || {
            let current = next.as_mut()?;

            loop {
                match highlights.next() {
                    Some(highlight)
                        if current.end == highlight.start && current.key == highlight.key =>
                    {
                        current.end = highlight.end;
                    }
                    Some(highlight) => return next.replace(highlight),
                    None => return next.take(),
                }
            }
        })
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

/// A `tree-sitter` capture (internal to [`Highlights`]).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Capture {
    start: Cursor,
    end: Cursor,
    pattern: usize,
    key: ThemeKey,
}

impl Capture {
    fn start(&self, start: Cursor) -> Self {
        let mut capture = *self;
        capture.start = start;
        capture
    }

    fn end(&self, end: Cursor) -> Self {
        let mut capture = *self;
        capture.end = end;
        capture
    }

    /// Collects captures (non-overlapping, in text order, favoring lower pattern index).
    fn collect(it: impl IntoIterator<Item = Capture>) -> Vec<Capture> {
        fn insert_at_or_after(captures: &mut Vec<Capture>, mut capture: Capture, mut index: usize) {
            fn insert(
                captures: &mut Vec<Capture>,
                capture: Capture,
                index: usize,
            ) -> (usize, Option<Capture>) {
                debug_assert!(capture.start < capture.end);

                let old = captures[index];
                let new = capture;
                let is_better = new.pattern < old.pattern; // Lower is better

                let (inserts, remainder) = if old.end <= new.start {
                    // old: [---[
                    // new:       [---[
                    (0, Some(new))
                } else {
                    use std::cmp::Ordering;
                    match (old.start.cmp(&new.start), new.end.cmp(&old.end)) {
                        // old: [----------------[
                        // new:    [--------[
                        (Ordering::Less, Ordering::Less) => {
                            if is_better {
                                let (one, two, three) =
                                    (old.end(new.start), new, old.start(new.end));

                                captures[index] = three;
                                captures.insert(index, two);
                                captures.insert(index, one);

                                (2, None)
                            } else {
                                (0, None)
                            }
                        }
                        // old: [----------------[
                        // new:    [-------------[
                        (Ordering::Less, Ordering::Equal) => {
                            if is_better {
                                let (one, two) = (old.end(new.start), new);

                                captures[index] = two;
                                captures.insert(index, one);

                                (1, None)
                            } else {
                                (0, None)
                            }
                        }
                        // old: [----------------[
                        // new:    [----------------[
                        (Ordering::Less, Ordering::Greater) => {
                            let (capture, remainder) = (new.end(old.end), new.start(old.end));

                            let (inserts, new_remainder) = insert(captures, capture, index);
                            debug_assert!(new_remainder.is_none());

                            (inserts, Some(remainder))
                        }
                        // old: [----------------[
                        // new: [-----------[
                        (Ordering::Equal, Ordering::Less) => {
                            if is_better {
                                let (one, two) = (new, old.start(new.end));

                                captures[index] = two;
                                captures.insert(index, one);

                                (1, None)
                            } else {
                                (0, None)
                            }
                        }
                        // old: [----------------[
                        // new: [----------------[
                        (Ordering::Equal, Ordering::Equal) => {
                            if is_better {
                                captures[index] = new;
                            }

                            (0, None)
                        }
                        // old: [----------------[
                        // new: [-------------------[
                        (Ordering::Equal, Ordering::Greater) => {
                            let (capture, remainder) = (new.end(old.end), new.start(old.end));

                            let (inserts, new_remainder) = insert(captures, capture, index);
                            debug_assert!(inserts == 0);
                            debug_assert!(new_remainder.is_none());

                            (0, Some(remainder))
                        }
                        // old:    [-----...
                        // new: [--------...
                        (Ordering::Greater, _) => unreachable!("Must start inside index's capture"),
                    }
                };

                if let Some(new) = remainder {
                    let index = index + inserts + 1;

                    match captures.get(index) {
                        // old: [-----..
                        // new:     [-----..
                        Some(old) if old.start < new.start => unreachable!("No overlaps"),
                        // old: [-----..
                        // new: [-----..
                        Some(old) if new.start == old.start => (inserts, Some(new)),
                        // old:     [-----..
                        // new: [-..[
                        Some(old) if new.end <= old.start => {
                            captures.insert(index, new);

                            (inserts + 1, None)
                        }
                        // old:     [-----..
                        // new: [----..
                        Some(old) => {
                            let (one, two) = (new.end(old.start), new.start(old.start));

                            captures.insert(index, one);

                            (inserts + 1, Some(two))
                        }
                        None => {
                            debug_assert!(index == captures.len());
                            captures.push(new);

                            (inserts + 1, None)
                        }
                    }
                } else {
                    (inserts, None)
                }
            }

            loop {
                let (inserts, remainder) = insert(captures, capture, index);
                if let Some(remainder) = remainder {
                    index += inserts + 1;
                    capture = remainder;
                } else {
                    break;
                }
            }
        }

        fn insert_before(captures: &mut Vec<Capture>, capture: Capture, index: usize) {
            match index.checked_sub(1).map(|index| captures[index]) {
                // prev   :     [-----..
                // capture: [----..
                Some(prev) if capture.start <= prev.start => unreachable!("Binary search"),
                // prev   :     [-----..
                // capture:     [..-----
                Some(_) => insert_at_or_after(captures, capture, index - 1),
                None => match captures.get(index) {
                    // next   : [----..
                    // capture:     [-----..
                    Some(next) if next.start <= capture.start => unreachable!("Binary search"),
                    // next   :     [-----..
                    // capture: [-..[
                    Some(next) if capture.end <= next.start => captures.insert(index, capture),
                    // next   :     [-----..
                    // capture: [----..
                    Some(next) => {
                        let (one, two) = (capture.end(next.start), capture.start(next.start));
                        captures.insert(index, one);
                        insert_at_or_after(captures, two, index + 1);
                    }
                    // First insert!
                    None => captures.push(capture),
                },
            }
        }

        let mut captures = Vec::<Capture>::new();

        for capture in it {
            match captures.binary_search_by_key(&capture.start.index, |capture| capture.start.index)
            {
                Ok(index) => insert_at_or_after(&mut captures, capture, index),
                Err(index) => insert_before(&mut captures, capture, index),
            }

            debug_assert!(captures.windows(2).all(|captures| {
                captures[0].end <= captures[1].start
                    && captures[0].start < captures[0].end
                    && captures[1].start < captures[1].end
            }));
        }

        captures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let capture = |range: Range<usize>, pattern| Capture {
            start: Cursor::new(range.start, 0, range.start),
            end: Cursor::new(range.end, 0, range.end),
            pattern,
            key: Default::default(),
        };

        let data = [
            (vec![], vec![]),
            (
                vec![capture(0..10, 10), capture(5..15, 9)],
                vec![capture(0..5, 10), capture(5..10, 9), capture(10..15, 9)],
            ),
            (
                vec![capture(0..10, 10), capture(20..30, 10), capture(10..20, 10)],
                vec![capture(0..10, 10), capture(10..20, 10), capture(20..30, 10)],
            ),
            (
                vec![capture(0..10, 10), capture(20..30, 10), capture(5..25, 9)],
                vec![
                    capture(0..5, 10),
                    capture(5..10, 9),
                    capture(10..20, 9),
                    capture(20..25, 9),
                    capture(25..30, 10),
                ],
            ),
            (
                vec![
                    capture(2..14, 10),
                    capture(3..6, 9),
                    capture(5..8, 8),
                    capture(10..13, 7),
                    capture(1..9, 6),
                    capture(12..16, 5),
                ],
                vec![
                    capture(1..2, 6),
                    capture(2..3, 6),
                    capture(3..5, 6),
                    capture(5..6, 6),
                    capture(6..8, 6),
                    capture(8..9, 6),
                    capture(9..10, 10),
                    capture(10..12, 7),
                    capture(12..13, 5),
                    capture(13..14, 5),
                    capture(14..16, 5),
                ],
            ),
        ];

        for (captures, expected) in data {
            assert!(Capture::collect(captures.iter().copied()) == expected);
        }
    }
}
