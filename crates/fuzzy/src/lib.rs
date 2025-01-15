mod byte;

use byte::*;
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

impl Config {
    pub const FILES: Self = Self {
        hits_bonus: 10,
        accumulated_hits_bonus: 1,
        accumulated_hits_bonus_limit: 10,
        needle_start_bonus: 1,
        haystack_start_bonus: 1,
        uppercase_bonus: 1,
        space_as_separator_malus: 1,
    };

    // TODO
    pub const COMPLETIONS: Self = Self {
        hits_bonus: 10,
        accumulated_hits_bonus: 1,
        accumulated_hits_bonus_limit: 10,
        needle_start_bonus: 1,
        haystack_start_bonus: 1,
        uppercase_bonus: 1,
        space_as_separator_malus: 1,
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Match                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Match {
    /// The index of the corresponding haystack.
    pub index: usize,
    /// The score of the match.
    pub score: isize,
    /// The byte indices of the needle in the haystack.
    pub indices: Vec<Range<usize>>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Fuzzy                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Scratch {
    needle_set: [bool; 256],
    needle_bytes: Vec<NeedleByte>,
    haystack_bytes: Vec<HaystackByte>,
    ranges: Vec<Range<usize>>,
    best_ranges: Vec<Range<usize>>,
}

impl Scratch {
    fn new() -> Self {
        Scratch {
            needle_set: [false; 256],
            needle_bytes: Default::default(),
            haystack_bytes: Default::default(),
            ranges: Default::default(),
            best_ranges: Default::default(),
        }
    }

    fn clear(&mut self) {
        self.needle_set = [false; 256];
        self.needle_bytes.clear();
        self.haystack_bytes.clear();
        self.ranges.clear();
        self.best_ranges.clear();
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct Fuzzy {
    config: Config,
    needle: String,
    haystack: Vec<String>,
    matches: Vec<Match>,
    scratch: Scratch,
}

impl Fuzzy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            needle: Default::default(),
            haystack: Default::default(),
            matches: Default::default(),
            scratch: Scratch::new(),
        }
    }

    pub fn needle(&self) -> &str {
        &self.needle
    }

    pub fn haystack(&self) -> &[String] {
        &self.haystack
    }

    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    pub fn needle_mut(&mut self) -> &mut String {
        &mut self.needle
    }

    pub fn haystack_mut(&mut self) -> &mut Vec<String> {
        &mut self.haystack
    }

    pub fn search(&mut self) {
        if self.needle.is_empty() {
            self.matches = self
                .haystack
                .iter()
                .enumerate()
                .map(|(index, _)| Match {
                    index,
                    score: Default::default(),
                    indices: Default::default(),
                })
                .collect();
        } else {
            self.matches.clear();
            self.scratch.clear();

            Byte::parse(&self.needle, |_, byte, is_start| {
                self.scratch.needle_set[byte.byte as usize] = true;
                self.scratch.needle_bytes.push(NeedleByte {
                    byte: byte.byte,
                    is_start,
                    is_separator: byte.class == Class::Separator,
                    accepted: Default::default(),
                });
            });

            for index in 0..self.haystack.len() {
                self.search_one(index).map(|m| self.matches.push(m));
            }

            self.matches.sort_by_key(|m| -m.score);
        }
    }

    pub fn clear(&mut self) {
        self.needle.clear();
        self.haystack.clear();
        self.matches.clear();
        self.scratch.clear();
    }
}

/// Private.
impl Fuzzy {
    fn search_one(&mut self, index: usize) -> Option<Match> {
        self.scratch.haystack_bytes.clear();

        Byte::parse(&self.haystack[index], |index, byte, is_start| {
            let in_set_0 = self.scratch.needle_set[byte.accepts.0.byte as usize];
            let in_set_1 = matches!(
                byte.accepts.1,
                Some(accept) if self.scratch.needle_set[accept.byte as usize],
            );

            if in_set_0 || in_set_1 {
                self.scratch.haystack_bytes.push(HaystackByte {
                    index,
                    is_start,
                    accepts: byte.accepts,
                });
            }
        });

        let mut best_score = None;

        emit(
            &mut self.scratch.needle_bytes,
            &self.scratch.haystack_bytes,
            |needle_bytes| {
                let score = score(&self.config, needle_bytes, &mut self.scratch.ranges);

                if best_score < Some(score) {
                    best_score = Some(score);
                    self.scratch.best_ranges.clear();
                    self.scratch
                        .best_ranges
                        .extend_from_slice(&self.scratch.ranges);
                }
            },
        );

        best_score.map(|score| Match {
            index,
            score,
            indices: std::mem::take(&mut self.scratch.best_ranges),
        })
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn emit(
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

fn score(config: &Config, needle_bytes: &[NeedleByte], ranges: &mut Vec<Range<usize>>) -> isize {
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
                    config
                        .accumulated_hits_bonus_limit
                        .min(accumulated + config.accumulated_hits_bonus)
                };
                range.end += 1;
            }
            _ => {
                accumulated = 0;
                ranges.push(accepted.index..accepted.index + 1);
            }
        }

        score += accumulated
            + config.hits_bonus
            + config.needle_start_bonus * accepted.needle_start_bonus as isize
            + config.haystack_start_bonus * accepted.haystack_start_bonus as isize
            + config.uppercase_bonus * accepted.uppercase_bonus as isize
            - config.space_as_separator_malus * accepted.space_as_separator_malus as isize;
        prev_is_separator = is_separator;
    }

    score
}
