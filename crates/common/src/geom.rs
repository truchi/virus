use crate::muck;
use std::ops::{Add, BitAnd, Neg, Sub};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Point                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Position => Sint32x2);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Position {
    pub top: i32,
    pub left: i32,
}

impl Position {
    /// Translates the position by `translation`.
    pub fn translate(self, translation: Self) -> Self {
        self + translation
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            top: -self.top,
            left: -self.left,
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            top: self.top + rhs.top,
            left: self.left + rhs.left,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            top: self.top - rhs.top,
            left: self.left - rhs.left,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Size                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Size => Uint32x2);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Crops the size to `max`.
    pub fn crop(self, max: Self) -> Self {
        Self {
            width: self.width.min(max.width),
            height: self.height.min(max.height),
        }
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Rect                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rectangle {
    pub top: i32,
    pub left: i32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Returns the position of the rectangle.
    pub fn position(self) -> Position {
        Position {
            top: self.top,
            left: self.left,
        }
    }

    /// Returns the size of the rectangle.
    pub fn size(self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Returns the bottom coordinate of the rectangle.
    pub fn bottom(self) -> i32 {
        self.top + self.height as i32
    }

    /// Returns the right coordinate of the rectangle.
    pub fn right(self) -> i32 {
        self.left + self.width as i32
    }

    /// Translates the rectangle by `translation`.
    pub fn translate(self, translation: Position) -> Self {
        self + translation
    }

    /// Crops the rectangle to `max`.
    pub fn crop(self, max: Size) -> Self {
        self.position() & self.size().crop(max)
    }

    /// Returns the intersection of `self` and `other`.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let top = self.top.max(other.top);
        let left = self.left.max(other.left);
        let bottom = self.bottom().min(other.bottom());
        let right = self.right().min(other.right());

        if top < bottom && left < right {
            Some(Self {
                top,
                left,
                width: (right - left) as u32,
                height: (bottom - top) as u32,
            })
        } else {
            None
        }
    }

    /// Translates and crops the rectangle to `region`.
    pub fn region(&self, region: Self) -> Option<Self> {
        self.translate(region.position()).intersection(region)
    }
}

impl From<(Position, Size)> for Rectangle {
    fn from((position, size): (Position, Size)) -> Self {
        Self {
            top: position.top,
            left: position.left,
            width: size.width,
            height: size.height,
        }
    }
}

impl BitAnd<Size> for Position {
    type Output = Rectangle;

    fn bitand(self, rhs: Size) -> Self::Output {
        Rectangle::from((self, rhs))
    }
}

impl Add<Position> for Rectangle {
    type Output = Self;

    fn add(self, rhs: Position) -> Self::Output {
        Self {
            top: self.top + rhs.top,
            left: self.left + rhs.left,
            width: self.width,
            height: self.height,
        }
    }
}

impl Sub<Position> for Rectangle {
    type Output = Self;

    fn sub(self, rhs: Position) -> Self::Output {
        Self {
            top: self.top - rhs.top,
            left: self.left - rhs.left,
            width: self.width,
            height: self.height,
        }
    }
}

impl Add<Size> for Rectangle {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            top: self.top,
            left: self.left,
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl Sub<Size> for Rectangle {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        Self {
            top: self.top,
            left: self.left,
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
