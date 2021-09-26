use crate::{grid::GameGrid, math::Vec2, Stuff};

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
    pub fn carve(&self, grid: &mut GameGrid) {
        for y in self.min.y..=self.max.y {
            for x in self.min.x..=self.max.x {
                grid[Vec2::new(x, y)] = Stuff::default();
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

        d.x.abs() < w + 1 && d.y.abs() < h + 1
    }
}
