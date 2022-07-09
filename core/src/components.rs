use crate::math::Vec2;
use cao_db::entity_id::EntityId;
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
            ("sword", Icon("lorc/pointy-sword.svg")),
            ("hp_potion", Icon("delapouite/health-potion.svg")),
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
pub struct Item;

#[derive(Debug, Clone, Copy)]
pub struct Melee {
    pub power: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Description(pub String);

#[derive(Debug, Clone)]
pub struct Inventory {
    pub capacity: usize,
    pub items: smallvec::SmallVec<[EntityId; 32]>,
}

#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("Inventory is full")]
    Full,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: smallvec::SmallVec::with_capacity(capacity),
        }
    }

    pub fn add(&mut self, item: EntityId) -> Result<(), InventoryError> {
        if self.items.len() >= self.capacity {
            return Err(InventoryError::Full);
        }
        self.items.push(item);
        Ok(())
    }

    pub fn remove(&mut self, item: EntityId) -> Option<EntityId> {
        let (i, _id) = self.items.iter().enumerate().find(|(_i, x)| **x == item)?;
        Some(self.items.remove(i))
    }

    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.items.iter().copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OwnedItem {
    pub owner: EntityId,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum StuffTag {
    Player,
    Wall,
    Troll,
    Orc,
    Sword,
    HpPotion,
}

pub const ENEMY_TAGS: &[StuffTag] = &[StuffTag::Troll, StuffTag::Orc];
pub const ENEMY_WEIGHTS: &[i32] = &[1, 10];

pub const ITEM_TAGS: &[StuffTag] = &[StuffTag::Sword, StuffTag::HpPotion];
pub const ITEM_WEIGHTS: &[i32] = &[1, 2];

impl StuffTag {
    pub fn is_opaque(self) -> bool {
        match self {
            StuffTag::Wall => true,
            StuffTag::Player
            | StuffTag::Troll
            | StuffTag::Orc
            | StuffTag::Sword
            | StuffTag::HpPotion => false,
        }
    }

    /// once explored, these stuff remain visible on the screen, even when visibility is obstructed
    pub fn static_visiblity(self) -> bool {
        match self {
            StuffTag::Wall | StuffTag::Sword | StuffTag::HpPotion => true,
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

    pub fn full(self) -> bool {
        self.current >= self.max
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Heal {
    pub hp: i32,
}
