use crate::{game_config::RoomKind, grid::Grid, math::Vec2};

#[derive(Debug)]
pub struct RectRoom {
    pub role: RoomKind,
    pub min: Vec2,
    pub max: Vec2,
}

impl RectRoom {
    pub fn new(role: RoomKind, x1: i32, y1: i32, width: i32, height: i32) -> Self {
        Self {
            role,
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

    pub fn intersects_segment(&self, p: Vec2, q: Vec2) -> bool {
        let c = self.center();
        let e = self.max - c; // halfwidth extents
        let m = (p + q) / 2;
        let d = p - m;
        let m = m - c; // translate to origin

        let abx = d.x.abs();
        if m.x.abs() > e.x + abx {
            return false;
        }
        let aby = d.y.abs();
        m.y.abs() <= e.y + aby
    }

    pub fn touches(&self, other: &Self) -> bool {
        let d = other.center() - self.center();

        let w = (self.max.x - self.min.x) + (other.max.x - other.min.x);
        let h = (self.max.y - self.min.y) + (other.max.y - other.min.y);

        d.x.abs() < w + 3 && d.y.abs() < h + 3
    }
}
