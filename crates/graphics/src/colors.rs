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
