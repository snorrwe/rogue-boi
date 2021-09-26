use crate::{grid::GameGrid, math::Vec2, Stuff};

pub struct RectRoom {
    pub(crate) x1: i32,
    pub(crate) y1: i32,
    pub(crate) x2: i32,
    pub(crate) y2: i32,
}

impl RectRoom {
    pub fn new(x1: i32, y1: i32, width: i32, height: i32) -> Self {
        Self {
            x1,
            y1,
            x2: x1 + width,
            y2: y1 + height,
        }
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new((self.x2 + self.x1) / 2, (self.y2 + self.y1) / 2)
    }

    /// carve out this room
    pub fn carve(&self, grid: &mut GameGrid) {
        for y in self.y1..=self.y2 {
            for x in self.x1..=self.x2 {
                grid[Vec2::new(x, y)] = Stuff::default();
            }
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
}
