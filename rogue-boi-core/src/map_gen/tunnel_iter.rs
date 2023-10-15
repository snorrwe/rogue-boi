use crate::math::Vec2;

/// Produces an L shaped tunnel between start and end, bending at corner
pub struct TunnelIter {
    pub current: Vec2,
    pub end: Vec2,
    pub corner: Vec2,
}

impl TunnelIter {
    pub fn new(start: Vec2, end: Vec2, corner: Vec2) -> Self {
        Self {
            current: start,
            end,
            corner,
        }
    }
}

impl Iterator for TunnelIter {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let dc = self.corner - self.current;
        let dc = dc.as_direction();
        let de = self.end - self.current;
        let de = de.as_direction();

        if dc.x == -de.x || dc.y == -de.y {
            // we're between the corner and the end, move towards end
            self.current += de;
        } else {
            // move towards the corner
            self.current += dc;
        }

        Some(self.current)
    }
}
