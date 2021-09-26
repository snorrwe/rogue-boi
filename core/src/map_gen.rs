use cao_db::prelude::*;
use rand::Rng;

use crate::{
    components::{Icon, Pos, StuffTag},
    grid::GameGrid,
    math::Vec2,
    rogue_db::*,
    Stuff, StuffPayload,
};

pub fn generate_map(world: &mut Db, grid: &mut GameGrid) {
    // fill the map with walls and delete old entities
    //
    for (_p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff.id.take() {
            // delete all but player entities from the database
            if !matches!(stuff.payload, StuffPayload::Player) {
                world.delete_entity(id.val.into());
            }
        }
        stuff.payload = StuffPayload::Wall;
    }

    // build rooms
    //
    build_rooms(grid);

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
                world.insert(stuff.id.expect("player id").into(), pos);
            }
        }
    }
}

fn build_rooms(grid: &mut GameGrid) {
    let mut rng = rand::thread_rng();

    let room0 = RectRoom::new(16, 15, 10, 15);
    room0.carve(grid);
    let room1 = RectRoom::new(1, 1, 3, 4);
    room1.carve(grid);

    for p in tunnel_between(&mut rng, room0.center(), room1.center()) {
        grid[p] = Stuff::default();
    }
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

struct TunnelIter {
    current: Vec2,
    end: Vec2,
    corner: Vec2,
}

impl TunnelIter {
    fn new(start: Vec2, end: Vec2, corner: Vec2) -> Self {
        Self {
            current: start,
            end,
            corner,
        }
    }
}

impl Iterator for TunnelIter {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let dc = self.corner - self.current;
        let dc = dc.as_direction();
        let de = self.end - self.current;
        let de = de.as_direction();

        if dc.x == -de.x || dc.y == -de.y {
            // we're between the corner and the end, move towards end
            self.current += de;
        } else {
            // move towards the corner
            self.current += dc;
        }

        Some(self.current)
    }
}

fn insert_wall(pos: Vec2, w: &mut Db) -> EntityId {
    let id = w.spawn_entity();
    w.insert(id, StuffTag::Wall);
    w.insert(id, Pos(pos));
    w.insert(id, Icon("delapouite/brick-wall.svg"));
    id
}

struct RectRoom {
    pub(crate) x1: i32,
    pub(crate) y1: i32,
    pub(crate) x2: i32,
    pub(crate) y2: i32,
}

impl RectRoom {
    pub fn new(x1: i32, y1: i32, width: i32, height: i32) -> Self {
        Self {
            x1,
            y1,
            x2: x1 + width,
            y2: y1 + height,
        }
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new((self.x2 + self.x1) / 2, (self.y2 + self.y1) / 2)
    }

    /// carve out this room
    pub fn carve(&self, grid: &mut GameGrid) {
        for y in self.y1..=self.y2 {
            for x in self.x1..=self.x2 {
                grid[Vec2::new(x, y)] = Stuff::default();
            }
        }
    }
}
