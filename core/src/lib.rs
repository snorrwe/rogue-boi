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

    let dims = Vec2 { x: 32, y: 32 };
    let data = vec![Stuff::default(); dims.x as usize * dims.y as usize].into_boxed_slice();

    let mut grid = GameGrid { dims, data };
    setup_bounds(&mut world, &mut grid);

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

#[derive(serde::Serialize)]
pub struct GameGrid {
    pub dims: Vec2,
    pub data: Box<[Stuff]>,
}

impl GameGrid {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        0 <= x && 0 <= y && x < self.dims.x && y < self.dims.y
    }

    pub fn at(&self, x: i32, y: i32) -> Option<&Stuff> {
        let w = self.dims.x;
        if !self.contains(x, y) {
            return None;
        }
        Some(&self.data[(y * w + x) as usize])
    }

    pub fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Stuff> {
        let w = self.dims.x;
        if !self.contains(x, y) {
            return None;
        }
        Some(&mut self.data[(y * w + x) as usize])
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Id {
    pub val: u64,
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
        if !self.inputs.is_empty() && self.time > 200 {
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

fn setup_bounds(world: &mut World, grid: &mut GameGrid) {
    let w = grid.dims.x;
    let h = grid.dims.y;

    if w == 0 || h == 0 {
        panic!();
    }

    for y in 0..h {
        insert_wall(0, y, world);
        insert_wall(w - 1, y, world);
    }
    for x in 1..w - 1 {
        insert_wall(x, 0, world);
        insert_wall(x, h - 1, world);
    }
}

fn insert_wall(x: i32, y: i32, w: &mut World) {
    let pos = Vec2::new(x, y);

    let id = w.spawn_entity();
    w.insert(id, StuffTag::Wall);
    w.insert(id, Pos(pos));
    w.insert(id, Icon("delapouite/brick-wall.svg"));
}

fn update_player(inputs: &[InputEvent], player: EntityId, q: Query<Pos>, grid: &mut GameGrid) {
    let mut delta = Vec2::new(0, 0);

    for event in inputs {
        match event {
            InputEvent::KeyDown { key } if key == "w" => delta.y = -1,
            InputEvent::KeyDown { key } if key == "s" => delta.y = 1,
            InputEvent::KeyDown { key } if key == "a" => delta.x = -1,
            InputEvent::KeyDown { key } if key == "d" => delta.x = 1,
            _ => {}
        }
    }

    if delta.x != 0 && delta.y != 0 {
        delta.x = 0;
    }
    if delta.x != 0 || delta.y != 0 {
        let q = q.into_inner();
        let pos = &mut q.get_mut(player).expect("Failed to get player pos").0;

        let new_pos = *pos + delta;
        if let Some(tile) = grid.at(new_pos.x, new_pos.y) {
            match tile.payload {
                StuffPayload::Empty => {
                    // update the grid asap so the monsters will see the updated player position
                    let old_stuff = std::mem::take(grid.at_mut(pos.x, pos.y).unwrap());
                    *grid.at_mut(new_pos.x, new_pos.y).unwrap() = old_stuff;

                    *pos = new_pos;
                }
                StuffPayload::Wall => { /* don't step */ }
                StuffPayload::Player => unreachable!(),
            }
        }
    }
}

fn update_grid(q: Query<(EntityId, Pos, StuffTag)>, grid: &mut GameGrid) {
    let w = grid.dims.x;
    let h = grid.dims.y;
    assert!(w > 0 && h > 0);
    // zero out the map
    for i in 0..w * h {
        grid.data[i as usize] = Default::default();
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
