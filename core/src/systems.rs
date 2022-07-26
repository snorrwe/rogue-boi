use crate::{
    archetypes::icon,
    components::*,
    game_log,
    grid::Grid,
    math::{remap_f64, walk_square, Vec2},
    pathfinder::find_path,
    InputEvent, PlayerActions, PlayerOutput, RenderedOutput, Stuff, UseItem,
};
use cecs::prelude::*;
use rand::Rng;
use tracing::{debug, error, info};
use wasm_bindgen::JsValue;

pub fn update_input_events(inputs: Res<Vec<InputEvent>>, mut actions: ResMut<PlayerActions>) {
    let mut delta = Vec2::new(0, 0);
    for event in &inputs[..] {
        match event {
            InputEvent::KeyDown { key } if key == "w" || key == "ArrowUp" => delta.y = -1,
            InputEvent::KeyDown { key } if key == "s" || key == "ArrowDown" => delta.y = 1,
            InputEvent::KeyDown { key } if key == "a" || key == "ArrowLeft" => delta.x = -1,
            InputEvent::KeyDown { key } if key == "d" || key == "ArrowRight" => delta.x = 1,
            InputEvent::KeyDown { key } if key == "e" => actions.insert_interact(),
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

pub fn update_player_item_use<'a>(
    actions: Res<PlayerActions>,
    mut cmd: Commands,
    mut player_query: Query<(EntityId, &'a mut Inventory), With<PlayerTag>>,
    stuff_tags: Query<&StuffTag>,
    mut hp_query: Query<&mut Hp>,
    pos_query: Query<&Pos>,
    heal_query: Query<&Heal>,
    item_query: Query<Option<&Ranged>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    mut app_mode: ResMut<AppMode>,
    names: Query<&Name>,
    mut use_item: ResMut<UseItem>,
) {
    let (player_id, inventory) = player_query.iter_mut().next().unwrap();
    let pos = *pos_query.fetch(player_id).unwrap();

    let mut cleanup = |id, use_item: &mut UseItem| {
        inventory.remove(id);
        cmd.delete(id);
        use_item.0 = None;
    };

    if let Some(id) = use_item.0 {
        debug!("Using item {}", id);
        let tag = stuff_tags.fetch(id);
        match tag {
            Some(StuffTag::HpPotion) => {
                game_log!("Drink a health potion.");
                let hp = hp_query.fetch_mut(player_id).unwrap();
                if hp.full() {
                    game_log!("The potion has no effect");
                }
                let heal = heal_query.fetch(id).unwrap();
                hp.current = (hp.current + heal.hp).min(hp.max);
                cleanup(id, &mut use_item);
            }
            Some(StuffTag::LightningScroll) => match actions.target() {
                Some(target_id) => {
                    debug!("Use lightning scroll {}", id);
                    let target_hp = match hp_query.fetch_mut(target_id) {
                        Some(x) => x,
                        None => {
                            game_log!("Invalid target for lightning bolt");
                            should_run.0 = false;
                            return;
                        }
                    };
                    let target_pos = match pos_query.fetch(target_id) {
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
                        debug!("Lightning bolt hits {} for {} damage!", target_id, dmg);
                        if let Some(Name(name)) = names.fetch(target_id) {
                            game_log!("Lightning Bolt hits {} for {} damage!", name, dmg);
                        }
                    } else {
                        game_log!("Lightning bolt misses!");
                    }
                    cleanup(id, &mut use_item);
                }
                None => {
                    game_log!("Select a target");
                    debug!("Lightning bolt has no target!");
                    should_run.0 = false;
                    *app_mode = AppMode::Targeting;
                    return;
                }
            },
            None => {
                error!("Item has no stuff tag");
                cleanup(id, &mut use_item)
            }
            _ => {
                unreachable!("Bad item use")
            }
        }
    }
}

pub fn update_player_world_interact<'a>(
    mut q_player: Query<(EntityId, &'a mut Inventory, &'a Pos), With<PlayerTag>>,
    mut cmd: Commands,
    q_item: Query<(&StuffTag, Option<&Name>)>,
    grid: Res<Grid<Stuff>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    actions: Res<PlayerActions>,
) {
    if !actions.interact() {
        return;
    }
    for (id, inventory, pos) in q_player.iter_mut() {
        if grid[pos.0] != Some(id) {
            let stuff_id = grid[pos.0].unwrap();
            let (tag, name) = q_item.fetch(stuff_id).unwrap();
            debug!(
                id = tracing::field::display(stuff_id),
                "Interacting with {:?}", tag
            );
            match tag {
                StuffTag::Sword | StuffTag::HpPotion | StuffTag::LightningScroll => {
                    // pick up item
                    match inventory.add(stuff_id) {
                        Ok(_) => {
                            cmd.entity(stuff_id).remove::<Pos>();
                            let Name(ref name) = name.unwrap();
                            game_log!("Picked up a {}", name);
                        }
                        Err(err) => match err {
                            crate::components::InventoryError::Full => {
                                game_log!("Inventory is full");
                                should_run.0 = false;
                            }
                        },
                    }
                }
                _ => unreachable!(),
            }
        } else {
            game_log!("Nothing to do...");
            should_run.0 = false;
        }
    }
}

pub fn update_player_inventory(
    mut q_melee: Query<&mut Melee>,
    q_inventory: Query<(EntityId, &Inventory), With<PlayerTag>>,
) {
    for (player_id, inventory) in q_inventory.iter() {
        let mut power = 1;
        for item in inventory.items.iter().copied() {
            if let Some(melee_weapon) = q_melee.fetch(item) {
                power += melee_weapon.power;
            }
        }
        q_melee.fetch_mut(player_id).unwrap().power = power;
    }
}

pub fn handle_player_move<'a>(
    actions: Res<PlayerActions>,
    mut player_q: Query<(&'a Melee, &'a mut Pos), With<PlayerTag>>,
    stuff_tags: Query<&StuffTag>,
    mut hp: Query<&mut Hp>,
    mut grid: ResMut<Grid<Stuff>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    names: Query<&Name>,
) {
    let delta = match actions.move_action() {
        Some(x) => x,
        None => {
            return;
        }
    };
    for (power, pos) in player_q.iter_mut() {
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
                            let hp = hp.fetch_mut(stuff_id).expect("Enemy has no hp");
                            let power = power.power;
                            hp.current -= power;
                            debug!("kick enemy {}: {:?}", stuff_id, hp);
                            if let Some(Name(name)) = names.fetch(stuff_id) {
                                game_log!("Bonk {} for {} damage", name, power);
                            }
                        } else {
                            debug!("miss enemy {}", stuff_id);
                            game_log!("Your attack misses");
                        }
                    }
                    StuffTag::LightningScroll | StuffTag::HpPotion | StuffTag::Sword => {
                        // pick up item
                        grid_step(pos, new_pos, &mut grid);
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

pub fn update_grid(
    player: Query<(EntityId, &Pos), With<PlayerTag>>,
    mid_stuff: Query<(EntityId, &Pos), (WithOut<PlayerTag>, WithOut<Ai>, WithOut<StaticStuff>)>,
    ais: Query<(EntityId, &Pos), With<Ai>>,
    mut grid: ResMut<Grid<Stuff>>,
    static_stuff: Res<StaticGrid>,
) {
    // zero out the map
    grid.copy(&static_stuff.0);

    // layer back to front, in case multiple entities sit on the same position
    let iterators: [&mut dyn Iterator<Item = (EntityId, &Pos)>; 3] =
        [&mut player.iter(), &mut mid_stuff.iter(), &mut ais.iter()];
    for iter in iterators {
        for (id, pos) in iter {
            let pos = pos.0;
            grid[pos] = Some(id);
        }
    }
}

pub fn update_melee_ai<'a>(
    mut q_player: Query<(EntityId, &'a mut Hp, &'a LastPos), (With<Pos>, With<PlayerTag>)>,
    mut q_enemy: Query<
        (
            EntityId,
            Option<&'a Name>,
            &'a Melee,
            &'a mut PathCache,
            Option<&'a Leash>,
        ),
        (With<Pos>, With<Ai>),
    >,
    mut q_pos: Query<&mut Pos>,
    q_tag: Query<&StuffTag>,
    q_walk: Query<&Walkable>,
    mut grid: ResMut<Grid<Stuff>>,
) {
    let (player_id, player_hp, LastPos(last_player_pos)) = match q_player.iter_mut().next() {
        Some(x) => x,
        None => {
            debug!("No player on the map! Skipping melee update");
            return;
        }
    };
    let Pos(player_pos) = *q_pos.fetch(player_id).unwrap();

    for (id, name, Melee { power, skill }, cache, leash) in q_enemy.iter_mut() {
        let name = name
            .map(|name| name.0.clone())
            .unwrap_or_else(|| id.to_string());
        let Pos(pos) = q_pos.fetch_mut(id).unwrap();
        if pos.manhatten(player_pos) <= 1 {
            if !skill_check(*skill) {
                game_log!("{} misses", name);
                continue;
            }

            player_hp.current -= power;
            game_log!("{} hits you for {} damage", name, power);
            cache.path.clear();
        } else if walk_grid_on_segment(*pos, player_pos, &grid, &q_tag).is_none() {
            debug!("Player is visible, finding path");
            cache.path.clear();
            cache.path.push(player_pos); // push the last pos, so entities can follow players
                                         // across corridors
            cache.path.push(*last_player_pos);
            if !find_path(*pos, *last_player_pos, &grid, &q_walk, &mut cache.path) {
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

pub fn update_player_hp<'a>(
    mut cmd: Commands,
    mut query_player: Query<(EntityId, &'a Hp, &'a mut Icon), With<PlayerTag>>,
) {
    for (player_id, hp, i) in query_player.iter_mut() {
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
    query_hp: Query<(EntityId, &Hp, Option<&Name>), (With<Ai>, WithOut<PlayerTag>)>,
) {
    for (id, _hp, name) in query_hp.iter().filter(|(_, hp, _)| (hp.current <= 0)) {
        debug!("Entity {} died", id);
        if let Some(Name(name)) = name {
            game_log!("{} died", name);
        }

        cmd.delete(id);
    }
}

/// Throw a 1D6, if result is <= skill then the check passes
pub fn skill_check(skill: i32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=6) <= skill
}

pub fn update_tick(mut t: ResMut<GameTick>) {
    t.0 += 1;
}

pub fn should_update_world(should_tick: Res<ShouldTick>, r: Res<ShouldUpdateWorld>) -> bool {
    r.0 && should_tick.0
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
    selected: Res<Selected>,
    history: Res<LogHistory>,
) {
    use std::fmt::Write;

    let _span = tracing::span!(tracing::Level::DEBUG, "update_output").entered();

    let player = q_player
        .iter()
        .next()
        .map(|(pos, hp, attack)| PlayerOutput {
            player_hp: *hp,
            player_attack: attack.power,
            player_pos: pos.0,
        });
    let mut log = String::with_capacity(1024);
    for (tick, payload) in history.0.iter() {
        writeln!(&mut log, "------- {} -------", tick.0).unwrap();
        writeln!(&mut log, "{}", payload).unwrap();
    }
    let current_log = crate::logging::get_log_buffer();
    if !current_log.is_empty() {
        writeln!(&mut log, "------- {} -------", game_tick.0).unwrap();
        writeln!(&mut log, "{}", *current_log).unwrap();
    }
    let result = RenderedOutput {
        player,
        log,
        selected: selected.0.clone(),
    };
    output_cache.0 = JsValue::from_serde(&result).unwrap();
}

pub fn should_update_player(should_tick: Res<ShouldTick>, s: Res<ShouldUpdatePlayer>) -> bool {
    s.0 && should_tick.0
}

pub fn player_prepare(
    mut should_update: ResMut<ShouldUpdateWorld>,
    q: Query<&(), With<PlayerTag>>,
    mut should_update_player: ResMut<ShouldUpdatePlayer>,
    actions: Res<PlayerActions>,
    should_tick: Res<ShouldTick>,
) {
    if !should_tick.0 {
        should_update_player.0 = false;
        should_update.0 = false;
        return;
    }
    // if no player is found then don't update player logic
    should_update_player.0 = false;
    should_update.0 = true;
    for _ in q.iter() {
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

pub fn canvas_cell_size(width: f64, height: f64, viewport: Vec2) -> f64 {
    height.min(width) / (viewport.y * 2) as f64
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
    let cell_size = canvas_cell_size(width, height, viewport.0);
    let icon_scale = cell_size / 512.0;
    for y in min.y.max(0)..(max.y + 1).min(grid.height()) {
        for x in min.x.max(0)..(max.x + 1).min(grid.width()) {
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

pub fn handle_click(
    mut target: ResMut<Selected>,
    grid: Res<Grid<Stuff>>,
    viewport: Res<Viewport>,
    camera_pos: Res<CameraPos>,
    visible: Res<Visible>,
    res: Res<RenderResources>,
    click: Res<ClickPosition>,
) {
    if click.0.is_none() {
        return;
    }
    let width = res.width as f64;
    let height = res.height as f64;
    let cell_size = canvas_cell_size(width, height, viewport.0);

    let [x, y] = click.0.unwrap();

    let pos = Vec2::new(
        remap_f64(0.0, width, 0.0, viewport.0.x as f64 * 2.0, x) as i32,
        remap_f64(0.0, height, 0.0, viewport.0.y as f64 * 2.0, y) as i32,
    ) + camera_pos.0
        - viewport.0;

    debug!(
        pos = tracing::field::debug(pos),
        camera_pos = tracing::field::debug(camera_pos.0),
        viewport = tracing::field::debug(viewport.0),
        cell_size = tracing::field::debug(cell_size),
        "clicked on grid position",
    );
    if !visible.0.at(pos.x, pos.y).unwrap_or(&false) {
        target.0 = None;
        return;
    }

    let result = grid[pos];
    target.0 = result;

    debug!("targeting entity {:?}", result);
}

pub fn record_last_pos<'a>(mut q: Query<(&'a mut LastPos, &'a Pos)>) {
    for (last, current) in q.iter_mut() {
        last.0 = current.0
    }
}

pub fn init_static_grid(
    q: Query<(EntityId, &Pos), With<StaticStuff>>,
    mut grid: ResMut<StaticGrid>,
) {
    grid.0.fill(None);
    for (id, Pos(p)) in q.iter() {
        grid.0[*p] = Some(id);
    }
}

pub fn update_should_tick(
    mut dt: ResMut<DeltaTime>,
    mut time: ResMut<Time>,
    mut should_tick: ResMut<ShouldTick>,
    actions: Res<PlayerActions>,
    use_item: Res<UseItem>,
    tick_time: Res<TickInMs>,
) {
    time.0 += dt.0;
    dt.0 = 0;
    should_tick.0 = (use_item.0.is_some() || !actions.is_empty()) && time.0 >= tick_time.0;
    if should_tick.0 {
        time.0 = 0;
    }
}

pub fn handle_targeting(
    mut should_tick: ResMut<ShouldTick>,
    actions: Res<PlayerActions>,
    mut mode: ResMut<AppMode>,
) {
    match *mode {
        AppMode::Game => {}
        AppMode::Targeting => {
            if actions.target().is_some() {
                *mode = AppMode::Game;
            }
        }
    }
    should_tick.0 = should_tick.0 && matches!(*mode, AppMode::Game);
}

pub fn clean_inputs(
    should_tick: Res<ShouldTick>,
    mut inputs: ResMut<Vec<InputEvent>>,
    mut actions: ResMut<PlayerActions>,
) {
    if should_tick.0 {
        inputs.clear();
        actions.clear();
    }
}

pub fn rotate_log(mut history: ResMut<LogHistory>, tick: Res<GameTick>) {
    let mut buff = crate::logging::get_log_buffer();
    history.0.push_back((*tick, std::mem::take(&mut buff)));
    while history.0.len() > 10 {
        history.0.pop_front();
    }
}
