mod components;
mod math;
mod utils;

use cao_db::prelude::*;
use components::*;
use math::Vec2;

use wasm_bindgen::prelude::*;

use rogue_db::{Db as World, Query};

db!(
    module rogue_db
    components
    Pos,
    PlayerTag,
    WallTag,
    Icon,
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
}

#[wasm_bindgen]
pub fn init_core() -> Core {
    #[cfg(debug_assertions)]
    {
        utils::set_panic_hook();
    }
    let mut world = World::new(500_000);
    let player = world.spawn_entity();
    world.insert(player, Pos(Vec2::new(16, 16)));
    world.insert(player, Icon("delapouite/person.svg"));

    let dims = Vec2 { x: 32, y: 32 };
    let data = vec![Stuff::default(); dims.x as usize * dims.y as usize].into_boxed_slice();

    let mut grid = GameGrid { dims, data };
    setup_bounds(&mut world, &mut grid);

    Core {
        world,
        player,
        grid,
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

#[derive(serde::Serialize)]
pub struct GameGrid {
    pub dims: Vec2,
    pub data: Box<[Stuff]>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Id {
    pub val: u64,
}

#[wasm_bindgen]
impl Core {
    pub fn tick(&mut self) -> JsValue {
        update_player(self.player, Query::new(&self.world), &mut self.grid);

        JsValue::from_serde(self.get_grid()).unwrap()
    }

    fn get_grid(&self) -> &GameGrid {
        &self.grid
    }

    pub fn width(&self) -> i32 {
        self.get_grid().dims.x
    }

    pub fn height(&self) -> i32 {
        self.get_grid().dims.y
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

fn setup_bounds(w: &mut World, grid: &mut GameGrid) {
    let width = grid.dims.x;
    let height = grid.dims.y;

    if width == 0 || height == 0 {
        panic!();
    }

    for y in 0..height {
        insert_wall(width, 0, y, w, grid);
        insert_wall(width, width - 1, y, w, grid);
    }
    for x in 1..width - 1 {
        insert_wall(width, x, 0, w, grid);
        insert_wall(width, x, height - 1, w, grid);
    }
}

fn insert_wall(width: i32, x: i32, y: i32, w: &mut World, grid: &mut GameGrid) {
    let pos = Vec2::new(x, y);

    let id = w.spawn_entity();
    w.insert(id, WallTag);
    w.insert(id, Pos(pos));
    w.insert(id, Icon("delapouite/brick-wall.svg"));

    grid.data[(y * width + x) as usize] = Stuff {
        id: Some(Id { val: id.into() }),
        payload: StuffPayload::Wall,
    };
}

fn update_player(player: EntityId, q: Query<Pos>, grid: &mut GameGrid) {
    let q = q.into_inner();
    let pos = q.get(player).expect("Player has no pos");

    grid.data[(grid.dims.x * pos.0.y + pos.0.x) as usize] = Stuff {
        id: Some(Id { val: player.into() }),
        payload: StuffPayload::Player,
    };
}
