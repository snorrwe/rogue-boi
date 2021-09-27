use crate::{
    components::{Pos, StuffTag},
    grid::Grid,
    math::Vec2,
    rogue_db::*,
    Id, InputEvent, Stuff, StuffPayload,
};
use cao_db::prelude::*;

pub fn update_player(
    inputs: &[InputEvent],
    player: EntityId,
    q: Query<Pos>,
    grid: &mut Grid<Stuff>,
) {
    let mut delta = Vec2::new(0, 0);

    for event in inputs {
        match event {
            InputEvent::KeyDown { key } if key == "w" => delta.y = -1,
            InputEvent::KeyDown { key } if key == "s" => delta.y = 1,
            InputEvent::KeyDown { key } if key == "a" => delta.x = -1,
            InputEvent::KeyDown { key } if key == "d" => delta.x = 1,
            _ => {}
        }
    }

    if delta.x != 0 && delta.y != 0 {
        delta.x = 0;
    }
    if delta.x != 0 || delta.y != 0 {
        let q = q.into_inner();
        let pos = &mut q.get_mut(player).expect("Failed to get player pos").0;

        let new_pos = *pos + delta;
        if let Some(tile) = grid.at(new_pos.x, new_pos.y) {
            match tile.payload {
                StuffPayload::Empty => {
                    // update the grid asap so the monsters will see the updated player position
                    let old_stuff = std::mem::take(&mut grid[*pos]);
                    grid[new_pos] = old_stuff;

                    *pos = new_pos;
                }
                StuffPayload::Wall => { /* don't step */ }
                StuffPayload::Player => unreachable!(),
            }
        }
    }
}

pub fn update_grid(q: Query<(EntityId, Pos, StuffTag)>, grid: &mut Grid<Stuff>) {
    let w = grid.width();
    let h = grid.height();
    assert!(w > 0 && h > 0);
    // zero out the map
    grid.fill(Default::default());

    let q = q.into_inner();
    let it1 = q.1.iter();
    let it2 = q.2.iter();
    for (idx, (pos, tag)) in join!(it1, it2) {
        let tag: &StuffTag = tag;
        let id = q.0.id_at_index(idx);
        let pos = pos.0;

        let payload = match tag {
            StuffTag::Player => StuffPayload::Player,
            StuffTag::Wall => StuffPayload::Wall,
        };

        grid[Vec2::new(pos.x, pos.y)] = Stuff {
            id: Some(Id { val: id.into() }),
            payload,
        };
    }
}
