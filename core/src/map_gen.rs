mod rect_room;
mod tunnel_iter;

use self::rect_room::RectRoom;
use self::tunnel_iter::TunnelIter;
use cecs::prelude::*;
use rand::{
    prelude::{Distribution, SliceRandom},
    Rng,
};
use tracing::debug;

use crate::{
    archetypes::{init_entity, ENEMY_TAGS, ENEMY_WEIGHTS, ITEM_TAGS, ITEM_WEIGHTS},
    components::{PlayerTag, Pos, StuffTag},
    grid::Grid,
    math::Vec2,
    Stuff,
};

#[derive(Clone)]
pub struct MapGenProps {
    pub room_min_size: u32,
    pub room_max_size: u32,
    pub max_rooms: u32,
    pub max_monsters_per_room: u32,
    pub max_items_per_room: u32,
}

impl MapGenProps {
    pub fn from_level(_level: u32) -> Self {
        // TODO: use level
        MapGenProps {
            room_min_size: 6,
            room_max_size: 10,
            max_rooms: 50,
            max_monsters_per_room: 2,
            max_items_per_room: 2,
        }
    }
}

fn place_entities(
    rng: &mut impl Rng,
    grid: &mut Grid<Option<StuffTag>>,
    room: &RectRoom,
    max_monsters: u32,
) {
    let n_monsters_a = rng.gen_range(0..=max_monsters);
    let n_monsters_b = rng.gen_range(0..=max_monsters);
    let n_monsters = n_monsters_a.max(n_monsters_b); // bias towards more monsters

    let dist = rand::distributions::WeightedIndex::new(ENEMY_WEIGHTS).unwrap();

    for _ in 0..n_monsters {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = ENEMY_TAGS[dist.sample(rng)];
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
) {
    let n_items = rng.gen_range(0..=max_items);

    let dist = rand::distributions::WeightedIndex::new(ITEM_WEIGHTS).unwrap();

    for _ in 0..n_items {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = ITEM_TAGS[dist.sample(rng)];
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
) {
    // player may or may not exist at this point
    let player_id = player_q.iter().next();
    let mut working_set = Grid::new(grid.dims());
    // fill the map with walls and delete old entities
    //
    for (p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff {
            // delete all but player entities from the database
            // player is preserved between levels
            // ignore stale ids
            let id = *id;
            if entities.fetch(id).is_some() && Some(id) != player_id {
                cmd.delete(id);
            }
        }
        working_set[p] = Some(StuffTag::Wall);
        *stuff = None;
    }

    build_rooms(&mut working_set, &props);

    // insert entities into db
    //
    for (pos, tag) in working_set.iter().filter_map(|(p, t)| t.map(|t| (p, t))) {
        match tag {
            StuffTag::Player => {
                if let Some(player_id) = player_id {
                    // update player pos
                    cmd.entity(player_id).insert(Pos(pos));
                    grid[pos] = Some(player_id.into());
                } else {
                    init_entity(pos, tag, &mut cmd, &mut grid);
                }
            }
            _ => {
                init_entity(pos, tag, &mut cmd, &mut grid);
            }
        }
    }
}

fn build_rooms(grid: &mut Grid<Option<StuffTag>>, props: &MapGenProps) {
    let mut rng = rand::thread_rng();
    let mut rooms = Vec::<RectRoom>::with_capacity(props.max_rooms as usize);

    'outer: for _ in 0..props.max_rooms {
        let width = rng.gen_range(props.room_min_size..props.room_max_size) as i32;
        let height = rng.gen_range(props.room_min_size..props.room_max_size) as i32;

        let x = rng.gen_range(1..grid.width() - 1 - width);
        let y = rng.gen_range(1..grid.height() - 1 - height);

        let room = RectRoom::new(x, y, width, height);
        for r in rooms.iter() {
            if room.touches(&r) {
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
            grid[p] = None;
        }
    }

    for room in rooms.iter().skip(1) {
        place_entities(&mut rng, grid, room, props.max_monsters_per_room);
        place_items(&mut rng, grid, room, props.max_items_per_room);
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
