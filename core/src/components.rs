//! Implementation note: if a component is persisted, and stores an ID to another entity, make sure
//! it's remapped when loading! See [[Core::load]]
//!
use std::collections::{HashMap, VecDeque};

use crate::{grid::Grid, math::Vec2, Stuff};
use cecs::entity_id::EntityId;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use wasm_bindgen::JsValue;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pos(pub Vec2);

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Icon(pub &'static str);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ai;

#[derive(Debug, Clone, Copy)]
pub struct Walkable;

#[derive(Debug, Clone, Copy)]
pub struct Item;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Melee {
    pub power: i32,
    pub skill: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub capacity: usize,
    pub items: Vec<EntityId>,
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
            items: Vec::with_capacity(capacity),
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum StuffTag {
    Player,
    Wall,
    Troll,
    Orc,
    Sword,
    HpPotion,
    LightningScroll,
    ConfusionScroll,
    FireBallScroll,
}

impl StuffTag {
    pub fn is_opaque(self) -> bool {
        matches!(self, StuffTag::Wall)
    }

    /// once explored, these stuff remain visible on the screen, even when visibility is obstructed
    pub fn static_visiblity(self) -> bool {
        matches!(
            self,
            StuffTag::Wall
                | StuffTag::Sword
                | StuffTag::HpPotion
                | StuffTag::LightningScroll
                | StuffTag::ConfusionScroll
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Heal {
    pub hp: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ranged {
    pub power: i32,
    pub range: i32,
    pub skill: i32,
}

#[derive(Debug, Clone, Default)]
pub struct PathCache {
    pub path: SmallVec<[Vec2; 16]>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Leash {
    pub origin: Vec2,
    pub radius: i32,
}

#[derive(Debug, Clone)]
pub struct Color(pub JsValue);

pub struct Visible(pub Grid<bool>);

#[derive(Serialize, Deserialize)]
pub struct Explored(pub Grid<bool>);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct GameTick(pub i32);

impl Default for GameTick {
    fn default() -> Self {
        GameTick(1)
    }
}

#[derive(Clone, Copy)]
pub struct Viewport(pub Vec2);
#[derive(Clone, Copy)]
pub struct CameraPos(pub Vec2);
pub struct Output(pub JsValue);
pub struct Visibility(pub Vec2);
pub struct ShouldUpdateWorld(pub bool);
pub struct ShouldUpdatePlayer(pub bool);

#[derive(Default)]
pub struct IconCollection(pub HashMap<&'static str, web_sys::Path2d>);

#[derive(Default)]
pub struct RenderResources {
    pub canvas: Option<web_sys::HtmlCanvasElement>,
    pub ctx: Option<web_sys::CanvasRenderingContext2d>,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct Selected(pub Option<EntityId>);

#[derive(Default)]
pub struct ClickPosition(pub Option<[f64; 2]>);

#[derive(Clone, Copy)]
pub struct DungeonLevel(pub u32);
impl Default for DungeonLevel {
    fn default() -> Self {
        Self(1)
    }
}

pub struct Name(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LastPos(pub Vec2);

/// Mark entities that can't move
#[derive(Debug, Clone, Copy)]
pub struct StaticStuff;

/// Holds the static stuff
pub struct StaticGrid(pub Grid<Stuff>);

#[derive(Debug, Clone, Copy)]
pub struct BounceOffTime(pub i32);

#[derive(Default, Debug, Clone, Copy)]
pub struct ShouldTick(pub bool);

#[derive(Debug, Clone, Copy)]
pub struct DeltaTime(pub i32);

/// Number of real-world milliseconds a tick should take
#[derive(Debug, Clone, Copy)]
pub struct TickInMs(pub i32);

#[derive(Serialize, Deserialize)]
pub struct LogHistory {
    pub items: VecDeque<(GameTick, String)>,
    pub capacity: usize,
}

impl Default for LogHistory {
    fn default() -> Self {
        let capacity = 10;
        LogHistory {
            items: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Game,
    Targeting,
    TargetingPosition,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConfusedAi {
    pub duration: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Aoe {
    pub radius: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetPos {
    pub pos: Option<Vec2>,
}

#[derive(Debug, Clone, Copy)]
pub struct DropItem(pub EntityId);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldDims(pub Vec2);
