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

fn is_transparent(grid: &Grid<Stuff>, pos: Vec2) -> bool {
    match grid.at(pos.x, pos.y).map(|x| &x.payload) {
        Some(StuffPayload::Empty) => true,
        _ => false,
    }
}

/// return wether the segment hits something and where
fn walk_grid(from: Vec2, to: Vec2, grid: &Grid<Stuff>, skip_initial: bool) -> Option<Vec2> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    let nx = dx.abs() as f32;
    let ny = dy.abs() as f32;

    let sign_x = if dx > 0 { 1 } else { -1 };
    let sign_y = if dy > 0 { 1 } else { -1 };

    let mut p = from;
    let mut ix = 0.0;
    let mut iy = 0.0;
    if skip_initial {
        if (0.5 + ix) / nx < (0.5 + iy) / ny {
            // step horizontal
            p.x += sign_x;
            ix += 1.0;
        } else {
            //vertical
            p.y += sign_y;
            iy += 1.0;
        }
    }
    while ix < nx || iy < ny {
        if !is_transparent(grid, p) {
            return Some(p);
        }
        if (0.5 + ix) / nx < (0.5 + iy) / ny {
            // step horizontal
            p.x += sign_x;
            ix += 1.0;
        } else {
            // step vertical
            p.y += sign_y;
            iy += 1.0;
        }
    }
    None
}

fn set_visible(grid: &Grid<Stuff>, visible: &mut Grid<bool>, player_pos: Vec2, radius: i32) {
    visible.splat_set([Vec2::ZERO, visible.dims()], false);
    // walk the visible range
    let r2 = radius * radius;
    for y in -radius..=radius {
        for x in -radius..=radius {
            let limit = player_pos + Vec2::new(x, y);
            if (player_pos - limit).len_sq() <= r2 {
                match walk_grid(player_pos, limit, grid, true) {
                    Some(pos) if (pos - limit).len_sq() <= 2 => {
                        visible[Vec2::new(pos.x, pos.y)] = true
                    }
                    None => visible[Vec2::new(limit.x, limit.y)] = true,
                    _ => {}
                }
            }
        }
    }
}

/// recompute visible area
pub fn update_fov(
    player: EntityId,
    q: Query<Pos>,
    grid: &Grid<Stuff>,
    explored: &mut Grid<bool>,
    visible: &mut Grid<bool>,
) {
    let q = q.into_inner();
    let player_pos = q.get(player).unwrap();
    set_visible(&grid, visible, player_pos.0, 8);
    visible[player_pos.0] = true;
    explored.or_eq(&visible);
    // do not highlight the player
    visible[player_pos.0] = false;
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
