use crate::{
    components::{Pos, StuffTag},
    grid::Grid,
    math::Vec2,
    rogue_db::*,
    InputEvent, Stuff,
};
use cao_db::prelude::*;

pub fn update_player(
    inputs: &[InputEvent],
    player: EntityId,
    q: Query<(Pos, StuffTag)>,
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
        let pos = &mut q.0.get_mut(player).expect("Failed to get player pos").0;

        let new_pos = *pos + delta;
        match grid.at(new_pos.x, new_pos.y) {
            Some(Some(id)) => {
                match q.1.get(id.into()).unwrap() {
                    StuffTag::Player => unreachable!(),
                    StuffTag::Wall => { /* don't step */ }
                    StuffTag::Troll | StuffTag::Orc => todo!("hit the enemy"),
                }
            }
            Some(None) => {
                // empty position

                // update the grid asap so the monsters will see the updated player position
                let old_stuff = std::mem::take(&mut grid[*pos]);
                grid[new_pos] = old_stuff;

                *pos = new_pos;
            }
            None => {}
        }
    }
}

/// return wether the segment hits something and where
fn walk_grid_on_segment(
    from: Vec2,
    to: Vec2,
    grid: &Grid<Stuff>,
    skip_initial: bool,
) -> Option<Vec2> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    let sign_x = if dx > 0 { 1 } else { -1 };
    let sign_y = if dy > 0 { 1 } else { -1 };

    let nx = dx.abs() as f32;
    let ny = dy.abs() as f32;

    let mut p = from;
    let mut ix = 0.0;
    let mut iy = 0.0;
    if skip_initial {
        step(&mut p, &mut ix, &mut iy, nx, ny, sign_x, sign_y);
    }
    while ix < nx || iy < ny {
        if grid.at(p.x, p.y).and_then(|x| x.as_ref()).is_some() {
            return Some(p);
        }
        step(&mut p, &mut ix, &mut iy, nx, ny, sign_x, sign_y);
    }
    None
}

fn step(p: &mut Vec2, ix: &mut f32, iy: &mut f32, nx: f32, ny: f32, sign_x: i32, sign_y: i32) {
    if (0.5 + *ix) / nx < (0.5 + *iy) / ny {
        // step horizontal
        p.x += sign_x;
        *ix += 1.0;
    } else {
        //vertical
        p.y += sign_y;
        *iy += 1.0;
    }
}

fn set_visible(grid: &Grid<Stuff>, visible: &mut Grid<bool>, player_pos: Vec2, radius: i32) {
    visible.splat_set([Vec2::ZERO, visible.dims()], false);
    // walk the visible range
    for y in -radius..=radius {
        for x in -radius..=radius {
            let limit = player_pos + Vec2::new(x, y);
            match walk_grid_on_segment(player_pos, limit, grid, true) {
                None => {
                    if let Some(visible) = visible.at_mut(limit.x, limit.y) {
                        *visible = true;
                    }
                }
                _ => {}
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
    set_visible(&grid, visible, player_pos.0, 10);
    visible[player_pos.0] = true;
    explored.or_eq(&visible);
}

pub fn update_grid(q: Query<(EntityId, Pos)>, grid: &mut Grid<Stuff>) {
    let w = grid.width();
    let h = grid.height();
    assert!(w > 0 && h > 0);
    // zero out the map
    grid.fill(Default::default());

    let q = q.into_inner();
    for (idx, pos) in q.1.iter() {
        let id = q.0.id_at_index(idx);
        let pos = pos.0;

        grid[Vec2::new(pos.x, pos.y)] = Some(id.into());
    }
}
