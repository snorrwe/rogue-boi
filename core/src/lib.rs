mod components;
mod grid;
mod map_gen;
mod math;
mod systems;
mod utils;

use cao_db::prelude::*;
use components::*;
use grid::Grid;
use math::Vec2;

use systems::{update_fov, update_grid, update_player};
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
    grid: Grid<Stuff>,
    visible: Grid<bool>,
    explored: Grid<bool>,
    viewport: Vec2,
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

    let dims = Vec2 { x: 48, y: 32 };
    let mut grid = Grid::new(dims);
    map_gen::generate_map(
        player,
        &mut world,
        &mut grid,
        map_gen::MapGenProps {
            room_min_size: 3,
            room_max_size: 10,
            max_rooms: 50,
        },
    );

    Core {
        viewport: Vec2::new(15, 15),
        world,
        player,
        grid,
        visible: Grid::new(dims),
        explored: Grid::new(dims),
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

/// Id sent to JS
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

#[derive(serde::Serialize)]
pub struct RenderedGrid {
    pub grid: Grid<OutputStuff>,
    pub offset: Vec2,
}

#[derive(serde::Serialize, Default, Clone)]
pub struct OutputStuff {
    pub payload: Option<Stuff>,
    pub visible: bool,
    pub explored: bool,
}

#[wasm_bindgen]
impl Core {
    pub fn init(&mut self) {
        self.tick(0);
        update_fov(
            self.player,
            Query::new(&self.world),
            &self.grid,
            &mut self.explored,
            &mut self.visible,
        );
    }

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
            update_fov(
                self.player,
                Query::new(&self.world),
                &self.grid,
                &mut self.explored,
                &mut self.visible,
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

    pub fn player_pos(&self) -> Vec2 {
        let q: Query<Pos> = Query::new(&self.world);
        q.into_inner().get(self.player).unwrap().0
    }

    #[wasm_bindgen(js_name = "getGrid")]
    pub fn get_grid(&self) -> JsValue {
        let mut result = Grid::new(self.viewport * 2);
        let player_pos = self.player_pos();
        let min = player_pos - self.viewport;
        let max = player_pos + self.viewport;
        for y in min.y.max(0)..max.y.min(self.grid.height()) {
            for x in min.x.max(0)..max.x.min(self.grid.width()) {
                let pos = Vec2::new(x, y);
                let mut output = OutputStuff::default();
                output.explored = self.explored[pos];
                output.visible = self.visible[pos];
                if output.explored {
                    output.payload = self.grid[pos].clone().into();
                }
                result[pos - min] = output;
            }
        }
        let result = RenderedGrid {
            grid: result,
            offset: min,
        };
        JsValue::from_serde(&result).unwrap()
    }

    pub fn width(&self) -> i32 {
        self.grid.width()
    }

    pub fn height(&self) -> i32 {
        self.grid.height()
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
