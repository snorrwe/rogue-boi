use crate::{
    archetypes::{icon, init_entity},
    components::*,
    game_log,
    grid::Grid,
    math::{walk_square, Vec2},
    pathfinder::find_path,
    InputEvent, OutputStuff, PlayerActions, PlayerOutput, RenderedOutput, Stuff, StuffPayload,
};
use cao_db::prelude::*;
use rand::Rng;
use tracing::{debug, error, info};
use wasm_bindgen::{prelude::*, JsCast, JsValue};

pub fn update_input_events(inputs: Res<Vec<InputEvent>>, mut actions: ResMut<PlayerActions>) {
    let mut delta = Vec2::new(0, 0);
    for event in &inputs[..] {
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
    if delta != Vec2::ZERO {
        actions.insert_move(delta)
    }
}

pub fn update_player_item_use(
    actions: Res<PlayerActions>,
    mut cmd: Commands,
    player_query: Query<(EntityId, &mut Pos, &mut Inventory), With<PlayerTag>>,
    stuff_tags: Query<&StuffTag>,
    hp_query: Query<&mut Hp>,
    heal_query: Query<&Heal>,
    target_query: Query<(&Pos, &mut Hp)>,
    item_query: Query<Option<&Ranged>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
) {
    let (player_id, pos, inventory) = player_query.iter().next().unwrap();
    if let Some(id) = actions.use_item_action() {
        let tag = stuff_tags.fetch(id);
        match tag {
            Some(StuffTag::HpPotion) => {
                game_log!("Drink a health potion.");
                inventory.remove(id);
                let hp = hp_query.fetch(player_id).unwrap();
                if hp.full() {
                    game_log!("The potion has no effect");
                }
                let heal = heal_query.fetch(id).unwrap();
                hp.current = (hp.current + heal.hp).min(hp.max);
                cmd.delete(id);
            }
            Some(StuffTag::LightningScroll) => match actions.target() {
                Some(target_id) => {
                    debug!("Use lightning scroll {}", id);
                    let (target_pos, target_hp) = match target_query.fetch(target_id) {
                        Some(x) => x,
                        None => {
                            game_log!("Invalid target for lightning bolt");
                            should_run.0 = false;
                            return;
                        }
                    };
                    let range = item_query.fetch(id).unwrap();
                    let range = range.unwrap();
                    if target_pos.0.chebyshev(pos.0) > range.range {
                        game_log!("Target is too far away");
                        should_run.0 = false;
                        return;
                    }
                    if skill_check(range.skill) {
                        let dmg = range.power;
                        target_hp.current -= dmg;
                        game_log!("Lightning bolt hits {} for {} damage!", target_id, dmg);
                    } else {
                        game_log!("Lightning bolt misses!");
                    }
                    inventory.remove(id);
                    cmd.delete(id);
                }
                None => {
                    game_log!("Lightning bolt has no target!");
                    error!("Lightning bolt has no target!");
                    should_run.0 = false;
                    return;
                }
            },
            None => {
                error!("Item has no stuff tag");
                inventory.remove(id);
            }
            _ => {
                unreachable!("Bad item use")
            }
        }
    }
}

pub fn update_player_inventory(
    q_melee: Query<&mut Melee>,
    q_inventory: Query<(EntityId, &Inventory), With<PlayerTag>>,
) {
    for (player_id, inventory) in q_inventory.iter() {
        let mut power = 1;
        for item in inventory.items.iter().copied() {
            if let Some(melee_weapon) = q_melee.fetch(item) {
                power += melee_weapon.power;
            }
        }
        q_melee.fetch(player_id).unwrap().power = power;
    }
}

pub fn handle_player_move(
    actions: Res<PlayerActions>,
    mut cmd: Commands,
    player_q: Query<(&mut Inventory, &Melee, &mut Pos), With<PlayerTag>>,
    stuff_tags: Query<&StuffTag>,
    hp: Query<&mut Hp>,
    mut grid: ResMut<Grid<Stuff>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
) {
    let delta = match actions.move_action() {
        Some(x) => x,
        None => {
            return;
        }
    };
    for (inventory, power, pos) in player_q.iter() {
        let pos = &mut pos.0;
        let new_pos: Vec2 = *pos + delta;
        match grid.at(new_pos.x, new_pos.y).unwrap().and_then(|id| {
            let stuff_id = id.into();
            stuff_tags.fetch(stuff_id).map(|tag| (stuff_id, tag))
        }) {
            Some((stuff_id, tag)) => {
                match tag {
                    StuffTag::Player => unreachable!(),
                    StuffTag::Wall => {
                        game_log!("Can't move into wall");
                        should_run.0 = false;
                    }
                    StuffTag::Troll | StuffTag::Orc => {
                        if skill_check(power.skill) {
                            let hp = hp.fetch(stuff_id).expect("Enemy has no hp");
                            let power = power.power;
                            hp.current -= power;
                            debug!("kick enemy {}: {:?}", stuff_id, hp);
                            game_log!("Bonk enemy {} for {} damage", stuff_id, power);
                        } else {
                            debug!("miss enemy {}", stuff_id);
                            game_log!("Your attack misses");
                        }
                    }
                    StuffTag::LightningScroll | StuffTag::HpPotion | StuffTag::Sword => {
                        // pick up item
                        if let Err(err) = inventory.add(stuff_id) {
                            match err {
                                crate::components::InventoryError::Full => {
                                    game_log!("Inventory is full");
                                    should_run.0 = false;
                                }
                            }
                        }
                        grid_step(pos, new_pos, &mut grid);
                        // remove the position component of the item
                        cmd.entity(stuff_id).remove::<Pos>();
                        game_log!("Picked up a {:?}", tag);
                    }
                }
            }
            None => {
                // empty position
                // update the grid asap so the monsters will see the updated player position
                grid_step(pos, new_pos, &mut grid);
            }
        }
    }
}

fn grid_step(pos: &mut Vec2, new_pos: Vec2, grid: &mut Grid<Stuff>) {
    let old_stuff = std::mem::take(&mut grid[*pos]);
    grid[new_pos] = old_stuff;
    *pos = new_pos;
    game_log!("Step on tile: {}", new_pos);
}

/// return wether the segment hits something and where
fn walk_grid_on_segment(
    from: Vec2,
    to: Vec2,
    grid: &Grid<Stuff>,
    tags: &Query<&StuffTag>,
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
            .and_then(|id| tags.fetch(*id).filter(|tag| tag.is_opaque()))
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
    tags: &Query<&StuffTag>,
    player_pos: Vec2,
    radius: i32,
) {
    visible.splat_set([Vec2::ZERO, visible.dims()], false);
    // walk the visible range
    walk_square(-Vec2::splat(radius), Vec2::splat(radius))
        .map(|d| player_pos + d)
        .for_each(|limit| {
            if walk_grid_on_segment(player_pos, limit, grid, &tags).is_none() {
                if let Some(visible) = visible.at_mut(limit.x, limit.y) {
                    *visible = true;
                }
            }
        });
}

/// go over the visible range and if an item is adjacent to a visible empty tile, then set that to
/// visible as well
fn flood_vizibility(grid: &Grid<Stuff>, visible: &mut Grid<bool>, player_pos: Vec2, radius: i32) {
    let _s = tracing::debug_span!("flood").entered();

    let mut to_update = smallvec::SmallVec::<[_; 64]>::new();
    walk_square(-Vec2::splat(radius), Vec2::splat(radius))
        .map(|d| player_pos + d)
        .for_each(|pos| {
            if visible.at(pos.x, pos.y).copied().unwrap_or(false) {
                return;
            }
            walk_square(-Vec2::ONE, Vec2::ONE)
                .map(|d| pos + d)
                .for_each(|new_pos| {
                    if visible.at(new_pos.x, new_pos.y).copied().unwrap_or(false)
                        && grid[new_pos].is_none()
                    {
                        to_update.push(pos);
                    }
                });
        });
    for pos in to_update {
        visible[pos] = true;
    }
}

/// recompute visible area
pub fn update_fov(
    q: Query<&Pos, With<PlayerTag>>,
    tags_q: Query<&StuffTag>,
    grid: Res<Grid<Stuff>>,
    mut explored: ResMut<Explored>,
    mut visible: ResMut<Visible>,
    viewport: Res<Visibility>,
) {
    let radius = viewport.0.x.max(viewport.0.y);
    if let Some(player_pos) = q.iter().next() {
        set_visible(&grid, &mut visible.0, &tags_q, player_pos.0, radius);
        visible.0[player_pos.0] = true;
        flood_vizibility(&grid, &mut visible.0, player_pos.0, radius);
        explored.0.or_eq(&visible.0);
    }
}

pub fn update_grid(q: Query<(EntityId, &Pos)>, mut grid: ResMut<Grid<Stuff>>) {
    // zero out the map
    grid.fill(Default::default());
    for (id, pos) in q.iter() {
        let pos = pos.0;
        grid[pos] = Some(id.into());
    }
}

pub fn update_melee_ai(
    q_player: Query<(&mut Hp, &Pos), With<PlayerTag>>,
    q_enemy: Query<(EntityId, &Melee, &mut Pos, &mut PathCache, Option<&Leash>), With<Ai>>,
    q_tag: Query<&StuffTag>,
    q_walk: Query<&Walkable>,
    mut grid: ResMut<Grid<Stuff>>,
) {
    let (player_hp, Pos(player_pos)) = match q_player.iter().next() {
        Some(x) => x,
        None => {
            debug!("No player on the map! Skipping melee update");
            return;
        }
    };

    for (id, Melee { power, skill }, Pos(pos), cache, leash) in q_enemy.iter() {
        if pos.manhatten(*player_pos) <= 1 {
            if !skill_check(*skill) {
                game_log!("{} misses", id);
                continue;
            }

            player_hp.current -= power;
            game_log!("{} hits you for {} damage", id, power);
            cache.path.clear();
        } else if walk_grid_on_segment(*pos, *player_pos, &grid, &q_tag).is_none() {
            debug!("Player is visible, finding path");
            cache.path.clear();
            cache.path.push(*player_pos); // push the last pos, so entities can follow players
                                          // across corridors
            if !find_path(*pos, *player_pos, &grid, &q_walk, &mut cache.path) {
                // finding path failed, pop the player pos
                cache.path.clear();
            }
            // if the distance to the player is 1
            // there is a bug in pathfinding that returns the current pos as the last
            while cache.path.last() == Some(pos) {
                cache.path.pop();
            }
        } else if cache.path.is_empty() {
            // if the enemy has a leash and the player is not visible, return to the origin
            if let Some(leash) = leash {
                cache.path.clear();
                find_path(*pos, leash.origin, &grid, &q_walk, &mut cache.path);
            }
        }
        if let Some(mut new_pos) = cache.path.pop() {
            if let Some(leash) = leash {
                // if at the end of leash, don't move
                if new_pos.manhatten(leash.origin) > leash.radius {
                    cache.path.clear();
                    new_pos = *pos;
                }
            }

            if grid[new_pos].is_some() {
                // taken
                cache.path.clear();
            } else {
                grid[*pos] = None;
                grid[new_pos] = Some(id);
                *pos = new_pos;
            }
        }
    }
}

pub fn update_player_hp(
    mut cmd: Commands,
    query_player: Query<(EntityId, &Hp, &mut Icon), With<PlayerTag>>,
) {
    for (player_id, hp, i) in query_player.iter() {
        if hp.current <= 0 {
            info!("Player died");
            game_log!("Player died");
            *i = icon("tombstone");
            cmd.entity(player_id).remove::<PlayerTag>();
        }
    }
}

pub fn update_ai_hp(
    mut cmd: Commands,
    query_hp: Query<(EntityId, &Hp), (With<Ai>, WithOut<PlayerTag>)>,
) {
    for id in query_hp
        .iter()
        .filter_map(|(id, hp)| (hp.current <= 0).then_some(id))
    {
        debug!("Entity {} died", id);
        game_log!("{} died", id);

        cmd.delete(id);
    }
}

/// Throw a D6, if result is >= skill then the check passes
pub fn skill_check(skill: i32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=6) >= skill
}

pub fn update_tick(mut t: ResMut<GameTick>) {
    t.0 += 1;
}

pub fn should_update_world(r: Res<ShouldUpdateWorld>) -> bool {
    r.0
}

pub fn rotate_log() {
    crate::logging::rotate_log();
}

pub fn update_camera_pos(mut camera: ResMut<CameraPos>, q: Query<&Pos, With<PlayerTag>>) {
    for pos in q.iter() {
        camera.0 = pos.0;
    }
}

pub fn update_output(
    q_player: Query<(&Pos, &Hp, &Melee), With<PlayerTag>>,
    game_tick: Res<GameTick>,
    mut output_cache: ResMut<Output>,
) {
    let _span = tracing::span!(tracing::Level::DEBUG, "update_output").entered();

    let player = q_player
        .iter()
        .next()
        .map(|(pos, hp, attack)| PlayerOutput {
            player_hp: *hp,
            player_attack: attack.power,
            player_pos: pos.0,
        });
    let log = crate::logging::compute_log(game_tick.0 as usize);
    let result = RenderedOutput { player, log };
    output_cache.0 = JsValue::from_serde(&result).unwrap();
}

pub fn init_player(mut cmd: Commands, mut grid: ResMut<Grid<Stuff>>) {
    let dims = grid.dims() / 2;
    init_entity(dims, StuffTag::Player, &mut cmd, &mut grid);
}

pub fn should_update_player(s: Res<ShouldUpdatePlayer>) -> bool {
    s.0
}

pub fn player_prepare(
    mut should_update: ResMut<ShouldUpdateWorld>,
    mut player_id: ResMut<PlayerId>,
    q: Query<EntityId, With<PlayerTag>>,
    mut should_update_player: ResMut<ShouldUpdatePlayer>,
    actions: Res<PlayerActions>,
) {
    // if no player is found then don't update player logic
    should_update_player.0 = false;
    should_update.0 = true;
    for id in q.iter() {
        player_id.0 = id;
        should_update_player.0 = true;
        break;
    }
    if should_update_player.0 {
        if actions.wait() {
            game_log!("Waiting...");
            should_update_player.0 = false;
            return;
        }
    }
}

pub fn render_into_canvas(
    mut res: ResMut<RenderResources>,
    grid: Res<Grid<Stuff>>,
    viewport: Res<Viewport>,
    camera_pos: Res<CameraPos>,
    visible: Res<Visible>,
    explored: Res<Explored>,
    stuff: Query<(&StuffTag, &Icon, Option<&Color>)>,
    icons: Res<IconCollection>,
) {
    let width = res.width as f64;
    let height = res.height as f64;
    let ctx = match res.ctx.as_mut() {
        Some(x) => x,
        None => {
            debug!("No rendering context, skipping render");
            return;
        }
    };

    ctx.set_fill_style(&"gray".into());
    ctx.fill_rect(0.0, 0.0, width, height);
    let min = camera_pos.0 - viewport.0;
    let max = camera_pos.0 + viewport.0;
    let cell_size = height.min(width) / (viewport.0.y * 2) as f64;
    let icon_scale = cell_size / 512.0;
    for y in min.y.max(0)..max.y.min(grid.height()) {
        for x in min.x.max(0)..max.x.min(grid.width()) {
            let pos = Vec2::new(x, y);
            let visible = visible.0[pos];
            let explored = explored.0[pos];

            let render_pos = pos - min;
            let render_x = render_pos.x as f64 * cell_size;
            let render_y = render_pos.y as f64 * cell_size;

            if visible {
                ctx.set_fill_style(&"yellow".into());
            } else {
                ctx.set_fill_style(&"darkgray".into());
            }

            if explored {
                match grid[pos].and_then(|id| stuff.fetch(id).map(|x| (id, x.clone()))) {
                    Some((_id, (tag, icon, color))) => {
                        // icon background
                        if visible {
                            ctx.set_fill_style(&"black".into());
                        } else {
                            ctx.set_fill_style(&"darkgray".into());
                        }
                        ctx.fill_rect(render_x, render_y, cell_size, cell_size);
                        // render icon
                        if visible || tag.static_visiblity() {
                            ctx.fill_rect(render_x, render_y, cell_size, cell_size);
                            match icons.0.get(icon.0) {
                                Some(icon) => {
                                    match color {
                                        Some(Color(ref color)) => {
                                            ctx.set_fill_style(color);
                                        }
                                        None => {
                                            ctx.set_fill_style(&"white".into());
                                        }
                                    }
                                    ctx.save();
                                    ctx.translate(render_x, render_y).unwrap();
                                    ctx.scale(icon_scale, icon_scale).unwrap();
                                    ctx.fill_with_path_2d(&icon);
                                    ctx.restore();
                                }
                                None => {
                                    // if icon can not be fetched
                                    if let Some(Color(ref color)) = color {
                                        ctx.set_fill_style(color);
                                    }
                                    ctx.fill_rect(render_x, render_y, cell_size, cell_size);
                                }
                            }
                        }
                    }
                    None => {
                        // empty space
                        if visible {
                            ctx.set_fill_style(&"black".into());
                        }
                        ctx.fill_rect(render_x, render_y, cell_size, cell_size);
                    }
                }
            }
        }
    }
}
