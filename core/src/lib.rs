mod archetypes;
mod components;
mod grid;
mod logging;
mod map_gen;
mod math;
mod pathfinder;
mod systems;
mod utils;

use std::pin::Pin;

use archetypes::init_entity;
use cao_db::prelude::*;
use components::*;
use grid::Grid;
use icons::ICONS;
use math::Vec2;

use systems::{
    rotate_log, should_update, update_ai_hp, update_camera_pos, update_fov, update_grid,
    update_output, update_player, update_tick,
};
use wasm_bindgen::prelude::*;

use crate::systems::{update_input_events, update_melee_ai, update_player_hp};

#[derive(Clone)]
pub struct Visible(pub Grid<bool>);
#[derive(Clone)]
pub struct Explored(pub Grid<bool>);
#[derive(Clone, Copy)]
pub struct PlayerId(pub EntityId);
#[derive(Clone, Copy)]
pub struct ShouldUpdate(pub bool);
#[derive(Clone, Copy)]
pub struct GameTick(pub i32);
#[derive(Clone, Copy)]
pub struct Viewport(pub Vec2);
#[derive(Clone, Copy)]
pub struct CameraPos(pub Vec2);
#[derive(Clone)]
pub struct Output(pub JsValue);

/// State object
#[wasm_bindgen]
pub struct Core {
    world: Pin<Box<World>>,
    time: i32,
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
    let world_dims = Vec2 { x: 64, y: 64 };
    let mut world = World::new(world_dims.x as u32 * world_dims.y as u32);
    let mut grid = Grid::<Stuff>::new(world_dims);

    init_entity(
        world_dims / 2,
        StuffTag::Player,
        &mut Commands::new(&world),
        &mut grid,
    );
    world.insert_resource(grid);
    world.apply_commands().unwrap();

    let (player, _) = Query::<(EntityId, &PlayerTag)>::new(&world).one();

    world.insert_resource(map_gen::MapGenProps {
        room_min_size: 6,
        room_max_size: 10,
        max_rooms: 50,
        max_monsters_per_room: 2,
        max_items_per_room: 2,
    });
    world.insert_resource(GameTick(0));
    world.insert_resource(ShouldUpdate(false));
    world.insert_resource(PlayerId(player));
    world.insert_resource(Vec::<InputEvent>::with_capacity(16));
    world.insert_resource(PlayerActions::new());
    world.insert_resource(Visible(Grid::new(world_dims)));
    world.insert_resource(Explored(Grid::new(world_dims)));
    world.insert_resource(Viewport(Vec2::new(10, 10)));
    world.insert_resource(CameraPos(Vec2::ZERO));
    world.insert_resource(Output(JsValue::null()));

    world.add_stage(
        SystemStage::new("player-update")
            .with_system(update_player)
            .with_system(update_ai_hp)
            .with_system(update_camera_pos),
    );
    world.add_stage(
        SystemStage::new("ai-update")
            .with_should_run(should_update)
            .with_system(update_tick)
            .with_system(update_melee_ai)
            .with_system(update_player_hp),
    );
    world.add_stage(
        SystemStage::new("post-processing")
            .with_should_run(should_update)
            .with_system(update_grid)
            .with_system(update_fov)
            .with_system(rotate_log),
    );
    world.add_stage(SystemStage::new("render").with_system(update_output));

    // run initial update
    world.run_system(map_gen::generate_map);
    world.run_system(update_camera_pos);
    world.run_stage(
        SystemStage::new("initial-post-process")
            .with_system(update_grid)
            .with_system(update_fov),
    );
    world.run_system(update_output);

    let mut core = Core { world, time: 0 };
    core.init();
    core
}

pub type Stuff = Option<EntityId>;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "ty")]
pub enum StuffPayload {
    Empty,
    Wall,
    Player { id: EntityId },
    Troll { id: EntityId },
    Orc { id: EntityId },
    Item { id: EntityId },
}

impl StuffPayload {
    pub fn new(id: EntityId, tag: Option<StuffTag>) -> Self {
        match tag {
            None => StuffPayload::Empty,
            Some(StuffTag::Wall) => Self::Wall,
            Some(StuffTag::Player) => Self::Player { id },
            Some(StuffTag::Troll) => Self::Troll { id },
            Some(StuffTag::Orc) => Self::Orc { id },
            Some(StuffTag::HpPotion) | Some(StuffTag::Sword) | Some(StuffTag::LightningScroll) => {
                Self::Item { id }
            }
        }
    }
}

impl Default for StuffPayload {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "ty")]
pub enum InputEvent {
    KeyUp { key: String },
    KeyDown { key: String },
}

#[derive(serde::Serialize)]
pub struct RenderedOutput {
    pub player: Option<PlayerOutput>,
    pub log: String,
    pub grid: Grid<OutputStuff>,
    pub offset: Vec2,
}
#[derive(serde::Serialize)]
pub struct PlayerOutput {
    #[serde(rename = "playerHp")]
    pub player_hp: Hp,
    #[serde(rename = "playerAttack")]
    pub player_attack: i32,
    #[serde(rename = "playerPosition")]
    pub player_pos: Vec2,
}

#[derive(serde::Serialize, Default, Clone)]
pub struct OutputStuff {
    pub visible: bool,
    pub explored: bool,
    pub icon: Option<&'static str>,
    #[serde(flatten)]
    pub payload: StuffPayload,
}

#[derive(Default, Clone)]
pub struct PlayerActions {
    len: usize,
    move_action: Option<Vec2>,
    use_item_action: Option<EntityId>,
    target: Option<EntityId>,
    wait: bool,
}

impl PlayerActions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wait(&self) -> bool {
        self.wait
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.move_action = None;
        self.use_item_action = None;
        self.target = None;
        self.wait = false;
        self.len = 0;
    }

    pub fn insert_move(&mut self, delta: Vec2) {
        let old = self.move_action.replace(delta);
        if old.is_none() {
            self.len += 1;
        }
    }

    pub fn move_action(&self) -> Option<Vec2> {
        self.move_action
    }

    pub fn insert_use_item(&mut self, item: EntityId) {
        let old = self.use_item_action.replace(item);
        if old.is_none() {
            self.len += 1;
        }
    }

    pub fn use_item_action(&self) -> Option<EntityId> {
        self.use_item_action
    }

    pub fn set_target(&mut self, target: EntityId) {
        self.target = Some(target);
    }

    pub fn target(&self) -> Option<EntityId> {
        self.target
    }

    pub fn insert_wait(&mut self) {
        if !self.wait {
            self.len += 1;
        }
        self.wait = true;
    }
}

#[wasm_bindgen]
impl Core {
    pub fn init(&mut self) {
        game_log!("Hello wanderer!");
        self.tick(0);
    }

    /// return the name of the icons (without the extension!)
    pub fn icons(&self) -> JsValue {
        let entries: Vec<_> = ICONS.iter().map(|(k, _x)| k).collect();
        JsValue::from_serde(&entries).unwrap()
    }

    pub fn tick(&mut self, dt_ms: i32) {
        self.time += dt_ms;
        self.world.run_system(update_input_events);

        // min cooldown
        if self
            .world
            .get_resource::<PlayerActions>()
            .unwrap()
            .is_empty()
            || self.time < 120
        {
            return;
        }
        let _span = tracing::span!(tracing::Level::DEBUG, "game_update").entered();

        self.time = 0;
        self.world.tick();

        self.cleanup();
    }

    fn cleanup(&mut self) {
        fn clean(mut inputs: ResMut<Vec<InputEvent>>, mut actions: ResMut<PlayerActions>) {
            inputs.clear();
            actions.clear();
        }

        self.world.run_system(clean);
    }

    #[wasm_bindgen(js_name = "pushEvent")]
    pub fn push_event(&mut self, event: JsValue) {
        let event: InputEvent = event.into_serde().unwrap();
        let mut inputs = ResMut::<Vec<InputEvent>>::new(&self.world);
        inputs.push(event);
    }

    #[wasm_bindgen(js_name = "getGrid")]
    pub fn get_grid(&self) -> JsValue {
        self.world.get_resource::<Output>().unwrap().0.clone()
    }

    #[wasm_bindgen(js_name = "getInventory")]
    pub fn get_inventory(&self) -> JsValue {
        let item_props =
            Query::<(&Icon, &Description, &StuffTag, Option<&Ranged>)>::new(&self.world);
        let inventory = Query::<&Inventory, With<PlayerTag>>::new(&self.world)
            .iter()
            .next()
            .map(|inv| {
                inv.iter()
                    .map(|id| {
                        let props = item_props.fetch(id);
                        ItemDesc {
                            id,
                            description: props.map(|p| p.1 .0.clone()),
                            icon: props.map(|p| p.0 .0.to_string()),
                            usable: props
                                .map(|p| {
                                    matches!(p.2, StuffTag::HpPotion | StuffTag::LightningScroll)
                                })
                                .unwrap_or(false),
                            target_enemy: props
                                .map(|p| matches!(p.2, StuffTag::LightningScroll))
                                .unwrap_or(false),
                            range: props.and_then(|p| p.3).map(|r| r.range).unwrap_or(-1),
                        }
                    })
                    .collect::<Vec<_>>()
            });

        JsValue::from_serde(&inventory).unwrap()
    }

    #[wasm_bindgen(js_name = "useItem")]
    pub fn use_item(&mut self, id: JsValue) {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        self.world
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .insert_use_item(id);
    }

    #[wasm_bindgen]
    pub fn wait(&mut self) {
        self.world
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .insert_wait();
    }

    #[wasm_bindgen(js_name = "setTarget")]
    pub fn set_target(&mut self, id: JsValue) {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        self.world
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .set_target(id);
    }

    #[wasm_bindgen(js_name = "fetchEntity")]
    pub fn fetch_entity(&self, id: JsValue) -> JsValue {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        if !self.world.is_id_valid(id) {
            return JsValue::null();
        }
        let tag = Query::<&StuffTag>::new(&self.world).fetch(id).unwrap();
        archetypes::stuff_to_js(
            id,
            *tag,
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
        )
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemDesc {
    pub id: EntityId,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub usable: bool,
    pub target_enemy: bool,
    pub range: i32,
}
