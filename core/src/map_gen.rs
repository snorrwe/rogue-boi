mod rect_room;
mod tunnel_iter;

use self::rect_room::RectRoom;
use self::tunnel_iter::TunnelIter;
use cao_db::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::{
    components::{Ai, Hp, MeleeAi, Pos, StuffTag, ENEMY_TAGS, ICONS},
    grid::Grid,
    math::Vec2,
    rogue_db::*,
    Stuff,
};

pub struct MapGenProps {
    pub room_min_size: u32,
    pub room_max_size: u32,
    pub max_rooms: u32,
    pub max_monsters_per_room: u32,
}

fn init_entity(pos: Vec2, tag: StuffTag, world: &mut Db, grid: &mut Grid<Stuff>) {
    let id = world.spawn_entity();
    grid[pos] = Some(id.into());
    world.insert(id, tag);
    world.insert(id, Pos(pos));
    match tag {
        StuffTag::Player => {
            unreachable!()
        }
        StuffTag::Wall => {
            world.insert(id, ICONS["wall"]);
        }
        StuffTag::Troll => {
            world.insert(id, Hp::new(8));
            world.insert(id, ICONS["troll"]);
            world.insert(id, Ai);
            world.insert(id, MeleeAi { power: 3 });
        }
        StuffTag::Orc => {
            world.insert(id, Hp::new(4));
            world.insert(id, ICONS["orc-head"]);
            world.insert(id, Ai);
            world.insert(id, MeleeAi { power: 1 });
        }
    }
}

fn place_entities(
    rng: &mut impl Rng,
    grid: &mut Grid<Option<StuffTag>>,
    room: &RectRoom,
    max_monsters: u32,
) {
    let n_monsters = rng.gen_range(0, max_monsters + 1);

    for _ in 0..n_monsters {
        let x = rng.gen_range(room.min.x + 1, room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1, room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = ENEMY_TAGS.choose(rng).unwrap();
            grid[pos] = Some(*tag);
        }
    }
}

pub fn generate_map(
    player_id: EntityId,
    world: &mut Db,
    grid: &mut Grid<Stuff>,
    props: MapGenProps,
) {
    let mut working_set = Grid::new(grid.dims());
    // fill the map with walls and delete old entities
    //
    for (p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff {
            // delete all but player entities from the database
            let id: EntityId = (*id).into();
            if id != player_id {
                world.delete_entity(id);
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
                world.insert(player_id, Pos(pos));
                grid[pos] = Some(player_id.into());
            }
            _ => {
                init_entity(pos, tag, world, grid);
            }
        }
    }
}

fn build_rooms(grid: &mut Grid<Option<StuffTag>>, props: &MapGenProps) {
    let mut rng = rand::thread_rng();
    let mut rooms = Vec::<RectRoom>::with_capacity(props.max_rooms as usize);

    'outer: for _ in 0..props.max_rooms {
        let w = rng.gen_range(props.room_min_size, props.room_max_size) as i32;
        let h = rng.gen_range(props.room_min_size, props.room_max_size) as i32;

        let x = rng.gen_range(1, grid.width() - 1 - w);
        let y = rng.gen_range(1, grid.height() - 1 - h);

        let room = RectRoom::new(x, y, w, h);
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
