use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self::new(
            self.width.saturating_add(rhs.width),
            self.height.saturating_add(rhs.height),
        )
    }

    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self::new(
            self.width.saturating_sub(rhs.width),
            self.height.saturating_sub(rhs.height),
        )
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl SubAssign for Size {
    fn sub_assign(&mut self, rhs: Self) {
        self.width -= rhs.width;
        self.height -= rhs.height;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<Size> for Pos {
    fn from(s: Size) -> Self {
        Self::new(s.width.into(), s.height.into())
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<Size> for Pos {
    type Output = Self;

    fn add(self, rhs: Size) -> Self {
        Self::new(self.x + rhs.width as i32, self.y + rhs.height as i32)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<Size> for Pos {
    fn add_assign(&mut self, rhs: Size) {
        self.x += rhs.width as i32;
        self.y += rhs.height as i32;
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<Size> for Pos {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self {
        Self::new(self.x - rhs.width as i32, self.y - rhs.height as i32)
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<Size> for Pos {
    fn sub_assign(&mut self, rhs: Size) {
        self.x -= rhs.width as i32;
        self.y -= rhs.height as i32;
    }
}

impl Neg for Pos {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}
