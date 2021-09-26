use crate::math::Vec2;

pub struct Pos(pub Vec2);
pub struct Icon(pub &'static str);
pub enum StuffTag {
    Player,
    Wall,
}
