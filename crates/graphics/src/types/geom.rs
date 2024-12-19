use std::ops::{Add, Sub};

fn panicking_partial_min<T: PartialOrd>(a: T, b: T) -> T {
    a.partial_cmp(&b).unwrap().is_lt().then_some(a).unwrap_or(b)
}

fn panicking_partial_max<T: PartialOrd>(a: T, b: T) -> T {
    a.partial_cmp(&b).unwrap().is_lt().then_some(b).unwrap_or(a)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Position                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Position => Float32x2);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Position<T = f32> {
    pub top: T,
    pub left: T,
}

impl<T> Position<T> {
    /// Creates a new [`Position`].
    pub fn new(top: T, left: T) -> Self {
        Self { top, left }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<T: Add<U>, U> Add<Position<U>> for Position<T> {
    type Output = Position<<T as Add<U>>::Output>;

    fn add(self, rhs: Position<U>) -> Self::Output {
        Position {
            top: self.top + rhs.top,
            left: self.left + rhs.left,
        }
    }
}

impl<T: Sub<U>, U> Sub<Position<U>> for Position<T> {
    type Output = Position<<T as Sub<U>>::Output>;

    fn sub(self, rhs: Position<U>) -> Self::Output {
        Position {
            top: self.top - rhs.top,
            left: self.left - rhs.left,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Size                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Size => Float32x2);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new [`Size`].
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    /// Crops the size to `max`.
    pub fn crop(self, max: Self) -> Self
    where
        T: PartialOrd,
    {
        Self {
            width: panicking_partial_min(self.width, max.width),
            height: panicking_partial_min(self.height, max.height),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<T: Add<U>, U> Add<Size<U>> for Size<T> {
    type Output = Size<<T as Add<U>>::Output>;

    fn add(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T: Sub<U>, U> Sub<Size<U>> for Size<T> {
    type Output = Size<<T as Sub<U>>::Output>;

    fn sub(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Rectangle                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rectangle<T = f32, U = f32> {
    pub top: T,
    pub left: T,
    pub width: U,
    pub height: U,
}

impl<T, U> Rectangle<T, U> {
    /// Creates a new [`Rectangle`].
    pub fn new(position: Position<T>, size: Size<U>) -> Self {
        Self {
            top: position.top,
            left: position.left,
            width: size.width,
            height: size.height,
        }
    }

    /// Returns the position of the rectangle.
    pub fn position(self) -> Position<T> {
        Position {
            top: self.top,
            left: self.left,
        }
    }

    /// Returns the size of the rectangle.
    pub fn size(self) -> Size<U> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Returns the bottom coordinate of the rectangle.
    pub fn bottom(self) -> T
    where
        T: Add<Output = T>,
        U: TryInto<T>,
    {
        self.top + self.height.try_into().ok().unwrap()
    }

    /// Returns the right coordinate of the rectangle.
    pub fn right(self) -> T
    where
        T: Add<Output = T>,
        U: TryInto<T>,
    {
        self.left + self.width.try_into().ok().unwrap()
    }

    /// Crops the rectangle to `max`.
    pub fn crop(self, max: Size<U>) -> Self
    where
        U: Ord,
    {
        Self::new(
            Position::new(self.top, self.left),
            Size::new(self.width, self.height).crop(max),
        )
    }

    /// Returns the intersection of `self` and `other`.
    pub fn intersection(self, other: Self) -> Option<Rectangle<T, U>>
    where
        T: Copy,
        T: PartialOrd,
        T: Add<Output = T>,
        T: Sub<Output = T>,
        T: TryInto<U>,
        U: Copy,
        U: TryInto<T>,
    {
        let top = panicking_partial_max(self.top, other.top);
        let left = panicking_partial_max(self.left, other.left);
        let bottom = panicking_partial_min(self.bottom(), other.bottom());
        let right = panicking_partial_min(self.right(), other.right());

        if top < bottom && left < right {
            Some(Rectangle {
                top,
                left,
                width: (right - left).try_into().ok().unwrap(),
                height: (bottom - top).try_into().ok().unwrap(),
            })
        } else {
            None
        }
    }

    /// Translates and crops the rectangle to `region`.
    pub fn region(self, region: Self) -> Option<Rectangle<T, U>>
    where
        T: Copy,
        T: PartialOrd,
        T: Add<Output = T>,
        T: Sub<Output = T>,
        T: TryInto<U>,
        U: Copy,
        U: TryInto<T>,
    {
        (self + region.position()).intersection(region)
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<T: Add<V>, U, V> Add<Position<V>> for Rectangle<T, U> {
    type Output = Rectangle<<T as Add<V>>::Output, U>;

    fn add(self, rhs: Position<V>) -> Self::Output {
        Rectangle::new(
            Position {
                top: self.top,
                left: self.left,
            } + rhs,
            Size {
                width: self.width,
                height: self.height,
            },
        )
    }
}

impl<T: Sub<V>, U, V> Sub<Position<V>> for Rectangle<T, U> {
    type Output = Rectangle<<T as Sub<V>>::Output, U>;

    fn sub(self, rhs: Position<V>) -> Self::Output {
        Rectangle::new(
            Position {
                top: self.top,
                left: self.left,
            } - rhs,
            Size {
                width: self.width,
                height: self.height,
            },
        )
    }
}

impl<T, U: Add<V>, V> Add<Size<V>> for Rectangle<T, U> {
    type Output = Rectangle<T, <U as Add<V>>::Output>;

    fn add(self, rhs: Size<V>) -> Self::Output {
        Rectangle::new(
            Position {
                top: self.top,
                left: self.left,
            },
            Size {
                width: self.width,
                height: self.height,
            } + rhs,
        )
    }
}

impl<T, U: Sub<V>, V> Sub<Size<V>> for Rectangle<T, U> {
    type Output = Rectangle<T, <U as Sub<V>>::Output>;

    fn sub(self, rhs: Size<V>) -> Self::Output {
        Rectangle::new(
            Position {
                top: self.top,
                left: self.left,
            },
            Size {
                width: self.width,
                height: self.height,
            } - rhs,
        )
    }
}
