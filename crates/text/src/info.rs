use std::ops::{Add, AddAssign};

#[derive(Copy, Clone, Default, Debug)]
pub struct Info {
    pub bytes: usize,
    pub feeds: usize,
}

impl Add for Info {
    type Output = Self;

    fn add(self, info: Self) -> Self::Output {
        Self {
            bytes: self.bytes + info.bytes,
            feeds: self.feeds + info.feeds,
        }
    }
}

impl AddAssign for Info {
    fn add_assign(&mut self, info: Self) {
        *self = *self + info;
    }
}
