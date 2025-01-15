mod byte;
mod class;
mod config;
mod fuzzy;

use byte::*;
use class::*;
use config::*;
use fuzzy::*;

pub use fuzzy::Match;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Search                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Search {
    fuzzy: Fuzzy,
    needle: String,
    haystack: Vec<String>,
    matches: Vec<Match>,
}

impl Search {
    pub fn new(config: Config) -> Self {
        Self {
            fuzzy: Fuzzy::new(config),
            needle: Default::default(),
            haystack: Default::default(),
            matches: Default::default(),
        }
    }

    pub fn new_files() -> Self {
        Self::new(Config::FILES)
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
        self.matches = if self.needle.is_empty() {
            self.haystack
                .iter()
                .enumerate()
                .map(|(index, _)| Match {
                    index,
                    score: Default::default(),
                    indices: Default::default(),
                })
                .collect()
        } else {
            self.fuzzy
                .matches(&self.needle, self.haystack.iter().map(String::as_str))
        };
    }

    pub fn clear(&mut self) {
        self.fuzzy.clear();
        self.needle.clear();
        self.haystack.clear();
        self.matches.clear();
    }
}
