use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Neg, Sub, SubAssign},
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, Default, Debug, PartialEq, Eq, Hash,
)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entry(&self.x).entry(&self.y).finish()
    }
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0, y: 0 };
    pub const ONE: Vec2 = Vec2 { x: 1, y: 1 };
    pub const X: Vec2 = Vec2 { x: 1, y: 0 };
    pub const Y: Vec2 = Vec2 { x: 0, y: 1 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// not really normalized, but reduces all dims to `[-1‥1]`
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

    /// Chebyshev distance from self to the other
    pub fn chebyshev(self, rhs: Vec2) -> i32 {
        (self.x - rhs.x).abs().max((self.y - rhs.y).abs())
    }

    pub fn manhatten(self, rhs: Vec2) -> i32 {
        (self.x - rhs.x).abs() + (self.y - rhs.y).abs()
    }

    pub const fn splat(val: i32) -> Self {
        Self::new(val, val)
    }

    pub fn dot(self, rhs: Vec2) -> i32 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn neighbours(&self) -> [Self; 8] {
        let x = self.x;
        let y = self.y;

        [
            Vec2::new(x - 1, y - 1),
            Vec2::new(x, y - 1),
            Vec2::new(x + 1, y - 1),
            Vec2::new(x - 1, y),
            Vec2::new(x + 1, y),
            Vec2::new(x - 1, y + 1),
            Vec2::new(x, y + 1),
            Vec2::new(x + 1, y + 1),
        ]
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

impl Mul<i32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<i32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
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

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

pub fn walk_square(from: Vec2, to: Vec2) -> impl Iterator<Item = Vec2> {
    debug_assert!(from.x <= to.x);
    debug_assert!(from.y <= to.y);

    let fx = from.x;
    let tx = to.x;
    (from.y..=to.y).flat_map(move |y| (fx..=tx).map(move |x| Vec2::new(x, y)))
}

pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + t * b
}

pub fn inv_lerp_f64(a: f64, b: f64, val: f64) -> f64 {
    (val - a) / (b - a)
}

pub fn remap_f64(from_a: f64, to_a: f64, from_b: f64, to_b: f64, val: f64) -> f64 {
    let t = inv_lerp_f64(from_a, to_a, val);
    lerp_f64(from_b, to_b, t)
}
