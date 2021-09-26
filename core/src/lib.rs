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
    grid: GameGrid,
    world: World,
    player: EntityId,
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

    let grid = GameGrid { dims, data };

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
        for i in 0..self.grid.dims.x * self.grid.dims.y {
            self.grid.data[i as usize] = Stuff::default();
        }

        insert_player(self.player, Query::new(&self.world), &mut self.grid);

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

fn insert_player(player: EntityId, q: Query<Pos>, grid: &mut GameGrid) {
    let q = q.into_inner();
    let pos = q.get(player).expect("Player has no pos");

    grid.data[(grid.dims.x * pos.0.y + pos.0.x) as usize] = Stuff {
        id: Some(Id { val: player.into() }),
        payload: StuffPayload::Player,
    };
}
