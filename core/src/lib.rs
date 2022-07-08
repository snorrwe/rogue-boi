mod components;
mod grid;
mod logging;
mod map_gen;
mod math;
mod pathfinder;
mod systems;
mod utils;

use std::pin::Pin;

use cao_db::{commands::Commands, entity_id::EntityId, query::Query, World};
use components::*;
use grid::Grid;
use math::Vec2;

use systems::{init_player, update_fov, update_grid, update_player};
use tracing::debug;
use wasm_bindgen::prelude::*;

use crate::systems::{update_hp, update_input_events, update_melee_ai};

/// State object
#[wasm_bindgen]
pub struct Core {
    world: Pin<Box<World>>,
    player: EntityId,
    camera_pos: Vec2,
    grid: Grid<Stuff>,
    visible: Grid<bool>,
    explored: Grid<bool>,
    inputs: Vec<InputEvent>,
    actions: PlayerActions,
    output_cache: JsValue,
    viewport: Vec2,
    time: i32,
    game_tick: usize,
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
    let dims = Vec2 { x: 64, y: 64 };
    let mut world = World::new(dims.x as u32 * dims.y as u32);

    init_player(Commands::new(&mut world));
    world.apply_commands().unwrap();

    let (player, _) = Query::<(EntityId, &PlayerTag)>::new(&mut world).one();

    let mut grid = Grid::new(dims);
    map_gen::generate_map(
        player,
        Commands::new(&mut world),
        &mut grid,
        map_gen::MapGenProps {
            room_min_size: 6,
            room_max_size: 10,
            max_rooms: 50,
            max_monsters_per_room: 2,
            max_items_per_room: 2,
        },
    );

    let (_t, Pos(camera_pos)) = Query::<(&PlayerTag, &Pos)>::new(&world).one();
    let camera_pos = *camera_pos;

    let mut core = Core {
        world,
        player,
        grid,
        visible: Grid::new(dims),
        explored: Grid::new(dims),
        inputs: Vec::with_capacity(16),
        actions: PlayerActions::new(),
        output_cache: JsValue::null(),
        viewport: Vec2::new(10, 10),
        time: 0,
        game_tick: 0,
        camera_pos,
    };
    core.init();
    core
}

pub type Stuff = Option<Id>;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "ty")]
pub enum StuffPayload {
    Empty,
    Wall,
    Player { id: Id },
    Troll { id: Id },
    Orc { id: Id },
    Item { id: Id },
}

impl StuffPayload {
    pub fn from_world(id: EntityId, world: &World) -> Self {
        let tag = Query::<&StuffTag>::new(world).fetch(id);
        match tag {
            None => StuffPayload::Empty,
            Some(StuffTag::Wall) => Self::Wall,
            Some(StuffTag::Player) => Self::Player { id: id.into() },
            Some(StuffTag::Troll) => Self::Troll { id: id.into() },
            Some(StuffTag::Orc) => Self::Orc { id: id.into() },
            Some(StuffTag::HpPotion) | Some(StuffTag::Sword) => Self::Item { id: id.into() },
        }
    }
}

impl Default for StuffPayload {
    fn default() -> Self {
        Self::Empty
    }
}

/// Id sent to JS
#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Id {
    pub val: u32,
}

impl From<EntityId> for Id {
    fn from(eid: EntityId) -> Self {
        Self { val: eid.into() }
    }
}

impl From<Id> for EntityId {
    fn from(i: Id) -> Self {
        i.val.into()
    }
}

impl<'a> From<&'a Id> for EntityId {
    fn from(id: &'a Id) -> Self {
        (*id).into()
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

#[derive(Default)]
pub(crate) struct PlayerActions {
    len: usize,
    move_action: Option<Vec2>,
    use_item_action: Option<EntityId>,
}

impl PlayerActions {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PlayerActions {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn clear(&mut self) {
        self.move_action = None;
        self.use_item_action = None;
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
}

#[wasm_bindgen]
impl Core {
    pub fn init(&mut self) {
        game_log!("Hello wanderer!");
        self.tick(0);
        self.update_output();
    }

    pub fn icons(&self) -> JsValue {
        let entries: Vec<_> = ICONS.iter().map(|(_k, Icon(x))| x.to_string()).collect();
        JsValue::from_serde(&entries).unwrap()
    }

    pub fn tick(&mut self, dt_ms: i32) {
        self.time += dt_ms;
        update_input_events(self.inputs.as_slice(), &mut self.actions);

        // min cooldown
        if self.actions.is_empty() || self.time < 120 {
            return;
        }
        let _span = tracing::span!(tracing::Level::DEBUG, "game_update").entered();

        self.time = 0;
        self.game_tick += 1;

        // logic update
        if let Err(err) = update_player(
            &self.actions,
            Commands::new(&mut self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            &mut self.grid,
        ) {
            debug!("player update failed {:?}", err);
            match err {
                systems::PlayerError::NoPlayer | systems::PlayerError::CantMove => {
                    self.game_tick -= 1;
                    self.cleanup();
                    return;
                }
            }
        }
        update_melee_ai(
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            Query::new(&self.world),
            &mut self.grid,
        );
        update_hp(
            Commands::new(&mut self.world),
            Query::new(&self.world),
            Query::new(&self.world),
        );
        // apply logic update commands
        self.world.apply_commands().unwrap();

        // post processing
        update_grid(Query::new(&self.world), &mut self.grid);
        update_fov(
            self.player,
            Query::new(&self.world),
            Query::new(&self.world),
            &self.grid,
            &mut self.explored,
            &mut self.visible,
        );

        if let Some((_t, Pos(camera_pos))) =
            Query::<(&PlayerTag, &Pos)>::new(&self.world).iter().next()
        {
            self.camera_pos = *camera_pos;
        }

        self.update_output();

        // commands should be empty, but let's apply these just to be sure
        self.world.apply_commands().unwrap();

        self.cleanup();
        crate::logging::rotate_log();
    }

    fn cleanup(&mut self) {
        self.inputs.clear();
        self.actions.clear();
    }

    #[wasm_bindgen(js_name = "pushEvent")]
    pub fn push_event(&mut self, event: JsValue) {
        let event: InputEvent = event.into_serde().unwrap();
        self.inputs.push(event);
    }

    fn update_output(&mut self) {
        let _span = tracing::span!(tracing::Level::DEBUG, "update_output").entered();

        let mut result = Grid::new(self.viewport * 2);
        let min = self.camera_pos - self.viewport;
        let max = self.camera_pos + self.viewport;
        for y in min.y.max(0)..max.y.min(self.grid.height()) {
            for x in min.x.max(0)..max.x.min(self.grid.width()) {
                let pos = Vec2::new(x, y);
                let mut output = OutputStuff::default();
                output.explored = self.explored[pos];
                output.visible = self.visible[pos];
                if output.explored {
                    if let Some((id, ty)) = self.grid[pos].and_then(|id| {
                        let id = id.into();
                        Query::<&StuffTag>::new(&self.world)
                            .fetch(id)
                            .map(|ty| (id, ty))
                    }) {
                        if output.visible || ty.static_visiblity() {
                            output.payload = StuffPayload::from_world(id.into(), &self.world);
                            output.icon = Query::<&Icon>::new(&self.world)
                                .fetch(id)
                                .map(|icon| icon.0);
                        }
                    }
                }
                result[pos - min] = output;
            }
        }
        let player = Query::<(&Pos, &Hp, &Melee, &PlayerTag)>::new(&self.world)
            .iter()
            .next()
            .map(|(pos, hp, attack, _tag)| PlayerOutput {
                player_hp: *hp,
                player_attack: attack.power,
                player_pos: pos.0,
            });
        let log = crate::logging::compute_log(self.game_tick);
        let result = RenderedOutput {
            player,
            log,
            grid: result,
            offset: min,
        };
        self.output_cache = JsValue::from_serde(&result).unwrap();
    }

    #[wasm_bindgen(js_name = "getGrid")]
    pub fn get_grid(&self) -> JsValue {
        self.output_cache.clone()
    }

    pub fn width(&self) -> i32 {
        self.grid.width()
    }

    pub fn height(&self) -> i32 {
        self.grid.height()
    }

    #[wasm_bindgen(js_name = "getInventory")]
    pub fn get_inventory(&self) -> JsValue {
        let item_props = Query::<(&Icon, &Description, &StuffTag)>::new(&self.world);
        let inventory = Query::<&Inventory>::new(&self.world)
            .fetch(self.player)
            .map(|inv| {
                inv.iter()
                    .map(|id| {
                        let props = item_props.fetch(id);
                        ItemDesc {
                            id: id.into(),
                            description: props.map(|p| p.1 .0.clone()),
                            icon: props.map(|p| p.0 .0.to_string()),
                            usable: props
                                .map(|p| matches!(p.2, StuffTag::HpPotion))
                                .unwrap_or(false),
                        }
                    })
                    .collect::<Vec<_>>()
            });

        JsValue::from_serde(&inventory).unwrap()
    }

    #[wasm_bindgen(js_name = "useItem")]
    pub fn use_item(&mut self, id: JsValue) {
        let id: Id = JsValue::into_serde(&id).unwrap();
        self.actions.insert_use_item(id.into());
    }
}

#[derive(serde::Serialize)]
struct ItemDesc {
    pub id: Id,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub usable: bool,
}
