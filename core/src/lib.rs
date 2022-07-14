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

use cao_db::prelude::*;
use components::*;
use grid::Grid;
use icons::ICONS;
use math::Vec2;

use systems::{
    handle_click, handle_player_move, init_player, player_prepare, render_into_canvas, rotate_log,
    should_update_player, should_update_world, update_ai_hp, update_camera_pos, update_fov,
    update_grid, update_output, update_player_inventory, update_player_item_use, update_tick,
};
use tracing::debug;
use wasm_bindgen::{prelude::*, JsCast};

use crate::systems::{update_input_events, update_melee_ai, update_player_hp};

/// State object
#[wasm_bindgen]
pub struct Core {
    world: Pin<Box<World>>,
    time: i32,
}

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::DEBUG)
            .build(),
    );
}

#[wasm_bindgen(js_name = "initCore")]
pub fn init_core() -> Core {
    let world_dims = Vec2 { x: 64, y: 64 };
    let mut world = World::new(world_dims.x as u32 * world_dims.y as u32);

    world.insert_resource(Grid::<Stuff>::new(world_dims));
    world.insert_resource(GameTick(0));
    world.insert_resource(ClickPosition(None));
    world.insert_resource(Selected::default());
    world.insert_resource(map_gen::MapGenProps {
        room_min_size: 6,
        room_max_size: 10,
        max_rooms: 50,
        max_monsters_per_room: 2,
        max_items_per_room: 2,
    });
    world.insert_resource(IconCollection::default());
    world.insert_resource(ShouldUpdateWorld(false));
    world.insert_resource(ShouldUpdatePlayer(false));
    world.insert_resource(Vec::<InputEvent>::with_capacity(16));
    world.insert_resource(PlayerActions::new());
    world.insert_resource(Visible(Grid::new(world_dims)));
    world.insert_resource(Explored(Grid::new(world_dims)));
    world.insert_resource(Viewport(Vec2::new(20, 20)));
    world.insert_resource(Visibility(Vec2::new(10, 10)));
    world.insert_resource(CameraPos(Vec2::ZERO));
    world.insert_resource(Output(JsValue::null()));
    world.insert_resource(RenderResources::default());

    world.add_stage(SystemStage::new("player-update-pre").with_system(player_prepare));
    world.add_stage(
        SystemStage::new("player-update")
            .with_should_run(should_update_player)
            .with_system(update_player_item_use)
            .with_system(handle_player_move)
            .with_system(update_player_inventory),
    );
    world.add_stage(
        SystemStage::new("player-update-post")
            .with_system(update_ai_hp)
            .with_system(update_camera_pos),
    );
    world.add_stage(
        SystemStage::new("ai-update")
            .with_should_run(should_update_world)
            .with_system(update_tick)
            .with_system(update_melee_ai)
            .with_system(update_player_hp),
    );
    world.add_stage(
        SystemStage::new("post-update")
            .with_should_run(should_update_world)
            .with_system(update_grid)
            .with_system(update_fov),
    );
    world.add_stage(
        SystemStage::new("render")
            .with_system(update_output)
            .with_system(render_into_canvas),
    );
    world.add_stage(
        SystemStage::new("post-render")
            .with_should_run(should_update_world)
            .with_system(rotate_log),
    );

    // Initialize the game world
    world.run_system(init_player);
    let player = Query::<EntityId, With<PlayerTag>>::new(&world).one();
    world.insert_resource(PlayerId(player));
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
    pub selected: Option<EntityId>,
    pub player: Option<PlayerOutput>,
    pub log: String,
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

    #[wasm_bindgen(js_name = "getOutput")]
    pub fn get_output(&self) -> JsValue {
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
        debug!("set_target {}", id);
        self.world
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .set_target(id);
    }

    #[wasm_bindgen(js_name = "setSelection")]
    pub fn set_selection(&mut self, id: JsValue) {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        debug!("set_selection {}", id);
        if self.world.is_id_valid(id) {
            self.world.get_resource_mut::<Selected>().unwrap().0 = Some(id);
            self.world.run_system(update_output);
        }
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

    #[wasm_bindgen(js_name = "setCanvas")]
    pub fn set_canvas(&mut self, canvas: Option<web_sys::HtmlCanvasElement>) {
        debug!("Setting canvas to: {:?}", canvas);

        let dims = canvas
            .as_ref()
            .map(|canvas| (canvas.width(), canvas.height()));
        let (width, height) = dims.unwrap_or_default();
        let resources = RenderResources {
            ctx: canvas
                .as_ref()
                .and_then(|c| c.get_context("2d").ok())
                .and_then(|x| x) // unwrap the inner optional
                .and_then(|x| x.dyn_into().ok()),
            canvas,
            width,
            height,
        };

        self.world.insert_resource(resources);
        self.world.run_system(render_into_canvas);
    }

    #[wasm_bindgen(js_name = "setIconPayload")]
    pub fn set_icon_payload(&mut self, key: String, svg_path: String) {
        let collection = self.world.get_resource_mut::<IconCollection>().unwrap();

        let path = web_sys::Path2d::new_with_path_string(&svg_path).unwrap();
        collection.0.insert(key, path);
        self.world.run_system(render_into_canvas);
    }

    /// canvas relative coordinates
    #[wasm_bindgen(js_name = "canvasClicked")]
    pub fn canvas_clicked(&mut self, x: f64, y: f64) {
        debug!("click on {} {}", x, y);
        self.world.insert_resource(ClickPosition(Some([x, y])));
        self.world.run_system(handle_click);
        self.world.run_system(update_output);
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
