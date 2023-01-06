#[derive(Copy, Clone, Debug)]
pub enum Language {
    Rust,
    Yaml,
    Markdown,
}

impl Language {
    pub fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::language()),
            Self::Yaml => None,
            Self::Markdown => None,
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = ();

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        if path.ends_with(".rs") {
            Ok(Self::Rust)
        } else if path.ends_with(".yml") || path.ends_with(".yaml") {
            Ok(Self::Yaml)
        } else if path.ends_with(".md") || path.ends_with(".markdown") {
            Ok(Self::Markdown)
        } else {
            Err(())
        }
    }
}
