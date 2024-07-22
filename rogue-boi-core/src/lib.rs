#![feature(let_chains)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod archetypes;
mod colors;
mod components;
mod game_config;
mod grid;
mod map_gen;
mod math;
mod pathfinder;
mod systems;
mod utils;

use std::{cell::RefCell, rc::Rc};

use crate::systems::{
    drop_item, handle_click, init_world_systems, regenerate_dungeon, update_output,
};
use base64::{engine::GeneralPurpose, Engine};
use cecs::{persister::WorldSerializer, prelude::*};
use colors::WHITE;
use components::*;
use grid::Grid;
use icons::ICONS;
use math::Vec2;
use tracing::{debug, error};
use wasm_bindgen::prelude::*;

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
        .map(|(k, svg)| (*k, web_sys::Path2d::new_with_path_string(svg).unwrap()))
        .collect();
    IconCollection(inner)
}

fn get_world_persister() -> impl WorldSerializer {
    let persister = cecs::persister::WorldPersister::new()
        .with_resource::<WorldDims>()
        .with_resource::<GameTick>()
        .with_resource::<LogHistory>()
        .with_resource::<DungeonFloor>()
        .with_resource::<Explored>();

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
    world.insert_resource(Viewport(Vec2::new(16, 16)));
    world.insert_resource(Visibility(Vec2::new(10, 10)));
    world.insert_resource(CameraPos(Vec2::ZERO));
    world.insert_resource(Output(JsValue::null()));
    world.insert_resource(RenderResources::default());
    world.insert_resource(BounceOffTime(0));
    world.insert_resource(ShouldTick(false));
    world.insert_resource(DeltaTime(0));
    world.insert_resource(TickInMs(100));
    world.insert_resource(AppMode::Game);
    world.insert_resource(TargetPos::default());
    world.insert_resource(None::<DesiredStat>);
    world.insert_resource(PlayerId::default());
}

fn init_world_resources(world_dims: Vec2, world: &mut World) {
    init_world_transient_resources(world_dims, world);
    world.insert_resource(WorldDims(world_dims));
    world.insert_resource(GameTick::default());
    world.insert_resource(LogHistory::default());
    world.insert_resource(Explored(Grid::new(world_dims)));
}

pub fn init_world(world_dims: Vec2, world: &mut World) {
    let player_count = Query::<&(), With<PlayerTag>>::new(world).count();
    assert_eq!(
        player_count, 0,
        "re-initializing exiting World will cause inconsistencies"
    );
    init_world_resources(world_dims, world);
    init_world_systems(world);
}

pub const WORLD_DIMS: Vec2 = Vec2 { x: 64, y: 64 };

fn default_world() -> World {
    let mut world = World::new(WORLD_DIMS.x as u32 * WORLD_DIMS.y as u32);

    init_world(WORLD_DIMS, &mut world);
    world
}

#[wasm_bindgen(js_name = "initCore")]
pub fn init_core() -> Core {
    let world = default_world();

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

#[derive(Debug, Default, Clone)]
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

type ItemPropsTuple<'a> = (
    Option<&'a Icon>,
    Option<&'a Description>,
    Option<&'a Name>,
    &'a StuffTag,
    Option<&'a Ranged>,
    Option<&'a Color>,
);
type ItemPropsQ<'a> = Query<'a, ItemPropsTuple<'a>>;

fn to_item_desc(id: EntityId, i: ItemPropsTuple) -> ItemDesc {
    let (icon, desc, name, tag, ranged, color) = i;
    ItemDesc {
        id,
        color: color.and_then(|c| c.0.as_string()),
        name: name.map(|n| n.0.clone()),
        description: desc.map(|desc| desc.0.clone()),
        icon: icon.map(|icon| icon.0.to_string()),
        usable: archetypes::usable(*tag),
        range: ranged.map(|r| r.range).unwrap_or(0),
    }
}

const BASE64_ENGINE: GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(serde::Deserialize)]
pub struct MapGenParams {
    pub dims: Vec2,
    pub level: u32,
}

impl Default for MapGenParams {
    fn default() -> Self {
        MapGenParams {
            dims: WORLD_DIMS,
            level: 1,
        }
    }
}

#[wasm_bindgen]
impl Core {
    pub fn restart(&mut self) {
        let mut world = self.world.borrow_mut();

        // delete the player
        world
            .run_system(|mut cmd: Commands, q: Query<EntityId, With<PlayerTag>>| {
                for id in q.iter() {
                    cmd.delete(id);
                }
            })
            .unwrap();

        world.insert_resource(DeltaTime(0));
        world.insert_resource(GameTick::default());
        world.insert_resource(DungeonFloor::default());
        world.insert_resource(Selected::default());
        world.insert_resource(AppMode::Game);

        let log = world.get_resource_mut::<LogHistory>().unwrap();
        log.items.clear();
        log.push(WHITE, "Hello wanderer!");

        world.run_system(regenerate_dungeon).unwrap();
        drop(world);
        self.tick(10000);
    }

    /// Generate a dungoen without altering the world state
    pub fn generate_dungeon(&self, params: JsValue) -> JsValue {
        let params: Option<MapGenParams> =
            serde_wasm_bindgen::from_value(params).expect("Failed to deserialize params");
        let params = params.unwrap_or_default();

        let mut world = World::new(params.dims.x as u32 * params.dims.y as u32);
        init_world(params.dims, &mut world);
        world.insert_resource(DungeonFloor {
            current: params.level,
            desired: params.level,
        });
        world.insert_resource(map_gen::MapGenProps::from_level(params.level));
        world.run_system(map_gen::generate_map).unwrap();

        world.run_view_system(move |tags: Query<(EntityId, &Icon, &Pos)>| {
            let map = tags
                .iter()
                .map(|(_id, icon, pos)| (format!("{};{}", pos.0.x, pos.0.y), &icon.0))
                .collect::<std::collections::HashMap<_, _>>();
            serde_wasm_bindgen::to_value(&map).unwrap()
        })
    }

    /// return the name of the icons (without the extension!)
    pub fn icons(&self) -> JsValue {
        let entries: Vec<_> = ICONS.iter().map(|(k, _x)| k).collect();
        serde_wasm_bindgen::to_value(&entries).unwrap()
    }

    pub fn tick(&mut self, dt_ms: i32) {
        let mut world = self.world.borrow_mut();
        world.insert_resource(DeltaTime(dt_ms));
        world.tick();
    }

    #[wasm_bindgen(js_name = "pushEvent")]
    pub fn push_event(&mut self, event: JsValue) {
        let event: InputEvent = serde_wasm_bindgen::from_value(event).unwrap();
        let world = self.world.borrow();
        let mut inputs = ResMut::<Vec<InputEvent>>::new(&world);
        inputs.push(event);
    }

    #[wasm_bindgen(js_name = "getOutput")]
    pub fn get_output(&self) -> JsValue {
        let mut world = self.world.borrow_mut();
        let mut output = world.get_resource::<Output>().unwrap().0.clone();
        if output.is_null() {
            // Kinda lame, but ensure that the output is not null
            world.run_system(update_output).unwrap();
            output = world.get_resource::<Output>().unwrap().0.clone();
        }
        output
    }

    #[wasm_bindgen(js_name = "getEquipment")]
    pub fn get_equipment(&self) -> JsValue {
        let world = self.world.borrow();
        let item_props = ItemPropsQ::new(&world);

        let result = match Query::<&Equipment, With<PlayerTag>>::new(&world)
            .iter()
            .next()
        {
            Some(equipment) => serde_wasm_bindgen::to_value(&serde_json::json!({
                "weapon": equipment.weapon.map(|id|to_item_desc(id, item_props.fetch(id).unwrap())),
                "armor": equipment.armor.map(|id|to_item_desc(id, item_props.fetch(id).unwrap()))
            }))
            .unwrap(),
            None => JsValue::null(),
        };
        result
    }

    #[wasm_bindgen(js_name = "getInventory")]
    pub fn get_inventory(&self) -> JsValue {
        let world = self.world.borrow();
        let item_props = ItemPropsQ::new(&world);
        let inventory = Query::<&Inventory, With<PlayerTag>>::new(&world)
            .iter()
            .next()
            .map(|inv| {
                inv.iter()
                    .map(|id| to_item_desc(id, item_props.fetch(id).unwrap()))
                    .collect::<Vec<_>>()
            });

        serde_wasm_bindgen::to_value(&inventory).unwrap()
    }

    #[wasm_bindgen(js_name = "useItem")]
    pub fn use_item(&mut self, id: JsValue) {
        let id: EntityId = serde_wasm_bindgen::from_value(id).unwrap();
        if !self.world.borrow().is_id_valid(id) {
            error!("use_item id is not valid");
            return;
        }
        let mut w = self.world.borrow_mut();
        w.run_system(|mut cmd: Commands| {
            cmd.entity(id).insert_bundle((UseItem,));
        })
        .unwrap();
    }

    #[wasm_bindgen(js_name = "unequipItem")]
    pub fn unequip_item(&mut self, id: JsValue) {
        let id: EntityId = serde_wasm_bindgen::from_value(id).unwrap();
        if !self.world.borrow().is_id_valid(id) {
            error!("unequip_item id is not valid");
            return;
        }
        let mut w = self.world.borrow_mut();
        w.run_system(|mut cmd: Commands| {
            cmd.entity(id).insert_bundle((Unequip,));
        })
        .unwrap();
    }

    #[wasm_bindgen(js_name = "dropItem")]
    pub fn drop_item(&mut self, id: JsValue) {
        let id: EntityId = serde_wasm_bindgen::from_value(id).unwrap();
        if !self.world.borrow().is_id_valid(id) {
            error!("drop_item id is not valid");
            return;
        }

        let mut world = self.world.borrow_mut();
        world
            .run_system(
                |mut cmd: Commands,
                 mut q: Query<(&Pos, &mut Inventory), With<PlayerTag>>,
                 q_item: Query<&Name>,
                 mut log: ResMut<LogHistory>| {
                    // remove item from inventory and add a position
                    // TODO: random empty nearby position intead of the player's?
                    if let Some((pos, inv)) = q.single_mut() {
                        if let Some(item) = inv.remove(id) {
                            if let Some(name) = q_item.fetch(item) {
                                drop_item(cmd.entity(id), pos, name, &mut log);
                            }
                        }
                    }
                },
            )
            .unwrap();
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
        let id: EntityId = serde_wasm_bindgen::from_value(id).unwrap();
        debug!("set_target {}", id);
        self.world
            .borrow_mut()
            .get_resource_mut::<PlayerActions>()
            .unwrap()
            .set_target(id);
    }

    #[wasm_bindgen(js_name = "setSelection")]
    pub fn set_selection(&mut self, id: JsValue) {
        let id: EntityId = serde_wasm_bindgen::from_value(id).unwrap();
        debug!("set_selection {}", id);
        let mut world = self.world.borrow_mut();
        if world.is_id_valid(id) {
            world.get_resource_mut::<Selected>().unwrap().0 = Some(id);
            world.run_system(update_output).unwrap();
        }
    }

    #[wasm_bindgen(js_name = "fetchEntity")]
    pub fn fetch_entity(&self, id: JsValue) -> JsValue {
        let id: EntityId = serde_wasm_bindgen::from_value(id).unwrap();
        let mut world = self.world.borrow_mut();
        if !world.is_id_valid(id) {
            return JsValue::null();
        }
        world
            .run_system(move |tags: Query<&StuffTag>, q| {
                archetypes::stuff_to_js(id, *tags.fetch(id).unwrap(), &q)
            })
            .unwrap()
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
                world.run_system(handle_click).unwrap();
                world.run_system(update_output).unwrap();
            });
            canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        self.world.borrow_mut().insert_resource(resources);
    }

    #[wasm_bindgen(js_name = "cancelItemUse")]
    pub fn cancel_item_use(&mut self) {
        let mut world = self.world.borrow_mut();
        world
            .run_system(|mut cmd: Commands, q: Query<EntityId, With<UseItem>>| {
                q.iter().for_each(|id| {
                    cmd.entity(id).remove::<UseItem>();
                });
            })
            .unwrap();
        let mode = world.get_resource_mut::<AppMode>().unwrap();
        if matches!(*mode, AppMode::Targeting) {
            *mode = AppMode::Game;
        }
        let log = world.get_resource_mut::<LogHistory>().unwrap();
        log.push(WHITE, "Cancel item use");
    }

    pub fn save(&self) -> String {
        let world = self.world.borrow();
        let ser = WorldSer { world: &world };

        let mut result = Vec::<u8>::with_capacity(36000);
        ciborium::into_writer(&ser, &mut result).expect("failed to serialize");

        debug!("cbor size {}", result.len());
        let encoded = BASE64_ENGINE.encode(result);
        debug!("encoded size {}", encoded.len());

        encoded
    }

    pub fn load(&mut self, pl: String) {
        debug!("• loading");
        let pl = BASE64_ENGINE.decode(pl).expect("failed to b64 decode");

        let WorldDe { mut world } =
            ciborium::from_reader(pl.as_slice()).expect("failed to load world");

        let dims = *world
            .get_resource::<WorldDims>()
            .expect("world has no dims");
        init_world_transient_resources(dims.0, &mut world);

        world
            .run_system(archetypes::insert_transient_components)
            .unwrap();
        world.run_system(systems::init_grids).unwrap();
        world.run_system(systems::update_camera_pos).unwrap();
        world.run_system(systems::update_fov).unwrap();
        init_world_systems(&mut world);

        self.world.replace(world);
        self.tick(10000);
        debug!("✓ loading");
    }

    #[wasm_bindgen(js_name = "setLevelupStat")]
    pub fn set_levelup_stat(&mut self, stat: JsValue) {
        let stat: DesiredStat = serde_wasm_bindgen::from_value(stat).unwrap();
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
    pub range: i32,
}

pub struct WorldSer<'a> {
    pub world: &'a World,
}

pub struct WorldDe {
    pub world: World,
}

impl serde::Serialize for WorldSer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let p = get_world_persister();
        p.save(serializer, self.world)
    }
}

impl<'de> serde::Deserialize<'de> for WorldDe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let p = get_world_persister();
        let result: World = p.load(deserializer)?;
        Ok(Self { world: result })
    }
}
