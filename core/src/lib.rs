mod components;
mod grid;
mod map_gen;
mod math;
mod systems;
mod utils;

use cao_db::prelude::*;
use components::*;
use grid::GameGrid;
use math::Vec2;

use systems::{update_grid, update_player};
use wasm_bindgen::prelude::*;

use rogue_db::{Db as World, Query};

db!(
    module rogue_db
    components
    [
        Pos,
        Icon,
        StuffTag,
    ]
);

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// State object
#[wasm_bindgen]
pub struct Core {
    world: World,
    player: EntityId,
    grid: GameGrid,
    inputs: Vec<InputEvent>,
    time: i32,
}

#[wasm_bindgen(js_name = "initCore")]
pub fn init_core() -> Core {
    #[cfg(debug_assertions)]
    {
        utils::set_panic_hook();
    }
    let mut world = World::new(500_000);
    let player = world.spawn_entity();
    world.insert(player, StuffTag::Player);
    world.insert(player, Pos(Vec2::new(16, 16)));
    world.insert(player, Icon("delapouite/person.svg"));

    let dims = Vec2 { x: 64, y: 32 };
    let data = vec![Stuff::default(); dims.x as usize * dims.y as usize].into_boxed_slice();
    let mut grid = GameGrid { dims, data };

    map_gen::generate_map(
        player,
        &mut world,
        &mut grid,
        map_gen::MapGenProps {
            room_min_size: 4,
            room_max_size: 8,
            max_rooms: 30,
        },
    );

    Core {
        world,
        player,
        grid,
        inputs: Vec::with_capacity(512),
        time: 0,
    }
}

#[derive(Clone, serde::Serialize)]
pub struct Stuff {
    pub id: Option<Id>,
    #[serde(flatten)]
    pub payload: StuffPayload,
}

impl Default for Stuff {
    fn default() -> Self {
        Self {
            id: None,
            payload: StuffPayload::Empty,
        }
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "ty")]
pub enum StuffPayload {
    Empty,
    Player,
    Wall,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Id {
    pub val: u64,
}

impl From<EntityId> for Id {
    fn from(eid: EntityId) -> Self {
        Self { val: eid.into() }
    }
}
impl From<Id> for EntityId {
    fn from(i: Id) -> Self {
        i.val.into()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "ty")]
pub enum InputEvent {
    KeyUp { key: String },
    KeyDown { key: String },
}

#[wasm_bindgen]
impl Core {
    pub fn tick(&mut self, dt_ms: i32) {
        self.time += dt_ms;
        // min cooldown
        if !self.inputs.is_empty() && self.time > 120 {
            // logic update
            update_player(
                self.inputs.as_slice(),
                self.player,
                Query::new(&self.world),
                &mut self.grid,
            );
            self.time = 0;
            self.inputs.clear();
        }
        update_grid(Query::new(&self.world), &mut self.grid);
    }

    #[wasm_bindgen(js_name = "pushEvent")]
    pub fn push_event(&mut self, event: JsValue) {
        let event: InputEvent = event.into_serde().unwrap();
        self.inputs.push(event);
    }

    #[wasm_bindgen(js_name = "getGrid")]
    pub fn get_grid(&self) -> JsValue {
        JsValue::from_serde(&self.grid).unwrap()
    }

    pub fn width(&self) -> i32 {
        self.grid.dims.x
    }

    pub fn height(&self) -> i32 {
        self.grid.dims.y
    }

    pub fn player_id(&self) -> String {
        self.player.to_string()
    }

    pub fn get_icon(&self, id: JsValue) -> Option<String> {
        let id: Id = id.into_serde().unwrap();
        let id: EntityId = id.val.into();
        debug_assert!(self.world.is_valid(id));

        let q = Query::<Icon>::new(&self.world);
        let q = q.into_inner();

        q.get(id).map(|Icon(x)| x.to_string())
    }
}
