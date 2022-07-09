mod rect_room;
mod tunnel_iter;

use self::rect_room::RectRoom;
use self::tunnel_iter::TunnelIter;
use cao_db::prelude::*;
use rand::{
    prelude::{Distribution, SliceRandom},
    Rng,
};

use crate::{
    archetypes::{init_entity, ENEMY_TAGS, ENEMY_WEIGHTS, ITEM_TAGS, ITEM_WEIGHTS},
    components::{Pos, StuffTag},
    grid::Grid,
    math::Vec2,
    Stuff,
};

pub struct MapGenProps {
    pub room_min_size: u32,
    pub room_max_size: u32,
    pub max_rooms: u32,
    pub max_monsters_per_room: u32,
    pub max_items_per_room: u32,
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
        }
    }
}

pub fn generate_map(
    player_id: EntityId,
    mut cmd: Commands,
    grid: &mut Grid<Stuff>,
    props: MapGenProps,
) {
    grid.fill(None); // reset the grid
    let mut working_set = Grid::new(grid.dims());
    // fill the map with walls and delete old entities
    //
    for (p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff {
            // delete all but player entities from the database
            let id: EntityId = (*id).into();
            if id != player_id {
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
                // update player pos
                cmd.entity(player_id).insert(Pos(pos));
                grid[pos] = Some(player_id.into());
            }
            _ => {
                init_entity(pos, tag, &mut cmd, grid);
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
