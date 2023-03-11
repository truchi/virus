// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Cursor                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A `(index , line, column)` cursor.
#[derive(Copy, Clone, Default, Debug)]
pub struct Cursor {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl Cursor {
    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Self {
            index,
            line,
            column,
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

    pub fn over(&self, other: Self) -> Self {
        let one = u8::MAX as f32;

        let self_r = self.r as f32;
        let self_b = self.b as f32;
        let self_g = self.g as f32;
        let self_a = self.a as f32;

        let other_r = other.r as f32;
        let other_b = other.b as f32;
        let other_g = other.g as f32;
        let other_a = other.a as f32;

        let a = self_a + other_a * (one - self_a);
        let r = (self_r * self_a + other_r * other_a * (one - self_a)) / a;
        let g = (self_g * self_a + other_g * other_a * (one - self_a)) / a;
        let b = (self_b * self_a + other_b * other_a * (one - self_a)) / a;

        Self {
            r: r.round() as u8,
            g: g.round() as u8,
            b: b.round() as u8,
            a: a.round() as u8,
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
