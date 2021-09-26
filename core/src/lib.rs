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
    PlayerTag
);

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// State object
#[wasm_bindgen]
pub struct Core {
    pub world_diameter: Vec2,
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

    Core {
        world_diameter: Vec2 { x: 32, y: 32 },
        world,
        player,
    }
}

#[derive(serde::Serialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(tag = "ty")]
pub enum Stuff {
    Empty,
    Player,
}

#[derive(serde::Serialize)]
pub struct FrameBuffer {
    pub dims: Vec2,
    pub grid: Box<[Stuff]>,
}

#[wasm_bindgen]
impl Core {
    pub fn tick(&mut self) -> JsValue {
        let mut fb = FrameBuffer {
            dims: self.world_diameter,
            grid: vec![
                Stuff::Empty;
                self.world_diameter.x as usize * self.world_diameter.y as usize
            ]
            .into_boxed_slice(),
        };

        insert_player(self.player, Query::new(&self.world), &mut fb);

        JsValue::from_serde(&fb).unwrap()
    }

    pub fn width(&self) -> i32 {
        self.world_diameter.x
    }

    pub fn height(&self) -> i32 {
        self.world_diameter.y
    }

    pub fn player_id(&self) -> String {
        self.player.to_string()
    }
}

fn insert_player(player: EntityId, q: Query<Pos>, fb: &mut FrameBuffer) {
    let q = q.into_inner();
    let pos = q.get(player).expect("Player has no pos");

    fb.grid[(fb.dims.x * pos.0.y + pos.0.x) as usize] = Stuff::Player;
}
