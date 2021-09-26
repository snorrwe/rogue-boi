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
    pub fn tick(&mut self, _dt_ms: i32) {
        update_player(self.player, Query::new(&self.world), &mut self.grid);
        update_grid(Query::new(&self.world), &mut self.grid);
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
    w.insert(id, StuffTag::Wall);
    w.insert(id, Pos(pos));
    w.insert(id, Icon("delapouite/brick-wall.svg"));

    grid.data[(y * width + x) as usize] = Stuff {
        id: Some(Id { val: id.into() }),
        payload: StuffPayload::Wall,
    };
}

fn update_player(_player: EntityId, _q: Query<Pos>, _grid: &mut GameGrid) {}

fn update_grid(q: Query<(EntityId, Pos, StuffTag)>, grid: &mut GameGrid) {
    let w = grid.dims.x;
    let h = grid.dims.y;
    assert!(w > 0 && h > 0);
    // zero out the inner side
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            grid.data[i as usize] = Default::default();
        }
    }

    let q = q.into_inner();
    let it1 = q.1.iter();
    let it2 = q.2.iter();
    for (idx, (pos, tag)) in join!(it1, it2) {
        let tag: &StuffTag = tag;
        let id = q.0.id_at_index(idx);
        let pos = pos.0;

        let payload = match tag {
            StuffTag::Player => StuffPayload::Player,
            StuffTag::Wall => StuffPayload::Wall,
        };

        grid.data[(pos.y * w + pos.x) as usize] = Stuff {
            id: Some(Id { val: id.into() }),
            payload,
        };
    }
}
