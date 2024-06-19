mod byte;

use byte::{Byte, Class};
use std::ops::Range;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Config                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Default, Debug)]
pub struct Config {
    pub hits_bonus: isize,
    pub accumulated_hits_bonus: isize,
    pub accumulated_hits_bonus_limit: isize,
    pub needle_start_bonus: isize,
    pub haystack_start_bonus: isize,
    pub uppercase_bonus: isize,
    pub space_as_separator_malus: isize,
}

/// Private.
impl Config {
    fn score(&self, needle_bytes: &[NeedleByte], ranges: &mut Vec<Range<usize>>) -> isize {
        ranges.clear();

        let mut score = 0;
        let mut accumulated = 0;
        let mut prev_is_separator = false;

        for needle_byte in needle_bytes {
            let accepted = needle_byte.accepted;
            let is_separator = needle_byte.is_separator;

            match ranges.last_mut() {
                Some(range) if range.end == accepted.index => {
                    accumulated = if is_separator || prev_is_separator {
                        0
                    } else {
                        self.accumulated_hits_bonus_limit
                            .min(accumulated + self.accumulated_hits_bonus)
                    };
                    range.end += 1;
                }
                _ => {
                    accumulated = 0;
                    ranges.push(accepted.index..accepted.index + 1);
                }
            }

            score += accumulated
                + self.hits_bonus
                + self.needle_start_bonus * accepted.needle_start_bonus as isize
                + self.haystack_start_bonus * accepted.haystack_start_bonus as isize
                + self.uppercase_bonus * accepted.uppercase_bonus as isize
                - self.space_as_separator_malus * accepted.space_as_separator_malus as isize;
            prev_is_separator = is_separator;
        }

        score
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Fuzzy                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Fuzzy {
    config: Config,
    needle_set: [bool; 256],
    needle_bytes: Vec<NeedleByte>,
    haystack_bytes: Vec<HaystackByte>,
    ranges: Vec<Range<usize>>,
    best_ranges: Vec<Range<usize>>,
}

impl Fuzzy {
    pub fn new(config: Config, needle: &str) -> Self {
        let mut needle_set = [false; 256];
        let mut needle_bytes = Vec::with_capacity(needle.len());

        Byte::parse(needle, |_, byte, is_start| {
            needle_set[byte.byte as usize] = true;
            needle_bytes.push(NeedleByte {
                byte: byte.byte,
                is_start,
                is_separator: byte.class == Class::Separator,
                accepted: Default::default(),
            });
        });

        Self {
            config,
            needle_set,
            needle_bytes,
            haystack_bytes: Default::default(),
            ranges: Default::default(),
            best_ranges: Default::default(),
        }
    }

    pub fn score(&mut self, haystack: &str) -> Option<(isize, &[Range<usize>])> {
        self.haystack_bytes.clear();

        Byte::parse(haystack, |index, byte, is_start| {
            if self.needle_set[byte.accepts.0.byte as usize]
                || matches!(byte.accepts.1, Some(accept) if self.needle_set[accept.byte as usize])
            {
                self.haystack_bytes.push(HaystackByte {
                    index,
                    is_start,
                    accepts: byte.accepts,
                });
            }
        });

        let mut best_score = None;

        Self::emit_matches(
            &mut self.needle_bytes,
            &self.haystack_bytes,
            |needle_bytes| {
                let score = self.config.score(needle_bytes, &mut self.ranges);

                if best_score < Some(score) {
                    best_score = Some(score);
                    self.best_ranges.clear();
                    self.best_ranges.extend_from_slice(&self.ranges);
                }
            },
        );

        best_score.map(|score| (score, self.best_ranges.as_slice()))
    }
}

/// Private.
impl Fuzzy {
    fn emit_matches(
        needle_bytes: &mut [NeedleByte],
        haystack_bytes: &[HaystackByte],
        mut emit: impl FnMut(&[NeedleByte]),
    ) {
        fn find(
            needle_byte: &NeedleByte,
            haystack_bytes: &[HaystackByte],
            from: usize,
        ) -> Option<Accepted> {
            for haystack_byte in haystack_bytes.iter().skip_while(|byte| byte.index < from) {
                let accept_0 = haystack_byte.accepts.0;
                let accept_1 = haystack_byte.accepts.1;
                let accepted = |accept: Accept| Accepted {
                    index: haystack_byte.index,
                    needle_start_bonus: haystack_byte.is_start && needle_byte.is_start,
                    haystack_start_bonus: haystack_byte.is_start,
                    uppercase_bonus: accept.uppercase_bonus,
                    space_as_separator_malus: accept.space_as_separator_malus,
                };

                if accept_0.byte == needle_byte.byte {
                    return Some(accepted(accept_0));
                } else if let Some(accept_1) = accept_1 {
                    if accept_1.byte == needle_byte.byte {
                        return Some(accepted(accept_1));
                    }
                }
            }

            None
        }

        let mut depth = 0;
        let mut from = 0;

        loop {
            if let Some(needle_byte) = needle_bytes.get(depth) {
                if let Some(accepted) = find(needle_byte, haystack_bytes, from) {
                    needle_bytes[depth].accepted = accepted;
                    depth += 1;
                    from = accepted.index + 1;
                } else if depth == 0 {
                    break;
                } else {
                    depth -= 1;
                    from = needle_bytes[depth].accepted.index + 1;
                }
            } else if depth == 0 {
                break;
            } else {
                emit(needle_bytes);
                depth -= 1;
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
struct NeedleByte {
    byte: u8,
    is_start: bool,
    is_separator: bool,
    accepted: Accepted,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
struct HaystackByte {
    index: usize,
    is_start: bool,
    accepts: (Accept, Option<Accept>),
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
struct Accepted {
    index: usize,
    needle_start_bonus: bool,
    haystack_start_bonus: bool,
    uppercase_bonus: bool,
    space_as_separator_malus: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
struct Accept {
    byte: u8,
    uppercase_bonus: bool,
    space_as_separator_malus: bool,
}
