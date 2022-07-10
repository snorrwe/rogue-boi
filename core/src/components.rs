use crate::math::Vec2;
use cao_db::entity_id::EntityId;
use serde::Serialize;
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Pos(pub Vec2);

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Icon(pub &'static str);

#[derive(Debug, Clone, Copy)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy)]
pub struct Ai;

#[derive(Debug, Clone, Copy)]
pub struct Walkable;

#[derive(Debug, Clone, Copy)]
pub struct Item;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Melee {
    pub power: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Description(pub String);

#[derive(Debug, Clone)]
pub struct Inventory {
    pub capacity: usize,
    pub items: SmallVec<[EntityId; 32]>,
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
            items: SmallVec::with_capacity(capacity),
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
    LightningScroll,
}

impl StuffTag {
    pub fn is_opaque(self) -> bool {
        matches!(self, StuffTag::Wall)
    }

    /// once explored, these stuff remain visible on the screen, even when visibility is obstructed
    pub fn static_visiblity(self) -> bool {
        matches!(
            self,
            StuffTag::Wall | StuffTag::Sword | StuffTag::HpPotion | StuffTag::LightningScroll
        )
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

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Ranged {
    pub power: i32,
    pub range: i32,
}

#[derive(Debug, Clone, Default)]
pub struct PathCache {
    pub path: SmallVec<[Vec2; 16]>,
}
