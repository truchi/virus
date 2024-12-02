#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Select {
    #[default]
    None,
    Range,
    Lines,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Normal { select: Select },
    Insert { select: Select },
    Files,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal {
            select: Default::default(),
        }
    }
}

impl Mode {
    pub fn select(&self) -> Select {
        match self {
            Mode::Normal { select } => *select,
            Mode::Insert { select } => *select,
            Mode::Files => Select::None,
        }
    }
}
