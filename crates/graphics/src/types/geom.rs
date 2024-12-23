use std::ops::{Add, Sub};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Position                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Position => Sint32x2);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Position {
    pub top: i32,
    pub left: i32,
}

impl Position {
    /// Creates a new [`Position`].
    pub fn new(top: i32, left: i32) -> Self {
        Self { top, left }
    }

    /// Creates a new [`Position`] from `u32`s.
    pub fn new_u32(top: u32, left: u32) -> Self {
        Self::new(top as i32, left as i32)
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Self::new(self.top + rhs.top, self.left + rhs.left)
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Self::new(self.top - rhs.top, self.left - rhs.left)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Size                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Size => Uint32x2);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Creates a new [`Size`].
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Creates a new [`Size`] from `i32`s.
    pub fn new_i32(width: i32, height: i32) -> Self {
        Self::new(width as u32, height as u32)
    }

    /// Crops the size to `max`.
    pub fn crop(self, max: Self) -> Self {
        Self::new(
            std::cmp::min(self.width, max.width),
            std::cmp::min(self.height, max.height),
        )
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Size) -> Self::Output {
        Self::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Size) -> Self::Output {
        Self::new(self.width - rhs.width, self.height - rhs.height)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Rectangle                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rectangle {
    pub top: i32,
    pub left: i32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Creates a new [`Rectangle`].
    pub fn new(position: Position, size: Size) -> Self {
        Self {
            top: position.top,
            left: position.left,
            width: size.width,
            height: size.height,
        }
    }

    /// Returns the position of the rectangle.
    pub fn position(self) -> Position {
        Position::new(self.top, self.left)
    }

    /// Returns the size of the rectangle.
    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns the bottom coordinate of the rectangle.
    pub fn bottom(self) -> i32 {
        self.top + self.height as i32
    }

    /// Returns the right coordinate of the rectangle.
    pub fn right(self) -> i32 {
        self.left + self.width as i32
    }

    /// Crops the rectangle to `max`.
    pub fn crop(self, max: Size) -> Self {
        Self::new(
            Position::new(self.top, self.left),
            Size::new(self.width, self.height).crop(max),
        )
    }

    /// Returns the intersection of `self` and `other`.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let top = std::cmp::max(self.top, other.top);
        let left = std::cmp::max(self.left, other.left);
        let bottom = std::cmp::min(self.bottom(), other.bottom());
        let right = std::cmp::min(self.right(), other.right());

        if top < bottom && left < right {
            Some(Self::new(
                Position::new(top, left),
                Size::new_i32(right - left, bottom - top),
            ))
        } else {
            None
        }
    }

    /// Translates and crops the rectangle to `region`.
    pub fn region(self, region: Self) -> Option<Self> {
        (self + region.position()).intersection(region)
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl Add<Position> for Rectangle {
    type Output = Rectangle;

    fn add(self, rhs: Position) -> Self::Output {
        Self::new(
            Position::new(self.top, self.left) + rhs,
            Size::new(self.width, self.height),
        )
    }
}

impl Sub<Position> for Rectangle {
    type Output = Rectangle;

    fn sub(self, rhs: Position) -> Self::Output {
        Self::new(
            Position::new(self.top, self.left) - rhs,
            Size::new(self.width, self.height),
        )
    }
}

impl Add<Size> for Rectangle {
    type Output = Rectangle;

    fn add(self, rhs: Size) -> Self::Output {
        Self::new(
            Position::new(self.top, self.left),
            Size::new(self.width, self.height) + rhs,
        )
    }
}

impl Sub<Size> for Rectangle {
    type Output = Rectangle;

    fn sub(self, rhs: Size) -> Self::Output {
        Self::new(
            Position::new(self.top, self.left),
            Size::new(self.width, self.height) - rhs,
        )
    }
}
