// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SelectMode {
    Range,
    Line,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Normal { select_mode: Option<SelectMode> },
    Insert,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal {
            select_mode: Default::default(),
        }
    }
}

impl Mode {
    pub fn select_mode(&self) -> Option<SelectMode> {
        match *self {
            Mode::Normal { select_mode } => select_mode,
            Mode::Insert => None,
        }
    }
}
