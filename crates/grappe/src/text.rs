use crate::line::Line;
use std::sync::Arc;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Text                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An immutable, thread-safe [`String`].
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Text {
    lines: Arc<Vec<Line>>,
    len: usize,
}

impl Text {
    /// Creates a new [`Text`] from `lines` and `len`.
    pub fn new(lines: Arc<Vec<Line>>, len: usize) -> Self {
        Self { lines, len }
    }

    /// Returns the byte length of this [`Text`].
    pub fn len(&self) -> usize {
        self.lines.iter().map(Line::len).sum()
    }

    /// Returns `true` if this [`Text`] has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        // Lines are never empty
        self.lines.len() == 0
    }

    /// Returns the [`Line`]s of this [`Text`].
    pub fn lines(&self) -> &Arc<Vec<Line>> {
        &self.lines
    }

    /// Gets the strong count to the [`lines`](Self::lines).
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.lines)
    }

    /// Gets the weak count to the [`lines`](Self::lines).
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.lines)
    }

    /// Makes a mutable reference into this [`Text`].
    pub fn make_mut(&mut self) -> &mut Vec<Line> {
        Arc::make_mut(&mut self.lines)
    }
}

impl From<&str> for Text {
    fn from(str: &str) -> Self {
        Self {
            lines: Arc::new(str.lines().map(Into::into).collect()),
            len: str.len(),
        }
    }
}

impl AsRef<[Line]> for Text {
    fn as_ref(&self) -> &[Line] {
        &self.lines
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.lines.iter() {
            std::fmt::Display::fmt(line, f)?;
        }

        Ok(())
    }
}
