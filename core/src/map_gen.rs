use cao_db::prelude::*;

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
        if let Some(id) = stuff.id {
            // delete all but player entities from the database
            if !matches!(stuff.payload, StuffPayload::Player) {
                world.delete_entity(id.val.into());
            }
        }
        *stuff = Stuff {
            id: None,
            payload: StuffPayload::Wall,
        };
    }

    // build rooms
    //
    let room = RectRoom::new(16, 15, 10, 15);
    room.carve(grid);
    let room = RectRoom::new(1, 1, 3, 4);
    room.carve(grid);

    // insert entities into db
    //
    for (pos, stuff) in grid.iter_mut() {
        let Vec2 { x, y } = pos;
        match stuff.payload {
            StuffPayload::Wall => {
                let id = insert_wall(x, y, world);
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

fn insert_wall(x: i32, y: i32, w: &mut Db) -> EntityId {
    let pos = Vec2::new(x, y);

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
