use tree_sitter::{InputEdit, Point};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Cursor                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `(index , line, column)` cursor.
#[derive(Copy, Clone, Eq, Ord, Default, Debug)]
pub struct Cursor {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl Cursor {
    pub const ZERO: Self = Self {
        index: 0,
        line: 0,
        column: 0,
    };

    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Self {
            index,
            line,
            column,
        }
    }

    pub fn into_input_edit(start: Self, old_end: Self, new_end: Self) -> InputEdit {
        InputEdit {
            start_byte: start.index,
            old_end_byte: old_end.index,
            new_end_byte: new_end.index,
            start_position: start.into(),
            old_end_position: old_end.into(),
            new_end_position: new_end.into(),
        }
    }

    pub fn from_input_edit(
        input_edit: InputEdit,
    ) -> (
        Self, // Start
        Self, // Old end
        Self, // New end
    ) {
        (
            Self::new(
                input_edit.start_byte,
                input_edit.start_position.row,
                input_edit.start_position.column,
            ),
            Self::new(
                input_edit.old_end_byte,
                input_edit.old_end_position.row,
                input_edit.old_end_position.column,
            ),
            Self::new(
                input_edit.new_end_byte,
                input_edit.new_end_position.row,
                input_edit.new_end_position.column,
            ),
        )
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        let index = self.index == other.index;

        debug_assert!(if index {
            self.line == other.line && self.column == other.column
        } else {
            self.line != other.line || self.column != other.column
        });

        index
    }
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let index = self.index.cmp(&other.index);

        debug_assert!({
            use std::cmp::Ordering::*;

            let line = self.line.cmp(&other.line);
            let column = self.column.cmp(&other.column);

            match index {
                Less => match line {
                    Less => true,
                    Equal => column == Less,
                    Greater => false,
                },
                Equal => line == Equal && column == Equal,
                Greater => match line {
                    Less => false,
                    Equal => column == Greater,
                    Greater => true,
                },
            }
        });

        Some(index)
    }
}

impl From<Cursor> for Point {
    fn from(cursor: Cursor) -> Self {
        Self {
            row: cursor.line,
            column: cursor.column,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Rgb                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `RGB` color.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const GREY: Self = Self::grey(127);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn grey(grey: u8) -> Self {
        Self::new(grey, grey, grey)
    }

    pub const fn with_alpha(&self, a: u8) -> Rgba {
        Rgba::new(self.r, self.g, self.b, a)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Rgba                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `RGBA` color.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const GREY: Self = Self::grey(127, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn grey(grey: u8, a: u8) -> Self {
        Self::new(grey, grey, grey, a)
    }

    pub const fn without_alpha(&self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }

    pub fn scale_alpha(&self, factor: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (self.a as f32 * factor as f32 / u8::MAX as f32).round() as u8,
        }
    }

    pub fn over(&self, other: Rgb) -> Rgb {
        let self_r = self.r as f32 / u8::MAX as f32;
        let self_g = self.g as f32 / u8::MAX as f32;
        let self_b = self.b as f32 / u8::MAX as f32;
        let self_a = self.a as f32 / u8::MAX as f32;

        let other_r = other.r as f32 / u8::MAX as f32;
        let other_g = other.g as f32 / u8::MAX as f32;
        let other_b = other.b as f32 / u8::MAX as f32;

        let r = self_r * self_a + other_r * (1. - self_a);
        let g = self_g * self_a + other_g * (1. - self_a);
        let b = self_b * self_a + other_b * (1. - self_a);

        Rgb {
            r: (u8::MAX as f32 * r).round() as u8,
            g: (u8::MAX as f32 * g).round() as u8,
            b: (u8::MAX as f32 * b).round() as u8,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Style                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Style {
    pub foreground: Rgba,
    pub background: Rgba,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}
