use crate::{grid::Grid, math::Vec2};

#[derive(Debug)]
pub struct RectRoom {
    pub(crate) min: Vec2,
    pub(crate) max: Vec2,
}

impl RectRoom {
    pub fn new(x1: i32, y1: i32, width: i32, height: i32) -> Self {
        Self {
            min: Vec2::new(x1, y1),
            max: Vec2::new(x1 + width, y1 + height),
        }
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new((self.max.x + self.min.x) / 2, (self.max.y + self.min.y) / 2)
    }

    /// carve out this room
    pub fn carve<T: Default + serde::Serialize>(&self, grid: &mut Grid<T>) {
        for y in self.min.y..=self.max.y {
            for x in self.min.x..=self.max.x {
                grid[Vec2::new(x, y)] = T::default();
            }
        }
    }

    #[allow(unused)]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn touches(&self, other: &Self) -> bool {
        let d = other.center() - self.center();

        let w = (self.max.x - self.min.x) + (other.max.x - other.min.x);
        let h = (self.max.y - self.min.y) + (other.max.y - other.min.y);

        d.x.abs() < w + 3 && d.y.abs() < h + 3
    }

    pub fn contains_point(&self, p: Vec2) -> bool {
        self.min.x <= p.x && p.x <= self.max.x && self.min.y <= p.y && p.y <= self.max.y
    }

    pub fn touches_point(&self, p: Vec2) -> bool {
        self.min.x - 1 <= p.x
            && p.x <= self.max.x + 1
            && self.min.y - 1 <= p.y
            && p.y <= self.max.y + 1
    }
}
