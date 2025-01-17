use crate::{cursor::Cursor, queries::HighlightsTag, smol::SmolCow};
use ropey::Rope;
use std::{cmp::Ordering, ops::Range};
use tree_sitter::{Node, Point, Query, QueryCursor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Highlights                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Highlight {
    start: usize,
    end: usize,
    pattern: usize,
    tag: Option<HighlightsTag>,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

/// Syntax highlighting.                                                                                                           
#[derive(Debug)]
pub struct Highlights {
    rope: Rope,
    start: usize,
    end: usize,
    highlights: Vec<Highlight>,
}

impl Highlights {
    /// Creates a new [`Highlights`] for `rope` and its `root` node with an highlight `query`.                                     
    ///                                                                                                                            
    /// NOTE lower pattern index matches are favored over high pattern index matches.                                              
    pub fn new(rope: &Rope, root: Node, lines: Range<usize>, query: &Query) -> Self {
        debug_assert!(lines.start <= lines.end);
        debug_assert!(lines.end <= rope.len_lines());

        let (start, end) = {
            let cursor = Cursor::build(rope.slice(..));

            (
                cursor.at_column(lines.start, 0),
                // TODO at line end, to not emit the extra line break
                if lines.end == rope.len_lines() {
                    cursor.at_end()
                } else {
                    cursor.at_column(lines.end, 0)
                },
            )
        };
        let mut cursor = {
            let mut cursor = QueryCursor::new();
            cursor.set_point_range(Range {
                start: Point {
                    row: start.line,
                    column: start.column,
                },
                end: Point {
                    row: end.line,
                    column: end.column,
                },
            });
            cursor
        };
        let unordered = cursor
            .matches(query, root, |node: Node| {
                rope.byte_slice(node.byte_range())
                    .chunks()
                    .map(|chunk| chunk.as_bytes())
            })
            .map(|query_match| {
                query_match
                    .captures
                    .into_iter()
                    .map(move |query_capture| Highlight {
                        start: query_capture.node.start_byte(),
                        end: query_capture.node.end_byte(),
                        pattern: query_match.pattern_index,
                        tag: Some(
                            query.capture_names()[query_capture.index as usize]
                                .parse()
                                .expect("Must exist since queries are static"),
                        ),
                    })
            })
            .flatten()
            // Non empty
            .filter(|highlight| highlight.start < highlight.end)
            // Overlapping line range
            .filter(|highlight| start.index < highlight.end)
            .filter(|highlight| highlight.start < end.index)
            // Cropped by line range
            .map(|highlight| Highlight {
                start: highlight.start.max(start.index),
                end: highlight.end.min(end.index),
                ..highlight
            })
            // Shifted relative to line range
            .map(|highlight| Highlight {
                start: highlight.start - start.index,
                end: highlight.end - start.index,
                ..highlight
            });

        Self {
            rope: rope.clone(),
            start: start.index,
            end: end.index,
            highlights: Self::reorder(end.index - start.index, unordered),
        }
    }

    /// Returns an iterator of strings and their highlight tag.                                                                    
    ///                                                                                                                            
    /// The whole line range of text is covered.                                                                                   
    pub fn highlights(&self) -> impl Iterator<Item = (SmolCow, Option<HighlightsTag>)> {
        Self::split(
            self.rope.byte_slice(self.start..self.end).chunks(),
            self.highlights
                .iter()
                .map(|highlight| highlight.end - highlight.start),
        )
        .zip(self.highlights.iter())
        .map(|(str, highlight)| (str, highlight.tag))
    }
}

/// Private.                                                                                                                       
impl Highlights {
    /// Reorders `unordered` highlights so that:                                                                                   
    /// - the full range is covered                                                                                                
    /// - highlights are ordered text-wise                                                                                         
    /// - lower pattern index highlights are favored                                                                               
    fn reorder(len: usize, unordered: impl Iterator<Item = Highlight>) -> Vec<Highlight> {
        let mut highlights = vec![Highlight {
            start: 0,
            end: len,
            pattern: usize::MAX,
            tag: None,
        }];

        for mut highlight in unordered {
            let mut i = highlights
                .binary_search_by(|c| {
                    if c.end < highlight.start {
                        Ordering::Less
                    } else if highlight.start < c.start {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
                .unwrap();

            loop {
                let current = highlights[i];
                let better = highlight.pattern < current.pattern;
                let overlaps = current.end < highlight.end;
                let same_start = current.start == highlight.start;
                let same_end = current.end == highlight.end;

                if better {
                    if !same_start {
                        let current = Highlight {
                            end: highlight.start,
                            ..current
                        };

                        highlights.insert(i, current);
                        i += 1;
                    }

                    if !overlaps {
                        let current = Highlight {
                            start: highlight.end,
                            ..current
                        };

                        highlights[i] = highlight;
                        (!same_end).then(|| highlights.insert(i + 1, current));
                        break;
                    }

                    highlights[i] = Highlight {
                        end: current.end,
                        ..highlight
                    };
                } else if !overlaps {
                    break;
                }

                highlight = Highlight {
                    start: current.end,
                    ..highlight
                };
                i += 1;
            }
        }

        debug_assert_eq!(highlights.first().unwrap().start, 0);
        debug_assert_eq!(highlights.last().unwrap().end, len);
        debug_assert!(highlights
            .iter()
            .all(|highlight| highlight.start < highlight.end));
        debug_assert!(highlights
            .windows(2)
            .all(|window| window[0].end == window[1].start));

        highlights
    }

    /// Splits `chunks` into new chunks whose length is given by `lens`.                                                           
    ///                                                                                                                            
    /// `chunks` and `lens` MUST cover the same range, i.e.:                                                                       
    /// `chunks.map(str::len).sum() == lens.sum()`.                                                                                
    fn split<'a>(
        mut chunks: impl Iterator<Item = &'a str>,
        mut lens: impl Iterator<Item = usize>,
    ) -> impl Iterator<Item = SmolCow<'a>> {
        let mut current_chunk = chunks.next();
        let mut current_len = lens.next();

        std::iter::from_fn(move || {
            let chunk = current_chunk?;
            let len = current_len.expect("More len if more chunk");

            match chunk.len().cmp(&len) {
                Ordering::Less => {
                    let mut smol = SmolCow::builder();
                    let mut take = len;

                    smol.append(chunk);
                    take -= chunk.len();

                    loop {
                        let c = chunks.next().expect("More chunk if more len");

                        match c.len().cmp(&take) {
                            Ordering::Less => {
                                smol.append(c);
                                take -= c.len();
                            }
                            Ordering::Equal => {
                                current_chunk = chunks.next();
                                current_len = lens.next();

                                smol.append(c);
                                return Some(smol.build());
                            }
                            Ordering::Greater => {
                                let (str, c) = c.split_at(take);
                                current_chunk = Some(c);
                                current_len = lens.next();

                                smol.append(str);
                                return Some(smol.build());
                            }
                        }
                    }
                }
                Ordering::Equal => {
                    current_chunk = chunks.next();
                    current_len = lens.next();

                    return Some(SmolCow::Str(chunk));
                }
                Ordering::Greater => {
                    let (str, c) = chunk.split_at(len);
                    current_chunk = Some(c);
                    current_len = lens.next();

                    return Some(SmolCow::Str(str));
                }
            }
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    const A: Option<HighlightsTag> = Some(HighlightsTag::Attribute);
    const B: Option<HighlightsTag> = Some(HighlightsTag::Comment);
    const C: Option<HighlightsTag> = Some(HighlightsTag::Constant);

    fn highlight(range: Range<usize>, pattern: usize, tag: Option<HighlightsTag>) -> Highlight {
        Highlight {
            start: range.start,
            end: range.end,
            pattern,
            tag,
        }
    }

    #[test]
    fn reorder_1() {
        //      Same start               Same start               Same start
        //      Ends before              Same end                 Ends after
        //      |---0B----|              |------0B------|         |--------0B---------|
        //      |------1A------|         |------1A------|         |------1A------|
        // |----|---0B----|-1A-|---------|------0B------|---------|--------0B----|-0B-|----|
        // |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
        // 0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15   16
        assert_eq!(
            Highlights::reorder(
                16,
                [
                    highlight(1..4, 1, A),
                    highlight(6..9, 1, A),
                    highlight(11..14, 1, A),
                    highlight(1..3, 0, B),
                    highlight(6..9, 0, B),
                    highlight(11..15, 0, B),
                ]
                .into_iter(),
            ),
            vec![
                highlight(0..1, usize::MAX, None),
                highlight(1..3, 0, B),
                highlight(3..4, 1, A),
                highlight(4..6, usize::MAX, None),
                highlight(6..9, 0, B),
                highlight(9..11, usize::MAX, None),
                highlight(11..14, 0, B),
                highlight(14..15, 0, B),
                highlight(15..16, usize::MAX, None),
            ]
        );
    }

    #[test]
    fn reorder_2() {
        //           Starts after             Starts after             Starts after
        //           Ends before              Same end                 Ends after
        //           |-0B-|                   |---0B----|              |------0B------|
        //      |------1A------|         |------1A------|         |------1A------|
        // |----|-1A-|-0B-|-1A-|---------|-1A-|---0B----|---------|-1A-|---0B----|-0B-|----|
        // |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
        // 0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15   16
        assert_eq!(
            Highlights::reorder(
                16,
                [
                    highlight(1..4, 1, A),
                    highlight(6..9, 1, A),
                    highlight(11..14, 1, A),
                    highlight(2..3, 0, B),
                    highlight(7..9, 0, B),
                    highlight(12..15, 0, B),
                ]
                .into_iter(),
            ),
            vec![
                highlight(0..1, usize::MAX, None),
                highlight(1..2, 1, A),
                highlight(2..3, 0, B),
                highlight(3..4, 1, A),
                highlight(4..6, usize::MAX, None),
                highlight(6..7, 1, A),
                highlight(7..9, 0, B),
                highlight(9..11, usize::MAX, None),
                highlight(11..12, 1, A),
                highlight(12..14, 0, B),
                highlight(14..15, 0, B),
                highlight(15..16, usize::MAX, None),
            ]
        );
    }

    #[test]
    fn reorder_3() {
        //      Same start               Same start               Same start
        //      Ends before              Same end                 Ends after
        //      |---1B----|              |------1B------|         |--------1B---------|
        //      |------0A------|         |------0A------|         |------0A------|
        // |----|------0A------|---------|------0A------|---------|------0A------|-1B-|----|
        // |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
        // 0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15   16
        assert_eq!(
            Highlights::reorder(
                16,
                [
                    highlight(1..4, 0, A),
                    highlight(6..9, 0, A),
                    highlight(11..14, 0, A),
                    highlight(1..3, 1, B),
                    highlight(6..9, 1, B),
                    highlight(11..15, 1, B),
                ]
                .into_iter(),
            ),
            vec![
                highlight(0..1, usize::MAX, None),
                highlight(1..4, 0, A),
                highlight(4..6, usize::MAX, None),
                highlight(6..9, 0, A),
                highlight(9..11, usize::MAX, None),
                highlight(11..14, 0, A),
                highlight(14..15, 1, B),
                highlight(15..16, usize::MAX, None),
            ]
        );
    }

    #[test]
    fn reorder_4() {
        //           Starts after             Starts after             Starts after
        //           Ends before              Same end                 Ends after
        //           |-1B-|                   |---1B----|              |------1B------|
        //      |------0A------|         |------0A------|         |------0A------|
        // |----|------0A------|---------|------0A------|---------|------0A------|-1B-|----|
        // |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
        // 0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15   16
        assert_eq!(
            Highlights::reorder(
                16,
                [
                    highlight(1..4, 0, A),
                    highlight(6..9, 0, A),
                    highlight(11..14, 0, A),
                    highlight(2..3, 1, B),
                    highlight(7..9, 1, B),
                    highlight(12..15, 1, B),
                ]
                .into_iter(),
            ),
            vec![
                highlight(0..1, usize::MAX, None),
                highlight(1..4, 0, A),
                highlight(4..6, usize::MAX, None),
                highlight(6..9, 0, A),
                highlight(9..11, usize::MAX, None),
                highlight(11..14, 0, A),
                highlight(14..15, 1, B),
                highlight(15..16, usize::MAX, None),
            ]
        );
    }

    #[test]
    fn reorder_5() {
        // |------------------------------------9C-----------------------------------------|
        //           |--------------------------0B--------------------------|
        //      |------1A------|         |------1A------|         |------1A------|
        // |-9C-|-1A-|---0B----|---0B----|------0B------|---0B----|---0B----|-1A-|---9C----|
        // |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
        // 0    1    2    3    4    5    6    7    8    9   10   11   12   13   14   15   16
        assert_eq!(
            Highlights::reorder(
                16,
                [
                    highlight(1..4, 1, A),
                    highlight(6..9, 1, A),
                    highlight(11..14, 1, A),
                    highlight(2..13, 0, B),
                    highlight(0..16, 9, C),
                ]
                .into_iter(),
            ),
            vec![
                highlight(0..1, 9, C),
                highlight(1..2, 1, A),
                highlight(2..4, 0, B),
                highlight(4..6, 0, B),
                highlight(6..9, 0, B),
                highlight(9..11, 0, B),
                highlight(11..13, 0, B),
                highlight(13..14, 1, A),
                highlight(14..16, 9, C),
            ]
        );
    }

    #[test]
    fn split() {
        // 122333444455555666666777777788888888999999999888888887777777666666555554444333221
        // aaaaabbbbbcccccdddddeeeeefffffggggghhhhhiiiiijjjjjkkkkklllllmmmmmnnnnnooooopppppq

        let chunks = [
            "aaaaa", "bbbbb", "ccccc", "ddddd", "eeeee", "fffff", "ggggg", "hhhhh", "iiiii",
            "jjjjj", "kkkkk", "lllll", "mmmmm", "nnnnn", "ooooo", "ppppp", "q",
        ];
        let lens = [1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 3, 2, 1];

        assert_eq!(
            Highlights::split(chunks.into_iter(), lens.into_iter())
                .map(|s| s.as_str().to_string())
                .collect::<Vec<_>>(),
            vec![
                "a",
                "aa",
                "aab",
                "bbbb",
                "ccccc",
                "ddddde",
                "eeeefff",
                "ffgggggh",
                "hhhhiiiii",
                "jjjjjkkk",
                "kklllll",
                "mmmmmn",
                "nnnno",
                "oooo",
                "ppp",
                "pp",
                "q",
            ],
        );
    }
}
