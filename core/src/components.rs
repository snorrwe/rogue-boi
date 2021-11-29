use crate::math::Vec2;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Pos(pub Vec2);

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Icon(pub &'static str);

lazy_static::lazy_static! {
    pub static ref ICONS: HashMap<&'static str, Icon> = {
        [
            ("wall", Icon("delapouite/brick-wall.svg")),
            ("troll", Icon("skoll/troll.svg")),
            ("orc-head", Icon("delapouite/orc-head.svg")),
            ("person", Icon("delapouite/person.svg")),
            ("tombstone", Icon("lorc/tombstone.svg")),
        ]
            .iter()
            .map(|x|*x)
            .collect()
    };
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy)]
pub struct Ai;

#[derive(Debug, Clone, Copy)]
pub struct Walkable;

#[derive(Debug, Clone, Copy)]
pub struct MeleeAi {
    pub power: i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum StuffTag {
    Player,
    Wall,
    Troll,
    Orc,
}

pub const ENEMY_TAGS: &[StuffTag] = &[StuffTag::Troll, StuffTag::Orc];
pub const ENEMY_WEIGHTS: &[i32] = &[1, 10];

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

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Hp {
    pub current: i32,
    pub max: i32,
}

impl Hp {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
}
