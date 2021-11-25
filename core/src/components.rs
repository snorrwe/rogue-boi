use crate::math::Vec2;
use serde::Serialize;

pub struct Pos(pub Vec2);

pub struct Icon(pub &'static str);

#[derive(Debug, Clone, Copy)]
pub struct Ai;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum StuffTag {
    Player,
    Wall,
    Troll,
    Orc,
}

pub const ENEMY_TAGS: &[StuffTag] = &[StuffTag::Troll, StuffTag::Orc];

impl StuffTag {
    pub fn is_opaque(self) -> bool {
        match self {
            StuffTag::Wall => true,
            StuffTag::Player | StuffTag::Troll | StuffTag::Orc => false,
        }
    }

    /// once explored, these stuff remain visible on the screen, even when visibility is obstructed
    pub fn static_visiblity(self) -> bool {
        match self {
            StuffTag::Wall => true,
            StuffTag::Player | StuffTag::Troll | StuffTag::Orc => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hp {
    pub current: i32,
    pub max: i32,
}

impl Hp {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
}
