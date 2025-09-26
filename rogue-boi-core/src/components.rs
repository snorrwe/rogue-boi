//! Implementation note: if a component is persisted, and stores an ID to another entity, make sure
//! it's remapped when loading! See [[Core::load]]
//!
use std::collections::VecDeque;

use crate::{HashMap, Stuff, grid::Grid, math::Vec2};
use cecs::entity_id::EntityId;
use serde_derive::{Deserialize, Serialize};
use smallvec::SmallVec;
use wasm_bindgen::JsValue;

// reexport generated tags
pub use crate::game_config::StuffTag;

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

impl std::ops::AddAssign for Melee {
    fn add_assign(&mut self, rhs: Self) {
        self.power += rhs.power;
        self.skill += rhs.skill;
    }
}

impl std::ops::SubAssign for Melee {
    fn sub_assign(&mut self, rhs: Self) {
        self.power -= rhs.power;
        self.skill -= rhs.skill;
    }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Hp {
    pub current: i32,
    pub max: i32,
}

impl Hp {
    pub const fn new(max: i32) -> Self {
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

impl Heal {
    pub const fn new(hp: i32) -> Self {
        Self { hp }
    }
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
pub struct Color(pub String);

unsafe impl Send for Color {}
unsafe impl Sync for Color {}

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

#[derive(Clone, Copy, Default)]
pub struct PlayerId(pub Option<EntityId>);

impl PlayerId {
    pub fn get<'a, T, F>(
        self,
        q: &'a cecs::query::Query<T, F>,
    ) -> Option<<T as cecs::query::QueryFragment>::Item<'a>>
    where
        T: cecs::query::QueryFragment,
        F: cecs::prelude::Filter,
    {
        self.0.and_then(|id| q.fetch(id))
    }

    pub fn get_mut<'a, T, F>(
        self,
        q: &'a mut cecs::query::Query<T, F>,
    ) -> Option<<T as cecs::query::QueryFragment>::ItemMut<'a>>
    where
        T: cecs::query::QueryFragment,
        F: cecs::prelude::Filter,
    {
        self.0.and_then(|id| q.fetch_mut(id))
    }
}

#[derive(Default)]
pub struct IconCollection(pub HashMap<&'static str, web_sys::Path2d>);

#[derive(Default)]
pub struct RenderResources {
    pub canvas: Option<web_sys::HtmlCanvasElement>,
    pub ctx: Option<web_sys::CanvasRenderingContext2d>,
    pub width: u32,
    pub height: u32,
}

impl RenderResources {
    pub fn update_dims(&mut self) {
        let dims = self
            .canvas
            .as_ref()
            .map(|canvas| (canvas.width(), canvas.height()));
        let (width, height) = dims.unwrap_or_default();
        self.width = width;
        self.height = height;
    }
}

#[derive(Default)]
pub struct Selected(pub Option<EntityId>);

#[derive(Default)]
pub struct ClickPosition(pub Option<[f64; 2]>);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DungeonFloor {
    pub current: u32,
    pub desired: u32,
}
impl Default for DungeonFloor {
    fn default() -> Self {
        Self {
            current: 0,
            desired: 1,
        }
    }
}

#[derive(Debug, Clone)]
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
    pub items: VecDeque<String>,
    pub capacity: usize,
}

impl LogHistory {
    pub fn push(&mut self, color: &str, line: impl AsRef<str>) {
        self.items.push_back(format!(
            "<span style=\"color:{}\">{}</span>",
            color,
            line.as_ref()
        ));
        while self.items.len() > self.capacity {
            self.items.pop_front();
        }
    }
}

impl Default for LogHistory {
    fn default() -> Self {
        let capacity = 20;
        LogHistory {
            items: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum AppMode {
    Game,
    Targeting,
    TargetingPosition,
    Levelup,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldDims(pub Vec2);

/// Allows going to the next dungeon level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NextLevel;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Level {
    pub current_level: u32,
    pub current_xp: u32,
    pub level_up_base: u32,
    pub level_up_factor: u32,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            current_level: 1,
            current_xp: 0,
            level_up_base: 200,
            level_up_factor: 150,
        }
    }
}

impl Level {
    pub fn experience_to_next_level(self) -> u32 {
        self.level_up_base + self.current_level * self.level_up_factor
    }

    pub fn add_xp(&mut self, exp: u32) {
        self.current_xp += exp;
    }

    pub fn needs_levelup(&self) -> bool {
        let required = self.experience_to_next_level();
        self.current_xp >= required
    }

    pub fn levelup(&mut self) {
        debug_assert!(self.current_xp >= self.experience_to_next_level());
        self.current_xp -= self.experience_to_next_level();
        self.current_level += 1;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Exp {
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum DesiredStat {
    Attack,
    Hp,
    MeleeDefense,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<EntityId>,
    pub armor: Option<EntityId>,
}

impl Equipment {
    pub fn contains(&self, id: EntityId) -> bool {
        self.weapon.map(|i| i == id).unwrap_or(false)
            || self.armor.map(|i| i == id).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defense {
    pub melee_defense: i32,
    pub ward: u8,
}

impl Defense {
    pub const fn new(melee_defense: i32) -> Self {
        Self {
            melee_defense,
            ward: 0,
        }
    }
}

impl std::ops::AddAssign for Defense {
    fn add_assign(&mut self, rhs: Self) {
        self.melee_defense += rhs.melee_defense;
        self.ward = self.ward.saturating_add(rhs.ward);
    }
}

impl std::ops::SubAssign for Defense {
    fn sub_assign(&mut self, rhs: Self) {
        self.melee_defense -= rhs.melee_defense;
        self.ward = self.ward.saturating_sub(rhs.ward);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Opaque;

/// once explored, these stuff remain visible on the screen, even when visibility is obstructed
#[derive(Debug, Clone, Copy)]
pub struct StaticVisibility;

/// Mark this item for use in this tick
#[derive(Debug, Clone, Copy)]
pub struct UseItem;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Poisoned {
    pub duration: i32,
    pub power: i32,
}

/// Mark item to remove
#[derive(Debug, Clone, Copy)]
pub struct ClearInventoryItem;

/// Mark item for consumtion
#[derive(Debug, Clone, Copy)]
pub struct MarkConsume;

#[derive(Debug, Clone, Copy)]
pub struct PoisionAttack;

#[derive(Debug, Clone, Copy)]
pub struct NeedsTargetEntity;

#[derive(Debug, Clone, Copy)]
pub struct NeedsTargetPosition;

#[derive(Debug, Clone, Copy)]
pub struct Targeting(pub EntityId);

#[derive(Debug, Clone, Copy)]
pub struct TargetingPos {
    pub src: Vec2,
    pub dst: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct LightningBolt;

#[derive(Debug, Clone, Copy)]
pub struct FireBall;

#[derive(Debug, Clone, Copy)]
pub struct ConfusionBolt;

#[derive(Debug, Clone, Copy)]
pub struct WardScroll;

/// Mark this item for unequip in this tick
#[derive(Debug, Clone, Copy)]
pub struct Unequip;
