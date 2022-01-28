use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

pub mod buffer;
pub mod error;
pub mod screen;
pub mod window;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Size {
    width: u16,
    height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

impl From<(u16, u16)> for Size {
    fn from(parts: (u16, u16)) -> Self {
        Size::new(parts.0, parts.1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Pos {
    x: u16,
    y: u16,
}

impl Pos {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_size_into() {
        let value: Size = (16, 16).into();
        assert_eq!(value, Size::new(16, 16));
    }

    #[test]
    pub fn test_size_from() {
        assert_eq!(Size::from((16, 16)), Size::new(16, 16));
    }

    #[test]
    pub fn test_add_for_pos() {
        assert_eq!(Pos::new(5, 5) + Pos::new(6, 8), Pos::new(11, 13));
    }

    #[test]
    pub fn test_addassign_for_pos() {
        let mut v1 = Pos::new(1, 2);
        let v2 = Pos::new(3, 4);
        v1 += v2;
        assert_eq!(v1, Pos::new(4, 6));
    }

    #[test]
    pub fn test_sub_for_pos() {
        assert_eq!(Pos::new(10, 11) - Pos::new(6, 8), Pos::new(4, 3));
    }

    #[test]
    pub fn test_subassign_for_pos() {
        let mut v1 = Pos::new(3, 4);
        let v2 = Pos::new(1, 2);
        v1 -= v2;
        assert_eq!(v1, Pos::new(2, 2));
    }
}
