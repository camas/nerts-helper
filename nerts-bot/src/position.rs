use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn distance(&self, other: Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }

    pub fn within_box(&self, box_origin: Position, box_size: Position) -> bool {
        self.x >= box_origin.x
            && self.x < box_origin.x + box_size.x
            && self.y >= box_origin.y
            && self.y < box_origin.y + box_size.y
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
