use std::ops::{Index, IndexMut};

use crate::{math::Vec2, Stuff};

#[derive(serde::Serialize)]
pub struct GameGrid {
    pub dims: Vec2,
    pub data: Box<[Stuff]>,
}

impl GameGrid {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        0 <= x && 0 <= y && x < self.dims.x && y < self.dims.y
    }

    pub fn at(&self, x: i32, y: i32) -> Option<&Stuff> {
        let w = self.dims.x;
        if !self.contains(x, y) {
            return None;
        }
        Some(&self.data[(y * w + x) as usize])
    }

    #[allow(unused)]
    pub fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Stuff> {
        let w = self.dims.x;
        if !self.contains(x, y) {
            return None;
        }
        Some(&mut self.data[(y * w + x) as usize])
    }

    #[allow(unused)]
    pub fn fill(&mut self, value: Stuff) {
        for i in 0..self.dims.x * self.dims.y {
            self.data[i as usize] = value.clone();
        }
    }
}

impl Index<Vec2> for GameGrid {
    type Output = Stuff;

    fn index(&self, index: Vec2) -> &Self::Output {
        assert!(self.contains(index.x, index.y));
        let w = self.dims.x;
        let Vec2 { x, y } = index;
        &self.data[(y * w + x) as usize]
    }
}

impl IndexMut<Vec2> for GameGrid {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        assert!(self.contains(index.x, index.y));
        let w = self.dims.x;
        let Vec2 { x, y } = index;
        &mut self.data[(y * w + x) as usize]
    }
}
