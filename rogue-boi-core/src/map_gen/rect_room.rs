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

    /// p : start of ray
    /// d : direction
    ///
    /// Ray = R(t) = p + td
    ///
    /// param t is the intersection point on the ray
    pub fn intersects_ray(&self, p: Vec2, d: Vec2, t: &mut f32) -> bool {
        let tmin = t;
        *tmin = 0.0;
        let mut tmax = f32::MAX;

        let min = self.min;
        let max = self.max;

        for i in 0..2 {
            if d[i] == 0 {
                // parallel to slab, no hit if origin not within slab
                if p[i] < min[i] || max[i] < p[i] {
                    return false;
                }
            }
            let ood = 1.0 / d[i] as f32;
            let mut t1 = (min[i] - p[i]) as f32 * ood;
            let mut t2 = (max[i] - p[i]) as f32 * ood;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            *tmin = tmin.max(t1);
            tmax = tmax.min(t2);

            if tmax < *tmin {
                return false;
            }
        }
        true
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
