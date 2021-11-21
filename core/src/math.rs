use std::ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, Default, Debug, PartialEq, Eq, Hash,
)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0, y: 0 };

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// not really normalized, but reduces all dims to [-1‥1]
    pub fn as_direction(self) -> Self {
        let x = if self.x != 0 {
            self.x / self.x.abs()
        } else {
            0
        };
        let y = if self.y != 0 {
            self.y / self.y.abs()
        } else {
            0
        };
        Self::new(x, y)
    }

    pub fn len_sq(self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    pub fn splat(val: i32) -> Self {
        Self::new(val, val)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = self;
        res += rhs;
        res
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = self;
        res -= rhs;
        res
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Index<usize> for Vec2 {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            _ => &self.y,
        }
    }
}

impl IndexMut<usize> for Vec2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }
}
