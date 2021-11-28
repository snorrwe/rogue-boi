use crate::{
    components::{Ai, Hp, Icon, MeleeAi, PlayerTag, Pos, StuffTag, Walkable, ICONS},
    grid::Grid,
    math::Vec2,
    pathfinder::find_path,
    rogue_db::*,
    InputEvent, Stuff,
};
use cao_db::prelude::*;
use smallvec::SmallVec;
use tracing::{debug, info, trace};

pub fn init_player(world: &mut Db) -> EntityId {
    let player = world.spawn_entity();
    world.insert(player, StuffTag::Player);
    world.insert(player, Pos(Vec2::new(16, 16)));
    world.insert(player, ICONS["person"]);
    world.insert(player, Hp::new(10));
    world.insert(player, PlayerTag);

    player
}

pub fn update_player(
    inputs: &[InputEvent],
    query: Query<(Pos, StuffTag, Hp, PlayerTag)>,
    grid: &mut Grid<Stuff>,
) {
    let (mut pos, stuff_tags, mut hp, player_tag) = query.into_inner();
    for (_idx, (Pos(ref mut pos), _tag)) in join!(pos.iter_mut(), player_tag.iter()) {
        let mut delta = Vec2::new(0, 0);

        for event in inputs {
            match event {
                InputEvent::KeyDown { key } if key == "w" || key == "ArrowUp" => delta.y = -1,
                InputEvent::KeyDown { key } if key == "s" || key == "ArrowDown" => delta.y = 1,
                InputEvent::KeyDown { key } if key == "a" || key == "ArrowLeft" => delta.x = -1,
                InputEvent::KeyDown { key } if key == "d" || key == "ArrowRight" => delta.x = 1,
                _ => {}
            }
        }

        if delta.x != 0 && delta.y != 0 {
            delta.x = 0;
        }
        if delta.x != 0 || delta.y != 0 {
            let new_pos = *pos + delta;
            match grid.at(new_pos.x, new_pos.y) {
                Some(Some(id)) => {
                    let id = id.into();
                    match stuff_tags.get(id).unwrap() {
                        StuffTag::Player => unreachable!(),
                        StuffTag::Wall => { /* don't step */ }
                        StuffTag::Troll | StuffTag::Orc => {
                            let hp = hp.get_mut(id).expect("Enemy has no hp");
                            hp.current -= 1;
                            debug!("kick enemy {}: {:?}", id, hp)
                        }
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
}

/// return wether the segment hits something and where
fn walk_grid_on_segment(
    from: Vec2,
    to: Vec2,
    grid: &Grid<Stuff>,
    tags: &ComponentFrag<StuffTag>,
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
    // skip the first pos
    step(&mut p, &mut ix, &mut iy, nx, ny, sign_x, sign_y);
    while ix < nx || iy < ny {
        // if there is an entity at this position and the entity is opaque
        if grid
            .at(p.x, p.y)
            .and_then(|x| x.as_ref())
            .and_then(|id| tags.get(id.into()).filter(|tag| tag.is_opaque()))
            .is_some()
        {
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

fn set_visible(
    grid: &Grid<Stuff>,
    visible: &mut Grid<bool>,
    tags: ComponentFrag<StuffTag>,
    player_pos: Vec2,
    radius: i32,
) {
    visible.splat_set([Vec2::ZERO, visible.dims()], false);
    // walk the visible range
    for y in -radius..=radius {
        for x in -radius..=radius {
            let limit = player_pos + Vec2::new(x, y);
            if walk_grid_on_segment(player_pos, limit, grid, &tags).is_none() {
                if let Some(visible) = visible.at_mut(limit.x, limit.y) {
                    *visible = true;
                }
            }
        }
    }
}

/// go over the visible range and if an item is adjacent to a visible empty tile, then set that to
/// visible as well
fn flood_vizibility(grid: &Grid<Stuff>, visible: &mut Grid<bool>, player_pos: Vec2, radius: i32) {
    let _s = tracing::debug_span!("flood").entered();

    let mut to_update = smallvec::SmallVec::<[_; 64]>::new();
    for y in -radius..=radius {
        for x in -radius..=radius {
            let pos = player_pos + Vec2::new(x, y);
            if visible.at(pos.x, pos.y).copied().unwrap_or(false) {
                continue;
            }
            for y in -1..=1 {
                for x in -1..=1 {
                    if visible.at(pos.x + x, pos.y + y).copied().unwrap_or(false)
                        && grid[pos + Vec2::new(x, y)].is_none()
                    {
                        to_update.push(pos);
                    }
                }
            }
        }
    }
    for pos in to_update {
        visible[pos] = true;
    }
}

/// recompute visible area
pub fn update_fov(
    player: EntityId,
    q: Query<(StuffTag, Pos)>,
    grid: &Grid<Stuff>,
    explored: &mut Grid<bool>,
    visible: &mut Grid<bool>,
) {
    const RADIUS: i32 = 8;

    let q = q.into_inner();
    let player_pos = q.1.get(player).unwrap();
    set_visible(&grid, visible, q.0, player_pos.0, RADIUS);
    visible[player_pos.0] = true;
    flood_vizibility(&grid, visible, player_pos.0, RADIUS);
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

pub fn update_melee_ai(
    q: Query<(EntityId, Pos, MeleeAi, StuffTag, Hp, Walkable, PlayerTag)>,
    grid: &mut Grid<Stuff>,
) {
    let (ids, mut pos, ai, tags, mut hp, walkable, player_tag) = q.into_inner();
    let player_id = match player_tag
        .iter()
        .next()
        .map(|(idx, _tag)| ids.id_at_index(idx))
    {
        Some(id) => id,
        None => {
            debug!("No player on the map! Skipping melee update");
            return;
        }
    };
    let player_hp = hp.get_mut(player_id).expect("Failed to get player hp");

    let Pos(player_pos) = *pos.get(player_id).expect("Failed to get player pos");

    for (idx, (MeleeAi { power }, (_tag, Pos(pos)))) in
        join!(ai.iter(), tags.iter(), pos.iter_mut())
    {
        let id = ids.id_at_index(idx);
        if pos.chebyshev(player_pos) <= 1 {
            player_hp.current -= power;
            debug!(
                "bonk the player with power {}. Player hp: {:?}",
                power, player_hp
            );
        } else if walk_grid_on_segment(*pos, player_pos, grid, &tags).is_none() {
            // TODO: pathfinder
            let mut path = SmallVec::new(); // TODO: cache paths?
            find_path(*pos, player_pos, grid, &walkable, &mut path);
            trace!("walk towards player {:?}", path);

            path.pop(); // the first position is the monster itself
            if let Some(new_pos) = path.pop() {
                grid[*pos] = None;
                grid[new_pos] = Some(id.into());
                *pos = new_pos;
            }
        }
    }
}

pub fn update_hp(world: &mut Db) {
    // update AI hps
    //
    let mut delete_list = smallvec::SmallVec::<[_; 4]>::new();
    let query = Query::<(EntityId, Hp, Ai)>::new(&world);
    let (ids, hps, ai) = query.into_inner();
    join!(hps.iter(), ai.iter())
        .filter(|(_, (hp, _ai))| hp.current <= 0)
        .into_iter()
        .for_each(|(idx, _)| {
            delete_list.push(ids.id_at_index(idx));
        });
    delete_list.into_iter().for_each(|id| {
        debug!("Entity {} died", id);
        world.delete_entity(id);
    });

    // update Player hp
    //
    let query = Query::<(EntityId, Hp, PlayerTag, Icon)>::new(&world);
    let (ids, hps, mut tags, mut icons) = query.into_inner();
    for (idx, (hp, (_tag, icon))) in join!(hps.iter(), tags.iter(), icons.iter_mut()) {
        let player_id = ids.id_at_index(idx);
        if hp.current <= 0 {
            info!("Player died");
            tags.remove(player_id);
            *icon = ICONS["tombstone"];
        }
    }
}
