#![feature(let_else)]
#![feature(let_chains)]

mod archetypes;
mod colors;
mod components;
mod grid;
mod map_gen;
mod math;
mod pathfinder;
mod systems;
mod utils;

use std::{cell::RefCell, rc::Rc};

use crate::systems::*;
use cecs::{persister::WorldSerializer, prelude::*};
use colors::WHITE;
use components::*;
use grid::Grid;
use icons::ICONS;
use math::Vec2;
use tracing::{debug, error};
use wasm_bindgen::{prelude::*, JsCast};

/// State object
#[wasm_bindgen]
pub struct Core {
    world: Rc<RefCell<World>>,
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

fn get_world_persister() -> impl WorldSerializer {
    let persister = cecs::persister::WorldPersister::new()
        .add_resource::<WorldDims>()
        .add_resource::<GameTick>()
        .add_resource::<LogHistory>()
        .add_resource::<DungeonFloor>()
        .add_resource::<Explored>();

    archetypes::register_persistent_components(persister)
}

fn init_world_transient_resources(world_dims: Vec2, world: &mut World) {
    world.insert_resource(Grid::<Stuff>::new(world_dims));
    world.insert_resource(StaticGrid(Grid::new(world_dims)));
    world.insert_resource(ClickPosition(None));
    world.insert_resource(Selected::default());
    world.insert_resource(compute_icons());
    world.insert_resource(ShouldUpdateWorld(false));
    world.insert_resource(ShouldUpdatePlayer(false));
    world.insert_resource(Vec::<InputEvent>::with_capacity(16));
    world.insert_resource(PlayerActions::new());
    world.insert_resource(Visible(Grid::new(world_dims)));
    world.insert_resource(Viewport(Vec2::new(20, 20)));
    world.insert_resource(Visibility(Vec2::new(10, 10)));
    world.insert_resource(CameraPos(Vec2::ZERO));
    world.insert_resource(Output(JsValue::null()));
    world.insert_resource(RenderResources::default());
    world.insert_resource(BounceOffTime(0));
    world.insert_resource(ShouldTick(false));
    world.insert_resource(DeltaTime(0));
    world.insert_resource(TickInMs(120));
    world.insert_resource(AppMode::Game);
    world.insert_resource(UseItem::default());
    world.insert_resource(TargetPos::default());
    world.insert_resource(None::<DesiredStat>);
}

fn init_world_resources(world_dims: Vec2, world: &mut World) {
    init_world_transient_resources(world_dims, world);
    world.insert_resource(WorldDims(world_dims));
    world.insert_resource(GameTick::default());
    world.insert_resource(LogHistory::default());
    world.insert_resource(Explored(Grid::new(world_dims)));
}

fn init_world_systems(world: &mut World) {
    world.add_stage(
        SystemStage::serial("inputs")
            .with_system(update_input_events)
            .with_system(update_should_tick)
            .with_system(handle_targeting)
            .with_system(player_prepare)
            .with_system(handle_levelup),
    );
    world.add_stage(
        SystemStage::serial("pre-update")
            .with_should_run(|should_tick: Res<ShouldTick>| should_tick.0)
            .with_system(record_last_pos),
    );
    world.add_stage(
        SystemStage::serial("player-update")
            .with_should_run(should_update_player)
            .with_system(update_player_item_use)
            .with_system(handle_player_move)
            .with_system(update_player_world_interact)
            .with_system(update_camera_pos),
    );
    world.add_stage(SystemStage::serial("update-ai-hp").with_system(update_ai_hp));
    world.add_stage(
        SystemStage::serial("ai-update")
            .with_should_run(should_update_world)
            .with_system(update_ai_move)
            .with_system(update_melee_ai)
            .with_system(update_confusion)
            .with_system(update_player_hp)
            .with_system(update_grid)
            .with_system(update_fov),
    );
    world.add_stage(SystemStage::serial("update-pos").with_system(perform_move));
    world.add_stage(
        SystemStage::serial("render")
            .with_system(update_output)
            .with_system(render_into_canvas)
            .with_system(systems::clean_inputs),
    );
    world.add_stage(
        SystemStage::serial("post-render")
            .with_should_run(should_update_world)
            .with_system(update_tick),
    );
    world.add_stage(
        SystemStage::serial("dungeon-delve")
            .with_should_run(|level: Res<DungeonFloor>| level.current != level.desired)
            .with_system(regenerate_dungeon),
    );
}

pub fn init_world(world_dims: Vec2, world: &mut World) {
    let player_count = Query::<&(), With<PlayerTag>>::new(&world).count();
    assert_eq!(
        player_count, 0,
        "re-initializing exiting World will cause inconsistencies"
    );
    init_world_resources(world_dims, world);
    init_world_systems(world);
}

#[wasm_bindgen(js_name = "initCore")]
pub fn init_core() -> Core {
    let world_dims = Vec2 { x: 64, y: 64 };
    let mut world = World::new(world_dims.x as u32 * world_dims.y as u32);

    init_world(world_dims, &mut world);

    let world = Rc::new(RefCell::new(world));
    Core { world }
}

pub type Stuff = Option<EntityId>;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "ty")]
pub enum InputEvent {
    KeyUp { key: String },
    KeyDown { key: String },
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedOutput {
    pub selected: Option<EntityId>,
    pub player: Option<PlayerOutput>,
    pub log: Vec<String>,
    pub targeting: bool,
    pub dungeon_level: u32,
    pub app_mode: AppMode,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerOutput {
    pub player_hp: Hp,
    pub player_attack: i32,
    pub player_pos: Vec2,
    pub current_xp: u32,
    pub needed_xp: u32,
    pub level: u32,
    pub defense: Defense,
}

#[derive(Default, Clone, Copy)]
pub struct UseItem(Option<EntityId>);

#[derive(Default, Clone)]
pub struct PlayerActions {
    len: usize,
    move_action: Option<Vec2>,
    target: Option<EntityId>,
    interact: bool,
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
        self.target = None;
        self.wait = false;
        self.len = 0;
        self.interact = false;
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

    pub fn interact(&self) -> bool {
        self.interact
    }

    pub fn insert_interact(&mut self) {
        if !self.interact {
            self.len += 1;
        }
        self.interact = true;
    }
}

#[wasm_bindgen]
impl Core {
    pub fn restart(&mut self) {
        let mut world = self.world.borrow_mut();

        // delete the player
        world.run_system(|mut cmd: Commands, q: Query<EntityId, With<PlayerTag>>| {
            for id in q.iter() {
                cmd.delete(id);
            }
        });

        world.insert_resource(DeltaTime(0));
        world.insert_resource(GameTick::default());
        world.insert_resource(DungeonFloor::default());

        world.insert_resource(AppMode::Game);
        world.insert_resource(UseItem::default());

        let log = world.get_resource_mut::<LogHistory>().unwrap();
        log.items.clear();
        log.push(WHITE, "Hello wanderer!");

        world.run_system(regenerate_dungeon);
    }

    /// return the name of the icons (without the extension!)
    pub fn icons(&self) -> JsValue {
        let entries: Vec<_> = ICONS.iter().map(|(k, _x)| k).collect();
        JsValue::from_serde(&entries).unwrap()
    }

    pub fn tick(&mut self, dt_ms: i32) {
        let mut world = self.world.borrow_mut();
        world.insert_resource(DeltaTime(dt_ms));
        world.tick();
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
        if !self.world.borrow().is_id_valid(id) {
            error!("use_item id is not valid");
            return;
        }
        self.world
            .borrow_mut()
            .get_resource_mut::<UseItem>()
            .unwrap()
            .0 = Some(id);
    }

    #[wasm_bindgen(js_name = "dropItem")]
    pub fn drop_item(&mut self, id: JsValue) {
        let id: EntityId = JsValue::into_serde(&id).unwrap();
        if !self.world.borrow().is_id_valid(id) {
            error!("drop_item id is not valid");
            return;
        }

        fn sys<'a>(
            mut cmd: Commands,
            mut q: Query<(&'a Pos, &'a mut Inventory), With<PlayerTag>>,
            q_item: Query<&Name>,
            item: Res<DropItem>,
            mut log: ResMut<LogHistory>,
        ) {
            // remove item from inventory and add a position
            // TODO: random empty nearby position intead of the player's?
            if let Some((pos, inv)) = q.iter_mut().next() {
                if let Some(item) = inv.remove(item.0) {
                    if let Some(Name(name)) = q_item.fetch(item) {
                        log.push(WHITE, format!("Drop {}", name));
                        cmd.entity(item).insert(*pos);
                    }
                }
            }
        }
        let mut world = self.world.borrow_mut();
        world.insert_resource(DropItem(id));
        world.run_system(sys);
        world.remove_resource::<DropItem>();
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
        let mut world = self.world.borrow_mut();
        if !world.is_id_valid(id) {
            return JsValue::null();
        }
        world.run_system(move |tags: Query<&StuffTag>, q| {
            archetypes::stuff_to_js(id, *tags.fetch(id).unwrap(), q)
        })
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

    #[wasm_bindgen(js_name = "cancelItemUse")]
    pub fn cancel_item_use(&mut self) {
        let mut world = self.world.borrow_mut();
        world.get_resource_mut::<UseItem>().unwrap().0 = None;
        let mode = world.get_resource_mut::<AppMode>().unwrap();
        if matches!(*mode, AppMode::Targeting) {
            *mode = AppMode::Game;
        }
        let log = world.get_resource_mut::<LogHistory>().unwrap();
        log.push(WHITE, "Cancel item use");
    }

    pub fn save(&self) -> String {
        let p = get_world_persister();
        let world = self.world.borrow();

        let mut result = Vec::<u8>::new();
        let mut s = bincode::Serializer::new(&mut result, bincode::config::DefaultOptions::new());

        p.save(&mut s, &world).unwrap();

        debug!("bincode size {}", result.len());
        let encoded = base64::encode(result);
        debug!("encoded size {}", encoded.len());

        encoded
    }

    pub fn load(&mut self, pl: String) {
        let p = get_world_persister();
        let pl = base64::decode(pl).unwrap();

        let mut world = p
            .load(&mut bincode::de::Deserializer::from_slice(
                pl.as_slice(),
                bincode::config::DefaultOptions::new(),
            ))
            .unwrap();

        world.gc_empty_entities();

        let dims = *world.get_resource::<WorldDims>().unwrap();
        init_world_transient_resources(dims.0, &mut world);

        world.run_system(archetypes::insert_transient_components);
        world.run_system(systems::init_grids);
        world.run_system(systems::update_camera_pos);
        world.run_system(systems::update_fov);
        init_world_systems(&mut world);

        *self.world.borrow_mut() = world;
        self.tick(0);
    }

    #[wasm_bindgen(js_name = "setLevelupStat")]
    pub fn set_levelup_stat(&mut self, stat: JsValue) {
        let stat: DesiredStat = stat.into_serde().unwrap();
        let mut w = self.world.borrow_mut();
        w.insert_resource(Some(stat));
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
