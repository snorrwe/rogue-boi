mod archetypes;
mod components;
mod grid;
mod logging;
mod map_gen;
mod math;
mod pathfinder;
mod systems;
mod utils;

use std::{cell::RefCell, rc::Rc};

use cecs::prelude::*;
use components::*;
use grid::Grid;
use icons::ICONS;
use math::Vec2;

use systems::{
    handle_click, handle_player_move, player_prepare, render_into_canvas, rotate_log,
    should_update_player, should_update_world, update_ai_hp, update_camera_pos, update_fov,
    update_grid, update_output, update_player_inventory, update_player_item_use, update_tick,
};
use tracing::debug;
use wasm_bindgen::{prelude::*, JsCast};

use crate::systems::{update_input_events, update_melee_ai, update_player_hp};

/// State object
#[wasm_bindgen]
pub struct Core {
    world: Rc<RefCell<World>>,
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

fn compute_icons() -> IconCollection {
    let inner = icons::ICONS_SVG
        .iter()
        .map(|(k, svg)| (*k, web_sys::Path2d::new_with_path_string(&svg).unwrap()))
        .collect();
    IconCollection(inner)
}

pub fn init_world(world_dims: Vec2, world: &mut World) {
    let player = Query::<EntityId, With<PlayerTag>>::new(&world)
        .iter()
        .next();
    assert!(
        player.is_none(),
        "re-initializing exiting World will cause inconsistencies"
    );
    world.insert_resource(Grid::<Stuff>::new(world_dims));
    world.insert_resource(GameTick::default());
    world.insert_resource(ClickPosition(None));
    world.insert_resource(Selected::default());
    world.insert_resource(compute_icons());
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
}

fn init_dungeon(world: &mut World) {
    // reset visibility
    world.get_resource_mut::<Visible>().unwrap().0.fill(false);
    world.get_resource_mut::<Explored>().unwrap().0.fill(false);
    let level = world
        .get_resource::<DungeonLevel>()
        .cloned()
        .unwrap_or_default();

    // reset some resources
    world.insert_resource(level);
    world.insert_resource(map_gen::MapGenProps::from_level(level));
    world.insert_resource(PlayerActions::new());

    world.run_system(map_gen::generate_map);
    world.insert_resource(PlayerId(
        Query::<EntityId, With<PlayerTag>>::new(&world).one(),
    ));

    world.run_system(update_camera_pos);
    world.run_stage(
        SystemStage::new("initial-post-process")
            .with_system(update_grid)
            .with_system(update_fov),
    );
    world.run_system(update_output);
}

#[wasm_bindgen(js_name = "initCore")]
pub fn init_core() -> Core {
    let world_dims = Vec2 { x: 64, y: 64 };
    let mut world = World::new(world_dims.x as u32 * world_dims.y as u32);

    init_world(world_dims, &mut world);
    init_dungeon(&mut world);

    let world = Rc::new(RefCell::new(world));
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

    pub fn restart(&mut self) {
        self.time = 0;
        let mut world = self.world.borrow_mut();
        world.insert_resource(GameTick::default());
        world.insert_resource(DungeonLevel::default());
        init_dungeon(&mut world);
        // trigger an initial render, otherwise we'll only see updated output when the player
        // interacts with something
        world.run_system(render_into_canvas);
    }

    /// return the name of the icons (without the extension!)
    pub fn icons(&self) -> JsValue {
        let entries: Vec<_> = ICONS.iter().map(|(k, _x)| k).collect();
        JsValue::from_serde(&entries).unwrap()
    }

    pub fn tick(&mut self, dt_ms: i32) {
        self.time += dt_ms;
        self.world.borrow_mut().run_system(update_input_events);

        // min cooldown
        if self
            .world
            .borrow_mut()
            .get_resource::<PlayerActions>()
            .unwrap()
            .is_empty()
            || self.time < 120
        {
            return;
        }
        self.time = 0;
        self.world.borrow_mut().tick();

        self.cleanup();
    }

    fn cleanup(&mut self) {
        fn clean(mut inputs: ResMut<Vec<InputEvent>>, mut actions: ResMut<PlayerActions>) {
            inputs.clear();
            actions.clear();
        }

        self.world.borrow_mut().run_system(clean);
    }

    #[wasm_bindgen(js_name = "pushEvent")]
    pub fn push_event(&mut self, event: JsValue) {
        let event: InputEvent = event.into_serde().unwrap();
        let world = self.world.borrow();
        let mut inputs = ResMut::<Vec<InputEvent>>::new(&world);
        inputs.push(event);
    }

    #[wasm_bindgen(js_name = "getOutput")]
    pub fn get_output(&self) -> JsValue {
        self.world
            .borrow()
            .get_resource::<Output>()
            .unwrap()
            .0
            .clone()
    }

    #[wasm_bindgen(js_name = "getInventory")]
    pub fn get_inventory(&self) -> JsValue {
        let world = self.world.borrow();
        let item_props = Query::<(
            Option<&Icon>,
            Option<&Description>,
            Option<&Name>,
            &StuffTag,
            Option<&Ranged>,
            Option<&Color>,
        )>::new(&world);
        let inventory = Query::<&Inventory, With<PlayerTag>>::new(&world)
            .iter()
            .next()
            .map(|inv| {
                inv.iter()
                    .map(|id| {
                        let (icon, desc, name, tag, ranged, color) = item_props.fetch(id).unwrap();
                        ItemDesc {
                            id,
                            color: color.and_then(|c| c.0.as_string()),
                            name: name.map(|n| n.0.clone()),
                            description: desc.map(|desc| desc.0.clone()),
                            icon: icon.map(|icon| icon.0.to_string()),
                            usable: matches!(tag, StuffTag::HpPotion | StuffTag::LightningScroll),
                            target_enemy: matches!(tag, StuffTag::LightningScroll),
                            range: ranged.map(|r| r.range).unwrap_or(0),
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
            .borrow_mut()
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .insert_use_item(id);
    }

    #[wasm_bindgen]
    pub fn wait(&mut self) {
        self.world
            .borrow_mut()
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .insert_wait();
    }

    #[wasm_bindgen(js_name = "setTarget")]
    pub fn set_target(&mut self, id: JsValue) {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        debug!("set_target {}", id);
        self.world
            .borrow_mut()
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .set_target(id);
    }

    #[wasm_bindgen(js_name = "setSelection")]
    pub fn set_selection(&mut self, id: JsValue) {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        debug!("set_selection {}", id);
        let mut world = self.world.borrow_mut();
        if world.is_id_valid(id) {
            world.get_resource_mut::<Selected>().unwrap().0 = Some(id);
            world.run_system(update_output);
        }
    }

    #[wasm_bindgen(js_name = "fetchEntity")]
    pub fn fetch_entity(&self, id: JsValue) -> JsValue {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        let world = self.world.borrow_mut();
        if !world.is_id_valid(id) {
            return JsValue::null();
        }
        let tag = Query::<&StuffTag>::new(&world).fetch(id).unwrap();
        archetypes::stuff_to_js(
            id,
            *tag,
            Query::new(&world),
            Query::new(&world),
            Query::new(&world),
            Query::new(&world),
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

        if let Some(canvas) = resources.canvas.as_ref() {
            let world = Rc::clone(&self.world);
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let [x, y] = [event.offset_x() as f64, event.offset_y() as f64];
                let mut world = world.borrow_mut();
                world.insert_resource(ClickPosition(Some([x, y])));
                world.run_system(handle_click);
                world.run_system(update_output);
            });
            canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        self.world.borrow_mut().insert_resource(resources);
        self.world.borrow_mut().run_system(render_into_canvas);
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemDesc {
    pub id: EntityId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub usable: bool,
    pub target_enemy: bool,
    pub range: i32,
}
