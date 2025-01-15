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
}
