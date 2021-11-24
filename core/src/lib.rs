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

use crate::systems::update_enemies;

db!(
    module rogue_db
    components
    [
        Pos,
        Icon,
        StuffTag,
        Hp,
        Ai,
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
    output_cache: JsValue,
}

fn init_player(world: &mut World) -> EntityId {
    let player = world.spawn_entity();
    world.insert(player, StuffTag::Player);
    world.insert(player, Pos(Vec2::new(16, 16)));
    world.insert(player, Icon("delapouite/person.svg"));
    world.insert(player, Hp::new(10));

    player
}

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(debug_assertions)]
    {
        utils::set_panic_hook();
    }
    tracing_wasm::set_as_global_default();
}

#[wasm_bindgen(js_name = "initCore")]
pub fn init_core() -> Core {
    let mut world = World::new(500_000);

    let player = init_player(&mut world);

    let dims = Vec2 { x: 64, y: 64 };
    let mut grid = Grid::new(dims);
    map_gen::generate_map(
        player,
        &mut world,
        &mut grid,
        map_gen::MapGenProps {
            room_min_size: 6,
            room_max_size: 10,
            max_rooms: 50,
            max_monsters_per_room: 2,
        },
    );

    let mut core = Core {
        output_cache: JsValue::null(),
        viewport: Vec2::new(15, 15),
        world,
        player,
        grid,
        visible: Grid::new(dims),
        explored: Grid::new(dims),
        inputs: Vec::with_capacity(512),
        time: 0,
    };
    core.init();
    core
}

pub type Stuff = Option<Id>;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "ty")]
pub enum StuffPayload {
    Empty,
    Player { id: Id },
    Wall,
    Troll { id: Id },
    Orc { id: Id },
}

impl StuffPayload {
    pub fn from_world(id: EntityId, world: &World) -> Self {
        let tag = <World as AsQuery<StuffTag>>::as_query(world).get(id);
        match tag {
            None => StuffPayload::Empty,
            Some(StuffTag::Wall) => Self::Wall,
            Some(StuffTag::Player) => Self::Player { id: id.into() },
            Some(StuffTag::Troll) => Self::Troll { id: id.into() },
            Some(StuffTag::Orc) => Self::Orc { id: id.into() },
        }
    }
}

impl Default for StuffPayload {
    fn default() -> Self {
        Self::Empty
    }
}

/// Id sent to JS
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
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

impl<'a> From<&'a Id> for EntityId {
    fn from(id: &'a Id) -> Self {
        (*id).into()
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
    pub visible: bool,
    pub explored: bool,
    pub icon: Option<&'static str>,
    #[serde(flatten)]
    pub payload: StuffPayload,
}

#[wasm_bindgen]
impl Core {
    pub fn init(&mut self) {
        self.tick(0);
        self.update_output();
    }

    pub fn tick(&mut self, dt_ms: i32) {
        self.time += dt_ms;
        // min cooldown
        if self.inputs.is_empty() || self.time < 120 {
            return;
        }
        let _span = tracing::span!(tracing::Level::DEBUG, "game_update").entered();

        // logic update
        update_player(
            self.inputs.as_slice(),
            self.player,
            Query::new(&self.world),
            &mut self.grid,
        );
        update_enemies(self.player, Query::new(&self.world), &mut self.grid);

        self.time = 0;
        self.inputs.clear();
        update_grid(Query::new(&self.world), &mut self.grid);
        update_fov(
            self.player,
            Query::new(&self.world),
            &self.grid,
            &mut self.explored,
            &mut self.visible,
        );

        self.update_output();
    }

    #[wasm_bindgen(js_name = "pushEvent")]
    pub fn push_event(&mut self, event: JsValue) {
        let event: InputEvent = event.into_serde().unwrap();
        self.inputs.push(event);
    }

    fn player_pos(&self) -> Vec2 {
        let q: Query<Pos> = Query::new(&self.world);
        q.into_inner().get(self.player).unwrap().0
    }

    fn update_output(&mut self) {
        let _span = tracing::span!(tracing::Level::DEBUG, "update_output").entered();

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
                    if let Some(id) = self.grid[pos] {
                        let ty = <World as AsQuery<StuffTag>>::as_query(&self.world)
                            .get(id.into())
                            .expect("Failed to get tag of stuff");
                        if output.visible || ty.static_visiblity() {
                            output.payload = StuffPayload::from_world(id.into(), &self.world);
                            output.icon = <World as AsQuery<Icon>>::as_query(&self.world)
                                .get(id.into())
                                .map(|icon| icon.0);
                        }
                    }
                }
                result[pos - min] = output;
            }
        }
        let result = RenderedGrid {
            grid: result,
            offset: min,
        };
        self.output_cache = JsValue::from_serde(&result).unwrap();
    }

    #[wasm_bindgen(js_name = "getGrid")]
    pub fn get_grid(&self) -> JsValue {
        self.output_cache.clone()
    }

    pub fn width(&self) -> i32 {
        self.grid.width()
    }

    pub fn height(&self) -> i32 {
        self.grid.height()
    }
}
