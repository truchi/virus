mod cursor;
mod next_chars;
mod prev_chars;

pub use cursor::*;
pub use next_chars::*;
pub use prev_chars::*;

use std::{fmt::Write, sync::Arc};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Line                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An immutable, thread-safe line in a [`Text`](crate::text::Text).
///
/// A thread-safe reference-counted `String` of line content (***without newlines***)
/// with a virtual final `\n`.
///
/// ***Do not insert newlines!***
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Line {
    string: Arc<String>,
}

impl Line {
    /// Creates a new [`Line`] from `string`.
    ///
    /// ***Does not check for newlines!***
    pub fn new(string: Arc<String>) -> Self {
        debug_assert!(!string.contains('\n'));
        Self { string }
    }

    /// Returns the byte length of this [`Line`].
    ///
    /// At least `1`, the newline.
    pub fn len(&self) -> usize {
        self.string.len() + /* newline */ 1
    }

    /// Returns wheter this [`Line`] is empty, i.e. `false`.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the `String` of this [`Line`].
    ///
    /// ***Does not include the final newline!***
    pub fn string(&self) -> &Arc<String> {
        &self.string
    }

    /// Gets the strong count to the [`string`](Self::string).
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.string)
    }

    /// Gets the weak count to the [`string`](Self::string).
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.string)
    }

    /// Makes a mutable reference into this [`Line`].
    ///
    /// ***Do not insert newlines!***
    pub fn make_mut(&mut self) -> &mut String {
        Arc::make_mut(&mut self.string)
    }
}

impl From<&str> for Line {
    /// ***Does not check for newlines!***
    fn from(str: &str) -> Self {
        debug_assert!(!str.contains('\n'));
        Self {
            string: Arc::new(str.to_string()),
        }
    }
}

impl AsRef<str> for Line {
    /// ***Does not include the final newline!***
    fn as_ref(&self) -> &str {
        &self.string
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string)?;
        f.write_char('\n')
    }
}
