use std::ops::{Index, IndexMut};

use crate::math::Vec2;

#[derive(Clone, serde::Serialize)]
pub struct Grid<T: serde::Serialize> {
    dims: Vec2,
    data: Box<[T]>,
}

impl<T: serde::Serialize> Grid<T> {
    pub fn new(dims: Vec2) -> Self
    where
        T: Default + Clone,
    {
        assert!(dims.x >= 0);
        assert!(dims.y >= 0);
        Self {
            dims,
            data: vec![T::default(); dims.x as usize * dims.y as usize].into_boxed_slice(),
        }
    }

    pub fn dims(&self) -> Vec2 {
        self.dims
    }

    pub fn width(&self) -> i32 {
        self.dims.x
    }

    pub fn height(&self) -> i32 {
        self.dims.y
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        0 <= x && 0 <= y && x < self.dims.x && y < self.dims.y
    }

    pub fn copy(&mut self, src: &Self)
    where
        T: Clone,
    {
        let dims = self.dims;
        assert_eq!(dims, src.dims);

        self.data = src.data.clone();
    }

    pub fn at(&self, x: i32, y: i32) -> Option<&T> {
        if !self.contains(x, y) {
            return None;
        }
        let w = self.dims.x;
        Some(&self.data[(y * w + x) as usize])
    }

    #[allow(unused)]
    pub fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut T> {
        if !self.contains(x, y) {
            return None;
        }
        let w = self.dims.x;
        Some(&mut self.data[(y * w + x) as usize])
    }

    #[allow(unused)]
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        for i in 0..self.dims.x * self.dims.y {
            self.data[i as usize] = value.clone();
        }
    }

    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (Vec2, &T)> {
        let w = self.dims.x;
        let h = self.dims.y;

        (0..h).flat_map(move |y| {
            (0..w).map(move |x| (Vec2::new(x, y), &self.data[(y * w + x) as usize]))
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Vec2, &mut T)> {
        let w = self.dims.x;
        self.data.iter_mut().enumerate().map(move |(i, v)| {
            let x = i as i32 % w;
            let y = i as i32 / w;
            (Vec2::new(x, y), v)
        })
    }

    pub fn or_eq(&mut self, rhs: &Self)
    where
        T: std::ops::BitOrAssign + Copy,
    {
        assert_eq!(self.dims, rhs.dims);
        for (a, b) in self.data.iter_mut().zip(rhs.data.iter()) {
            *a |= *b
        }
    }

    pub fn splat_set(&mut self, [from, to]: [Vec2; 2], value: T)
    where
        T: Clone,
    {
        for y in from.y..to.y {
            for x in from.x..to.x {
                self[Vec2::new(x, y)] = value.clone();
            }
        }
    }
}

impl<T: serde::Serialize> Index<Vec2> for Grid<T> {
    type Output = T;

    fn index(&self, index: Vec2) -> &Self::Output {
        assert!(
            self.contains(index.x, index.y),
            "{} is out of bounds of: {:?}",
            index,
            self.dims
        );
        let w = self.dims.x;
        let Vec2 { x, y } = index;
        &self.data[(y * w + x) as usize]
    }
}

impl<T: serde::Serialize> IndexMut<Vec2> for Grid<T> {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        assert!(
            self.contains(index.x, index.y),
            "{} is out of bounds of: {:?}",
            index,
            self.dims
        );
        let w = self.dims.x;
        let Vec2 { x, y } = index;
        &mut self.data[(y * w + x) as usize]
    }
}
