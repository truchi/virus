use tree_sitter::Parser;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Language                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub enum Language {
    Rust,
    Yaml,
    Markdown,
}

impl Language {
    pub fn iter() -> std::array::IntoIter<Self, 3> {
        [Self::Rust, Self::Yaml, Self::Markdown].into_iter()
    }

    pub fn extensions(&self) -> &[&str] {
        match self {
            Self::Rust => &[".rs"],
            Self::Yaml => &[".yml", ".yaml"],
            Self::Markdown => &[".md", ".markdown"],
        }
    }

    pub fn language(&self) -> Option<tree_sitter::Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::language()),
            Self::Yaml => None,
            Self::Markdown => None,
        }
    }

    pub fn parser(&self) -> Option<Parser> {
        self.language().map(|language| {
            let mut parser = Parser::new();
            parser.set_language(&language).unwrap();
            parser
        })
    }
}

impl TryFrom<&str> for Language {
    type Error = ();

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        for language in Self::iter() {
            for extension in language.extensions() {
                if path.ends_with(extension) {
                    return Ok(language);
                }
            }
        }

        Err(())
    }
}
