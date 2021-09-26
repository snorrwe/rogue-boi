mod rect_room;
mod tunnel_iter;

use self::rect_room::RectRoom;
use self::tunnel_iter::TunnelIter;
use cao_db::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::{
    components::{Icon, Pos, StuffTag},
    grid::GameGrid,
    math::Vec2,
    rogue_db::*,
    Stuff, StuffPayload,
};

pub struct MapGenProps {
    pub room_min_size: u32,
    pub room_max_size: u32,
    pub max_rooms: u32,
}

pub fn generate_map(player_id: EntityId, world: &mut Db, grid: &mut GameGrid, props: MapGenProps) {
    // fill the map with walls and delete old entities
    //
    for (_p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff.id.take() {
            // delete all but player entities from the database
            if !matches!(stuff.payload, StuffPayload::Player) {
                world.delete_entity(id.into());
            }
        }
        stuff.payload = StuffPayload::Wall;
    }

    // build rooms
    //
    build_rooms(grid, &props);

    // insert entities into db
    //
    for (pos, stuff) in grid.iter_mut() {
        match stuff.payload {
            StuffPayload::Wall => {
                let id = insert_wall(pos, world);
                stuff.id = Some(id.into());
            }
            StuffPayload::Empty => {}
            StuffPayload::Player => {
                // update player pos
                let pos = Pos(pos);
                stuff.id = Some(player_id.into());
                world.insert(player_id, pos);
            }
        }
    }
}

fn build_rooms(grid: &mut GameGrid, props: &MapGenProps) {
    let mut rng = rand::thread_rng();
    let mut rooms = Vec::<RectRoom>::with_capacity(props.max_rooms as usize);

    'outer: for _ in 0..props.max_rooms {
        let w = rng.gen_range(props.room_min_size, props.room_max_size) as i32;
        let h = rng.gen_range(props.room_min_size, props.room_max_size) as i32;

        let x = rng.gen_range(1, grid.dims.x - 1 - w);
        let y = rng.gen_range(1, grid.dims.y - 1 - h);

        let room = RectRoom::new(x, y, w, h);
        for r in rooms.iter() {
            if room.intersects(&r) {
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
            grid[p] = Stuff::default();
        }
    }

    // spawn the player in the first room
    grid[rooms[0].center()].payload = StuffPayload::Player;
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

fn insert_wall(pos: Vec2, w: &mut Db) -> EntityId {
    let id = w.spawn_entity();
    w.insert(id, StuffTag::Wall);
    w.insert(id, Pos(pos));
    w.insert(id, Icon("delapouite/brick-wall.svg"));
    id
}
