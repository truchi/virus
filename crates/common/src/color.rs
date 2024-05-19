use crate::muck;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Rgb                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An `RGB` color.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
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

    pub const fn transparent(&self, a: u8) -> Rgba {
        Rgba::new(self.r, self.g, self.b, a)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Rgba                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Rgba => Uint8x4);

/// An `RGBA` color.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const RED: Self = Rgb::RED.transparent(255);
    pub const GREEN: Self = Rgb::GREEN.transparent(255);
    pub const BLUE: Self = Rgb::BLUE.transparent(255);
    pub const BLACK: Self = Rgb::BLACK.transparent(255);
    pub const WHITE: Self = Rgb::WHITE.transparent(255);
    pub const GREY: Self = Rgb::GREY.transparent(255);
    pub const TRANSPARENT: Self = Rgb::BLACK.transparent(0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn grey(grey: u8, a: u8) -> Self {
        Self::new(grey, grey, grey, a)
    }

    pub const fn solid(&self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }

    pub fn is_visible(&self) -> bool {
        self.a != 0
    }
}

impl From<Rgba> for [u8; 4] {
    fn from(color: Rgba) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}
