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
    pub fn new(top: i32, left: i32, width: u32, height: u32) -> Self {
        Self {
            top,
            left,
            width,
            height,
        }
    }

    /// Returns the top coordinate of the rectangle.
    pub fn top(self) -> i32 {
        self.top
    }

    /// Returns the bottom coordinate of the rectangle.
    pub fn bottom(self) -> i32 {
        self.top + self.height as i32
    }

    /// Returns the left coordinate of the rectangle.
    pub fn left(self) -> i32 {
        self.left
    }

    /// Returns the right coordinate of the rectangle.
    pub fn right(self) -> i32 {
        self.left + self.width as i32
    }

    /// Returns the width of the rectangle.
    pub fn width(self) -> u32 {
        self.width
    }

    /// Returns the height of the rectangle.
    pub fn height(self) -> u32 {
        self.height
    }

    /// Returns the position of the rectangle.
    pub fn position(self) -> Position {
        Position::new(self.top, self.left)
    }

    /// Returns the size of the rectangle.
    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns a rectangle with `size` centered in `self`.
    pub fn centered(self, size: Size) -> Self {
        let top = self.top + (self.height as i32 - size.height as i32) / 2;
        let left = self.left + (self.width as i32 - size.width as i32) / 2;

        Self::from((Position::new(top, left), size))
    }

    /// Returns the intersection of `self` and `other`.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let top = std::cmp::max(self.top, other.top);
        let left = std::cmp::max(self.left, other.left);
        let bottom = std::cmp::min(self.bottom(), other.bottom());
        let right = std::cmp::min(self.right(), other.right());

        if top < bottom && left < right {
            Some(Self::from((
                Position::new(top, left),
                Size::new_i32(right - left, bottom - top),
            )))
        } else {
            None
        }
    }

    /// Translates `self` to `other` and returns the intersection.
    pub fn intersection_in(self, other: Self) -> Option<Self> {
        (self + other.position()).intersection(other)
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

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

impl Add<Position> for Rectangle {
    type Output = Rectangle;

    fn add(self, rhs: Position) -> Self::Output {
        Self::from((
            Position::new(self.top, self.left) + rhs,
            Size::new(self.width, self.height),
        ))
    }
}

impl Sub<Position> for Rectangle {
    type Output = Rectangle;

    fn sub(self, rhs: Position) -> Self::Output {
        Self::from((
            Position::new(self.top, self.left) - rhs,
            Size::new(self.width, self.height),
        ))
    }
}

impl Add<Size> for Rectangle {
    type Output = Rectangle;

    fn add(self, rhs: Size) -> Self::Output {
        Self::from((
            Position::new(self.top, self.left),
            Size::new(self.width, self.height) + rhs,
        ))
    }
}

impl Sub<Size> for Rectangle {
    type Output = Rectangle;

    fn sub(self, rhs: Size) -> Self::Output {
        Self::from((
            Position::new(self.top, self.left),
            Size::new(self.width, self.height) - rhs,
        ))
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection() {
        let zero = Rectangle::new(0, 0, 0, 0);
        let one = Rectangle::new(0, 0, 1, 1);

        assert_eq!(zero.intersection(one), None);
        assert_eq!(one.intersection(zero), None);
        assert_eq!(zero.intersection(zero), None);

        let a = Rectangle::new(1, 1, 1, 1);
        let b = Rectangle::new(2, 2, 1, 1);

        assert_eq!(a.intersection(b), None);
        assert_eq!(b.intersection(a), None);
        assert_eq!(a.intersection(a), Some(a));
        assert_eq!(b.intersection(b), Some(b));

        let big = Rectangle::new(10, 20, 80, 90);
        let small = Rectangle::new(50, 40, 20, 10);

        assert_eq!(big.intersection(small), Some(small));
        assert_eq!(small.intersection(big), Some(small));

        let a = Rectangle::new(10, 10, 10, 10);
        let b = Rectangle::new(15, 15, 10, 10);
        let ab = Rectangle::new(15, 15, 5, 5);

        assert_eq!(a.intersection(b), Some(ab));
        assert_eq!(b.intersection(a), Some(ab));
    }
}
