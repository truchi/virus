use smol_str::{SmolStr, SmolStrBuilder};
use std::ops::Deref;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            SmolCow                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Eq, Debug)]
pub enum SmolCow<'a> {
    Str(&'a str),
    Smol(SmolStr),
}

impl<'a> SmolCow<'a> {
    pub fn builder() -> SmolCowBuilder<'a> {
        SmolCowBuilder::new()
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Str(str) => str,
            Self::Smol(smol) => smol.as_str(),
        }
    }
}

impl<'a> AsRef<str> for SmolCow<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Deref for SmolCow<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> PartialEq for SmolCow<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         SmolCowBuilder                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
enum SmolCowBuilderRepr<'a> {
    Str(&'a str),
    SmolBuilder(SmolStrBuilder),
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Clone, Debug)]
pub struct SmolCowBuilder<'a>(SmolCowBuilderRepr<'a>);

impl<'a> SmolCowBuilder<'a> {
    pub fn new() -> Self {
        Self(SmolCowBuilderRepr::Str(""))
    }

    pub fn append(&mut self, str: &'a str) {
        match &mut self.0 {
            SmolCowBuilderRepr::Str(initial) if initial.is_empty() => *initial = str,
            SmolCowBuilderRepr::Str(initial) => {
                let mut builder = SmolStrBuilder::new();
                builder.push_str(initial);
                builder.push_str(str);
                *self = Self(SmolCowBuilderRepr::SmolBuilder(builder));
            }
            SmolCowBuilderRepr::SmolBuilder(builder) => builder.push_str(str),
        }
    }

    pub fn prepend(&mut self, str: &'a str) {
        match &mut self.0 {
            SmolCowBuilderRepr::Str(initial) if initial.is_empty() => *initial = str,
            SmolCowBuilderRepr::Str(initial) => {
                let mut builder = SmolStrBuilder::new();
                builder.push_str(str);
                builder.push_str(initial);
                *self = Self(SmolCowBuilderRepr::SmolBuilder(builder));
            }
            SmolCowBuilderRepr::SmolBuilder(builder) => {
                // NOTE SmolStrBuilder does not have prepending...
                let initial = builder.finish();
                let mut builder = SmolStrBuilder::new();
                builder.push_str(str);
                builder.push_str(initial.as_str());
                *self = Self(SmolCowBuilderRepr::SmolBuilder(builder));
            }
        }
    }

    pub fn build(self) -> SmolCow<'a> {
        match self.0 {
            SmolCowBuilderRepr::Str(str) => SmolCow::Str(str),
            SmolCowBuilderRepr::SmolBuilder(builder) => SmolCow::Smol(builder.finish()),
        }
    }
}
