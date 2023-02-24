mod rect_room;
mod tunnel_iter;

use std::collections::HashMap;

use self::rect_room::RectRoom;
use self::tunnel_iter::TunnelIter;
use crate::game_config::{ENEMY_CHANCES, ITEM_CHANCES};
use cecs::prelude::*;
use rand::{
    prelude::{Distribution, SliceRandom},
    Rng,
};
use tracing::debug;

use crate::{
    archetypes::init_entity,
    components::{DungeonFloor, PlayerTag, Pos, StuffTag, WorldDims},
    grid::Grid,
    math::Vec2,
    Stuff,
};

#[derive(Clone)]
pub struct MapGenProps {
    pub room_min_size: u32,
    pub room_max_size: u32,
    pub max_rooms: u32,
    pub max_items_per_floor: u32,
    pub max_monsters_per_floor: u32,
}

impl MapGenProps {
    pub fn from_level(level: u32) -> Self {
        // sorted from high to low
        const MAX_ITEMS: &[[u32; 2]] = &[[4, 2], [1, 1]];
        const MAX_MONSTERS: &[[u32; 2]] = &[[6, 5], [4, 3], [1, 2]];

        let max_items_per_floor = MAX_ITEMS
            .iter()
            .find(|[x, _]| x <= &level)
            .map(|[_, c]| *c)
            .unwrap_or(0);

        let max_monsters_per_floor = MAX_MONSTERS
            .iter()
            .find(|[x, _]| x <= &level)
            .map(|[_, c]| *c)
            .unwrap_or(0);

        MapGenProps {
            max_monsters_per_floor,
            max_items_per_floor,
            room_min_size: 6,
            room_max_size: 10,
            max_rooms: 50,
        }
    }
}

#[derive(Default, Debug)]
struct EntityChances {
    enemy_tags: smallvec::SmallVec<[StuffTag; 8]>,
    enemy_weights: smallvec::SmallVec<[i32; 8]>,
    item_tags: smallvec::SmallVec<[StuffTag; 8]>,
    item_weights: smallvec::SmallVec<[i32; 8]>,
}

pub type EntityChanceList<'a> = &'a [(u32, &'a [(StuffTag, i32)])];
fn entity_weighted_chances(
    floor: u32,
    chances: EntityChanceList,
) -> (
    smallvec::SmallVec<[StuffTag; 8]>,
    smallvec::SmallVec<[i32; 8]>,
) {
    // allow overriding chances on higher floors
    let mut weighted_chances = HashMap::new();
    for (key, weights) in chances {
        if key > &floor {
            break;
        }
        for (tag, weight) in *weights {
            weighted_chances.insert(*tag, *weight);
        }
    }

    (
        weighted_chances.keys().copied().collect(),
        weighted_chances.values().copied().collect(),
    )
}

impl EntityChances {
    pub fn from_level(level: u32) -> Self {
        let mut result = Self::default();

        (result.enemy_tags, result.enemy_weights) = entity_weighted_chances(level, ENEMY_CHANCES);
        (result.item_tags, result.item_weights) = entity_weighted_chances(level, ITEM_CHANCES);

        debug_assert_eq!(result.enemy_weights.len(), result.enemy_tags.len());
        debug_assert_eq!(result.item_weights.len(), result.item_tags.len());

        result
    }
}

fn place_entities(
    rng: &mut impl Rng,
    grid: &mut Grid<Option<StuffTag>>,
    room: &RectRoom,
    max_monsters: u32,
    weights: &EntityChances,
) {
    let n_monsters_a = rng.gen_range(0..=max_monsters);
    let n_monsters_b = rng.gen_range(0..=max_monsters);
    let n_monsters = n_monsters_a.max(n_monsters_b); // bias towards more monsters

    let dist = rand::distributions::WeightedIndex::new(&weights.enemy_weights[..]).unwrap();

    for _ in 0..n_monsters {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = weights.enemy_tags[dist.sample(rng)];
            grid[pos] = Some(tag);
            debug!("Placing {:?} at {}", tag, pos);
        }
    }
}

fn place_stairs(rng: &mut impl Rng, grid: &mut Grid<Option<StuffTag>>, room: &RectRoom) {
    loop {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            debug!("Placing end at {}", pos);
            grid[pos] = Some(StuffTag::Stairs);
            return;
        }
    }
}

fn place_items(
    rng: &mut impl Rng,
    grid: &mut Grid<Option<StuffTag>>,
    room: &RectRoom,
    max_items: u32,
    weights: &EntityChances,
) {
    let n_items = rng.gen_range(0..=max_items);

    let dist = rand::distributions::WeightedIndex::new(&weights.item_weights[..]).unwrap();

    for _ in 0..n_items {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = weights.item_tags[dist.sample(rng)];
            grid[pos] = Some(tag);
            debug!("Placing {:?} at {}", tag, pos);
        }
    }
}

pub fn generate_map(
    player_q: Query<EntityId, With<PlayerTag>>,
    entities: Query<EntityId>,
    mut cmd: Commands,
    mut grid: ResMut<Grid<Stuff>>,
    props: Res<MapGenProps>,
    dims: Res<WorldDims>,
    floor: Res<DungeonFloor>,
) {
    // player may or may not exist at this point
    let player_id = player_q.iter().next();
    let mut working_set = Grid::new(dims.0);
    // fill the map with walls and delete old entities
    //
    // FIXME: this is a lot of wall, use None pls
    working_set.splat_set([Vec2::ZERO, dims.0], Some(StuffTag::Wall));
    for (_p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff {
            // delete all but player entities from the database
            // player is preserved between levels
            // ignore stale ids
            let id = *id;
            if entities.fetch(id).is_some() && Some(id) != player_id {
                cmd.delete(id);
            }
        }
        *stuff = None;
    }

    build_rooms(&mut working_set, &props, floor.current);

    // insert entities into db
    //
    if dims.0 != grid.dims() {
        *grid = Grid::new(dims.0);
    }
    'insert_loop: for (pos, tag) in working_set.iter().filter_map(|(p, t)| t.map(|t| (p, t))) {
        match tag {
            StuffTag::Player => {
                if let Some(player_id) = player_id {
                    // update player pos
                    cmd.entity(player_id).insert(Pos(pos));
                    grid[pos] = Some(player_id);
                } else {
                    init_entity(pos, tag, &mut cmd, &mut grid);
                }
            }
            StuffTag::Wall => {
                // clear invisible walls
                // leave walls around the edge of the map just to be safe
                for y in -1..=1 {
                    for x in -1..=1 {
                        let stuff = working_set.at(pos.x + x, pos.y + y);
                        if stuff.is_none() || stuff.and_then(|s| *s).is_none() {
                            init_entity(pos, tag, &mut cmd, &mut grid);
                            continue 'insert_loop;
                        }
                    }
                }
            }
            _ => {
                init_entity(pos, tag, &mut cmd, &mut grid);
            }
        }
    }
}

fn build_rooms(grid: &mut Grid<Option<StuffTag>>, props: &MapGenProps, floor: u32) {
    let mut rng = rand::thread_rng();
    let mut rooms = Vec::<RectRoom>::with_capacity(props.max_rooms as usize);

    'outer: for _ in 0..props.max_rooms {
        let width = rng.gen_range(props.room_min_size..props.room_max_size) as i32;
        let height = rng.gen_range(props.room_min_size..props.room_max_size) as i32;

        // -2 so all rooms have walls, even those that touch the end of the map
        let x = rng.gen_range(1..grid.width() - 2 - width);
        let y = rng.gen_range(1..grid.height() - 2 - height);

        let room = RectRoom::new(x, y, width, height);
        for r in rooms.iter() {
            if room.touches(r) {
                continue 'outer;
            }
        }
        room.carve(grid);
        rooms.push(room);
    }

    assert!(rooms.len() >= 2);
    rooms.shuffle(&mut rng);

    for (r1, r2) in rooms.iter().zip(rooms.iter().skip(1)) {
        // connect these rooms
        for p in tunnel_between(&mut rng, r1.center(), r2.center()) {
            if is_on_edge(r1, p) || is_on_edge(r2, p) {
                grid[p] = Some(StuffTag::Door);
            } else {
                grid[p] = None;
            }
        }
    }

    let entity_weights = EntityChances::from_level(floor);
    for room in rooms.iter().skip(1) {
        place_entities(
            &mut rng,
            grid,
            room,
            props.max_monsters_per_floor,
            &entity_weights,
        );
        place_items(
            &mut rng,
            grid,
            room,
            props.max_items_per_floor,
            &entity_weights,
        );
    }

    // spawn the player in the first room
    grid[rooms[0].center()] = Some(StuffTag::Player);

    let end_room = rooms[1..]
        .choose(&mut rng)
        .expect("Expected more than 1 room");
    place_stairs(&mut rng, grid, end_room);
}

fn tunnel_between(mut rng: impl Rng, start: Vec2, end: Vec2) -> impl Iterator<Item = Vec2> {
    let Vec2 { x: x1, y: y1 } = start;
    let Vec2 { x: x2, y: y2 } = end;

    let cornerx;
    let cornery;
    if rng.gen_bool(0.5) {
        cornerx = x2;
        cornery = y1;
    } else {
        cornerx = x1;
        cornery = y2;
    }

    TunnelIter::new(start, end, Vec2::new(cornerx, cornery))
}

fn is_on_edge(room: &RectRoom, point: Vec2) -> bool {
    if room.min.x - 1 == point.x || room.max.x + 1 == point.x {
        room.min.y <= point.y && point.y <= room.max.y
    } else if room.min.y - 1 == point.y || room.max.y + 1 == point.y {
        room.min.x <= point.x && point.x <= room.max.x
    } else {
        false
    }
}
