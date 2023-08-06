use crate::{
    archetypes::icon,
    colors::*,
    components::*,
    grid::Grid,
    map_gen,
    math::{remap_f64, walk_square, Vec2},
    pathfinder::find_path,
    InputEvent, PlayerActions, PlayerOutput, RenderedOutput, Stuff,
};
use cecs::prelude::*;
use rand::{prelude::SliceRandom, Rng};
use tracing::{debug, info};

pub fn init_world_systems(world: &mut World) {
    world.add_stage(
        SystemStage::new("inputs")
            .with_system(update_input_events)
            .with_system(update_should_tick)
            .with_system(handle_targeting)
            .with_system(player_prepare)
            .with_system(handle_levelup),
    );
    world.add_stage(
        SystemStage::new("pre-update")
            .with_should_run(|should_tick: Res<ShouldTick>| should_tick.0)
            .with_system(record_last_pos),
    );
    world.add_stage(
        SystemStage::new("player-update")
            .with_should_run(should_update_player)
            .with_system(update_equipment_use)
            .with_system(update_consumable_use)
            .with_system(handle_player_move)
            .with_system(update_player_world_interact)
            .with_system(update_camera_pos),
    );
    world.add_stage(
        SystemStage::new("update_item_use")
            .with_should_run(should_update_player)
            .with_system(use_poison_scroll)
            .with_system(use_confusion_scroll)
            .with_system(use_lightning_scroll)
            .with_system(use_hp_potion)
            .with_system(use_fireball),
    );
    world.add_stage(
        SystemStage::new("update-ai-hp")
            .with_should_run(should_update_world)
            .with_system(update_poison)
            .with_system(update_ai_hp),
    );
    world.add_stage(
        SystemStage::new("ai-update")
            .with_should_run(should_update_world)
            .with_system(update_ai_move)
            .with_system(update_melee_ai)
            .with_system(update_confusion)
            .with_system(update_player_hp)
            .with_system(update_grid)
            .with_system(update_fov),
    );
    world.add_stage(SystemStage::new("update-pos").with_system(perform_move));
    world.add_stage(
        SystemStage::new("render")
            .with_system(update_output)
            .with_system(render_into_canvas)
            .with_system(clean_inputs),
    );
    world.add_stage(
        SystemStage::new("post-render")
            .with_should_run(should_update_world)
            .with_system(clear_consumable)
            .with_system(update_tick),
    );
    world.add_stage(
        SystemStage::new("dungeon-delve")
            .with_should_run(|level: Res<DungeonFloor>| level.current != level.desired)
            .with_system(regenerate_dungeon),
    );
}

fn update_input_events(inputs: Res<Vec<InputEvent>>, mut actions: ResMut<PlayerActions>) {
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

fn equip_item(id: EntityId, equipment: &mut Option<EntityId>, inventory: &mut Inventory) {
    // move old weapon back to the inventory
    match equipment.take() {
        Some(old_id) => {
            let (i, _) = inventory
                .items
                .iter()
                .enumerate()
                .find(|(_, i)| *i == &id)
                .unwrap();

            inventory.items[i] = old_id;
        }
        None => {
            // if old id doesn't exist then remove this item from the inventory
            inventory.remove(id);
        }
    }
    let _ = equipment.insert(id);
}

fn clear_consumable(
    mut player_query: Query<&mut Inventory, With<PlayerTag>>,
    mut cmd: Commands,
    q: Query<EntityId, With<ClearInventoryItem>>,
) {
    let Some(inventory) = player_query.iter_mut().next() else {
        return;
    };
    for id in q.iter() {
        debug!("Deleting item in inventory: {}", id);
        inventory.remove(id);
        cmd.delete(id);
    }
}

fn use_poison_scroll(
    mut cmd: Commands,
    mut target_query: Query<(Option<&mut Poisoned>, Option<&Name>)>,
    item_query: Query<(EntityId, &Ranged, &Targeting), (With<MarkConsume>, With<PoisionAttack>)>,
    mut log: ResMut<LogHistory>,
    mut should_run: ResMut<ShouldUpdateWorld>,
) {
    for (item_id, range, Targeting(target_id)) in item_query.iter() {
        let target_id = *target_id;
        debug!("Use PoisonScroll");
        let Some((target_poison, target_name)) = target_query.fetch_mut(target_id) else {
            log.push(IMPOSSIBLE, "Invalid target");
            should_run.0 = false;
            return;
        };

        if skill_check(range.skill) {
            // TODO: config duration
            let duration = 5;
            debug!("Poision Bolt hits {} for {} turns!", target_id, duration);
            if let Some(poision) = target_poison {
                poision.duration += duration;
            } else {
                cmd.entity(target_id).insert(Poisoned {
                    duration,
                    power: range.power,
                });
            }
            if let Some(Name(name)) = target_name {
                log.push(WHITE, &format!("{} suffers from poison!", name));
            }
        } else {
            log.push(WHITE, "Poison Bolt misses!");
        }
        cmd.entity(item_id).insert(ClearInventoryItem);
    }
}

fn use_hp_potion(
    mut cmd: Commands,
    mut player_query: Query<&mut Hp, With<PlayerTag>>,
    item_query: Query<(EntityId, &Heal), With<MarkConsume>>,
    mut log: ResMut<LogHistory>,
) {
    let Some(hp) = player_query.iter_mut().next() else {
        return;
    };

    for (id, heal) in item_query.iter() {
        debug!("Use hp potion");
        log.push(HEAL, "Drink a health potion.");
        if hp.full() {
            log.push(INVALID, "The potion has no effect");
        }
        hp.current = (hp.current + heal.hp).min(hp.max);
        cmd.entity(id).insert(ClearInventoryItem);
    }
}

fn use_confusion_scroll(
    mut cmd: Commands,
    item_query: Query<(EntityId, &Ranged, &Targeting), (With<MarkConsume>, With<ConfusionBolt>)>,
    mut target_query: Query<(Option<&mut ConfusedAi>, Option<&Name>)>,
    mut log: ResMut<LogHistory>,
    mut should_run: ResMut<ShouldUpdateWorld>,
) {
    for (item_id, range, target) in item_query.iter() {
        let target_id = target.0;
        debug!("Use ConfusionScroll");
        let Some((target_confusion, target_name)) = target_query.fetch_mut(target_id) else {
            log.push(IMPOSSIBLE, "Invalid target");
            should_run.0 = false;
            return;
        };
        if skill_check(range.skill) {
            let duration = range.power;
            debug!("Confusion Bolt hits {} for {} turns!", target_id, duration);
            if let Some(confusion) = target_confusion {
                confusion.duration += duration;
            } else {
                cmd.entity(target_id).insert(ConfusedAi { duration });
            }
            if let Some(Name(name)) = target_name {
                log.push(
                    WHITE,
                    &format!(
                        "The eyes of the {} look vacant, as it starts to stumble around!",
                        name
                    ),
                );
            }
        } else {
            log.push(WHITE, "Confusion Bolt misses!");
        }
        cmd.entity(item_id).insert(ClearInventoryItem);
    }
}

fn use_lightning_scroll(
    mut cmd: Commands,
    item_query: Query<(EntityId, &Ranged, &Targeting), (With<MarkConsume>, With<LightningBolt>)>,
    mut target_query: Query<(&mut Hp, Option<&Name>)>,
    mut log: ResMut<LogHistory>,
    mut should_run: ResMut<ShouldUpdateWorld>,
) {
    for (item_id, range, target) in item_query.iter() {
        let target_id = target.0;
        debug!("Use lightning scroll {}", item_id);
        let (target_hp, target_name) = match target_query.fetch_mut(target_id) {
            Some(x) => x,
            None => {
                log.push(INVALID, "Invalid target");
                should_run.0 = false;
                return;
            }
        };
        if skill_check(range.skill) {
            let dmg = range.power;
            target_hp.current -= dmg;
            debug!("Lightning Bolt hits {} for {} damage!", target_id, dmg);
            if let Some(Name(name)) = target_name {
                log.push(
                    WHITE,
                    &format!("Lightning Bolt hits {} for {} damage!", name, dmg),
                );
            }
        } else {
            log.push(INVALID, "Lightning Bolt misses!");
        }
        cmd.entity(item_id).insert(ClearInventoryItem);
    }
}

fn use_fireball(
    mut cmd: Commands,
    item_query: Query<
        (EntityId, &Ranged, &Aoe, &TargetingPos),
        (With<MarkConsume>, With<FireBall>),
    >,
    mut target_query: Query<(&mut Hp, Option<&Name>)>,
    mut log: ResMut<LogHistory>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    mut app_mode: ResMut<AppMode>,
    grid: Res<Grid<Stuff>>,
) {
    for (item_id, range, aoe, target_pos) in item_query.iter() {
        if target_pos.src.chebyshev(target_pos.dst) > range.range {
            log.push(INVALID, "Target is too far away. Try again");
            *app_mode = AppMode::TargetingPosition;
            should_run.0 = false;
            return;
        }
        // TODO: put the log line as component
        log.push(
            PLAYER_ATTACK,
            format!("Hurl a fire ball at {}", target_pos.dst),
        );
        let radius = Vec2::splat(aoe.radius as i32);
        let power = range.power;
        grid.scan_range(
            [target_pos.dst - radius, target_pos.dst + radius],
            |_pos, id| {
                if let Some(id) = id {
                    if let Some((hp, name)) = target_query.fetch_mut(*id) {
                        // TODO skill check?
                        hp.current -= power;
                        if let Some(Name(ref name)) = name {
                            // TODO: put the log line as component
                            // or maybe polymorphic system?
                            log.push(
                                PLAYER_ATTACK,
                                format!(
                                    "{} is engulfed in a fiery explosion, taking {} damage",
                                    name, power
                                ),
                            );
                        }
                    }
                }
            },
        );
        cmd.entity(item_id).insert(ClearInventoryItem);
    }
}

fn update_consumable_use(
    actions: Res<PlayerActions>,
    mut cmd: Commands,
    mut player_query: Query<&Pos, With<PlayerTag>>,
    q: QuerySet<(
        Query<(EntityId, Option<&Ranged>), (With<UseItem>, With<NeedsTargetEntity>)>,
        Query<(EntityId, &Ranged), (With<UseItem>, With<NeedsTargetPosition>)>,
        Query<
            EntityId,
            (
                With<UseItem>,
                WithOut<EquipmentType>,
                WithOut<NeedsTargetPosition>,
                WithOut<NeedsTargetEntity>,
            ),
        >,
    )>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    mut app_mode: ResMut<AppMode>,
    target_query: Query<&Pos>,
    target_pos: Res<TargetPos>,
    mut log: ResMut<LogHistory>,
) {
    let Some(player_pos) = player_query.iter_mut().next() else {
        return;
    };

    for (id, range) in q.q0().iter() {
        match actions.target() {
            None => {
                log.push(WHITE, "Select a target");
                debug!("Targeted item has no target!");
                should_run.0 = false;
                *app_mode = AppMode::Targeting;
            }
            Some(target_id) => {
                debug!("Use PoisonScroll");
                let Some(target_pos) = target_query.fetch(target_id) else {
                    log.push(IMPOSSIBLE, "Invalid target");
                    should_run.0 = false;
                    return;
                };
                if let Some(range) = range {
                    if target_pos.0.chebyshev(player_pos.0) > range.range {
                        log.push(IMPOSSIBLE, "Target is too far away");
                        should_run.0 = false;
                        return;
                    }
                }

                cmd.entity(id)
                    .insert_bundle((MarkConsume, Targeting(target_id)));
            }
        }
    }
    for (id, range) in q.q1().iter() {
        match target_pos.pos {
            Some(target_pos) => {
                if target_pos.chebyshev(player_pos.0) > range.range {
                    log.push(INVALID, "Target is too far away. Try again");
                    *app_mode = AppMode::TargetingPosition;
                    should_run.0 = false;
                    return;
                }
                cmd.entity(id).insert_bundle((
                    MarkConsume,
                    TargetingPos {
                        src: player_pos.0,
                        dst: target_pos,
                    },
                ));
            }
            None => {
                log.push(NEEDS_TARGET, "Select a target position");
                debug!("Position target item has no target!");
                should_run.0 = false;
                *app_mode = AppMode::TargetingPosition;
            }
        }
    }
    for id in q.q2().iter() {
        cmd.entity(id).insert(MarkConsume);
    }
}

fn update_equipment_use(
    mut cmd: Commands,
    mut player_query: Query<(EntityId, &mut Inventory, &mut Equipment), With<PlayerTag>>,
    q: Query<(EntityId, &EquipmentType), With<UseItem>>,
    mut item_query: QuerySet<(Query<&mut Melee>, Query<&mut Defense>)>,
) {
    let Some((player_id, inventory, equipment)) = player_query.iter_mut().next() else {
        return;
    };
    for (id, ty) in q.iter() {
        debug!("Equipping {}, tag: {:?}", id, ty);

        cmd.entity(id).remove::<UseItem>();

        match ty {
            EquipmentType::Weapon => {
                let old_id = &mut equipment.weapon;

                // update power
                let new_power = *item_query.q0().fetch(id).unwrap();
                let old_power = old_id.and_then(|id| item_query.q0().fetch(id).copied());
                let player_power = item_query.q0_mut().fetch_mut(player_id).unwrap();
                *player_power += new_power;
                if let Some(old_power) = old_power {
                    *player_power -= old_power;
                }

                equip_item(id, old_id, inventory);
            }
            EquipmentType::Armor => {
                let old_id = &mut equipment.armor;

                // update defense
                let new_defense = *item_query.q1().fetch(id).unwrap();
                let old_defense = old_id.and_then(|id| item_query.q1().fetch(id).copied());
                let player_defense = item_query.q1_mut().fetch_mut(player_id).unwrap();
                *player_defense += new_defense;
                if let Some(old_defense) = old_defense {
                    *player_defense -= old_defense;
                }

                equip_item(id, old_id, inventory);
            }
        }
    }
}

fn update_player_world_interact(
    mut q_player: Query<(EntityId, &mut Inventory, &Pos), With<PlayerTag>>,
    mut cmd: Commands,
    q_item: Query<(Option<&Item>, Option<&NextLevel>, Option<&Name>)>,
    grid: Res<Grid<Stuff>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    actions: Res<PlayerActions>,
    mut level: ResMut<DungeonFloor>,
    mut log: ResMut<LogHistory>,
) {
    if !actions.interact() {
        return;
    }
    for (id, inventory, pos) in q_player.iter_mut() {
        if grid[pos.0] != Some(id) {
            let stuff_id = grid[pos.0].unwrap();
            let (item_tag, next_level_tag, name) = q_item.fetch(stuff_id).unwrap();
            debug!(
                id = tracing::field::display(stuff_id),
                "Interacting with entity"
            );
            if item_tag.is_some() {
                match inventory.add(stuff_id) {
                    Ok(_) => {
                        cmd.entity(stuff_id).remove::<Pos>();
                        let Name(ref name) = name.unwrap();
                        log.push(WHITE, format!("Picked up a {}", name));
                    }
                    Err(err) => match err {
                        crate::components::InventoryError::Full => {
                            log.push(INVALID, "Inventory is full");
                            should_run.0 = false;
                        }
                    },
                }
            } else if next_level_tag.is_some() {
                log.push(WHITE, "You descend the staircase");
                level.desired += 1;
            } else {
                debug!("Cant interact with {}", id);
            }
        } else {
            log.push(IMPOSSIBLE, "Nothing to do...");
            should_run.0 = false;
        }
    }
}

fn compute_damage(power: i32, defense: i32) -> i32 {
    // all damage must be at least 1
    (power - defense).max(1)
}

fn handle_player_move(
    actions: Res<PlayerActions>,
    mut player_q: Query<(&Melee, &mut Pos), With<PlayerTag>>,
    stuff_tags: Query<&StuffTag>,
    mut enemy_q: Query<(&mut Hp, &Defense)>,
    mut grid: ResMut<Grid<Stuff>>,
    mut should_run: ResMut<ShouldUpdateWorld>,
    names: Query<&Name>,
    mut log: ResMut<LogHistory>,
    mut cmd: Commands,
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
        match grid
            .at(new_pos.x, new_pos.y)
            .unwrap()
            .and_then(|id| stuff_tags.fetch(id).map(|tag| (id, tag)))
        {
            Some((stuff_id, tag)) => match tag {
                StuffTag::Player => unreachable!(),
                StuffTag::Door => {
                    // TODO: ability to close the door?
                    // would need some persistent state then, instead of deleting
                    cmd.delete(stuff_id);
                }
                StuffTag::Wall => {
                    log.push(INVALID, "Can't move into wall");
                    should_run.0 = false;
                }
                StuffTag::Gargoyle
                | StuffTag::Goblin
                | StuffTag::Troll
                | StuffTag::Orc
                | StuffTag::Warlord
                | StuffTag::Zombie
                | StuffTag::Minotaur => {
                    if skill_check(power.skill) {
                        let (hp, defense) = enemy_q.fetch_mut(stuff_id).expect("Enemy has no hp");
                        let damage = compute_damage(power.power, defense.melee_defense);
                        hp.current -= damage;
                        debug!("kick enemy {}: {:?}", stuff_id, hp);
                        if let Some(Name(name)) = names.fetch(stuff_id) {
                            log.push(
                                PLAYER_ATTACK,
                                format!("Bonk {} for {} damage", name, damage),
                            );
                        }
                    } else {
                        debug!("miss enemy {}", stuff_id);
                        log.push(PLAYER_ATTACK, "Your attack misses");
                    }
                }
                StuffTag::LightningScroll
                | StuffTag::PoisonScroll
                | StuffTag::HpPotion
                | StuffTag::LeatherArmor
                | StuffTag::ChainMailArmor
                | StuffTag::Sword
                | StuffTag::RareSword
                | StuffTag::Dagger
                | StuffTag::RareDagger
                | StuffTag::ConfusionScroll
                | StuffTag::FireBallScroll
                | StuffTag::Tombstone
                | StuffTag::Stairs => {
                    grid_step(pos, new_pos, &mut grid);
                }
            },
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
}

/// return wether the segment hits something and where
fn walk_grid_on_segment(
    from: Vec2,
    to: Vec2,
    grid: &Grid<Stuff>,
    opaque: &Query<&(), With<Opaque>>,
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
            .map(|id| opaque.contains(*id))
            .unwrap_or(false)
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
    opaque: &Query<&(), With<Opaque>>,
    player_pos: Vec2,
    radius: i32,
) {
    visible.splat_set([Vec2::ZERO, visible.dims()], false);
    // walk the visible range
    walk_square(-Vec2::splat(radius), Vec2::splat(radius))
        .map(|d| player_pos + d)
        .for_each(|limit| {
            if walk_grid_on_segment(player_pos, limit, grid, opaque).is_none() {
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
    grid: Res<Grid<Stuff>>,
    mut explored: ResMut<Explored>,
    mut visible: ResMut<Visible>,
    viewport: Res<Visibility>,
    opaque: Query<&(), With<Opaque>>,
) {
    let radius = viewport.0.x.max(viewport.0.y);
    if let Some(player_pos) = q.iter().next() {
        set_visible(&grid, &mut visible.0, &opaque, player_pos.0, radius);
        visible.0[player_pos.0] = true;
        flood_vizibility(&grid, &mut visible.0, player_pos.0, radius);
        explored.0.or_eq(&visible.0);
    }
}

fn update_grid(
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

fn perform_move(mut q: Query<(&mut Pos, &mut Velocity)>, mut grid: ResMut<Grid<Stuff>>) {
    for (pos, vel) in q.iter_mut() {
        if vel.0 == Vec2::ZERO {
            continue;
        }
        let new_pos = pos.0 + vel.0;
        if grid[new_pos].is_none() {
            let res = grid[pos.0].take();
            grid[new_pos] = res;
            pos.0 = new_pos;
        } else {
            debug!("Pos {} is occupied", new_pos);
        }
        vel.0 = Vec2::ZERO;
    }
}

fn update_confusion(
    mut cmd: Commands,
    mut confused: Query<(EntityId, Option<&Name>, &mut ConfusedAi)>,
    mut log: ResMut<LogHistory>,
) {
    for (id, name, confusion) in confused.iter_mut() {
        confusion.duration -= 1;
        if confusion.duration <= 0 {
            if let Some(Name(name)) = name {
                log.push(STATUS_EFFECT, format!("{} is no longer confused!", name));
            }
            cmd.entity(id).remove::<ConfusedAi>();
        }
    }
}

fn update_ai_move(
    q_player: Query<(&Pos, &LastPos), (With<Pos>, With<PlayerTag>)>,
    grid: Res<Grid<Stuff>>,
    mut melee: Query<
        (EntityId, &mut PathCache, &Pos, Option<&Leash>),
        (With<Melee>, With<Velocity>, WithOut<ConfusedAi>),
    >,
    mut confused: Query<EntityId, (With<ConfusedAi>, With<Velocity>)>,
    mut q_vel: Query<&mut Velocity>,
    q_walk: Query<&Walkable>,
    opaque: Query<&(), With<Opaque>>,
) {
    let (Pos(player_pos), LastPos(last_player_pos)) = match q_player.iter().next() {
        Some(x) => x,
        None => {
            debug!("No player on the map! Skipping melee update");
            return;
        }
    };
    for (id, cache, Pos(pos), leash) in melee.iter_mut() {
        let vel = q_vel.fetch_mut(id).unwrap();
        if pos.manhatten(*player_pos) > 1 {
            if walk_grid_on_segment(*pos, *player_pos, &grid, &opaque).is_none() {
                debug!("Player is visible, finding path");
                cache.path.clear();
                cache.path.push(*player_pos); // push the last pos, so entities can follow players
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
                    vel.0 = new_pos - *pos;
                }
            }
        } else {
            cache.path.clear();
        }
    }

    let delta = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y];
    let mut rng = rand::thread_rng();
    for id in confused.iter_mut() {
        let vel = q_vel.fetch_mut(id).unwrap();
        vel.0 = *delta.choose(&mut rng).unwrap();
    }
}

fn update_melee_ai(
    mut q_player: Query<(EntityId, &Pos, &Defense), (With<Hp>, With<PlayerTag>)>,
    mut q_target: Query<(&mut Hp, Option<&Name>)>,
    mut q_enemy: Query<
        (
            EntityId,
            Option<&Name>,
            &Melee,
            &Pos,
            Option<&ConfusedAi>,
            Option<&Velocity>,
        ),
        With<Ai>,
    >,
    grid: Res<Grid<Stuff>>,
    mut log: ResMut<LogHistory>,
) {
    let (player_id, Pos(player_pos), player_defense) = match q_player.iter_mut().next() {
        Some(x) => x,
        None => {
            debug!("No player on the map! Skipping melee update");
            return;
        }
    };

    for (id, name, Melee { power, skill }, Pos(pos), confused, vel) in q_enemy.iter_mut() {
        let name = name
            .map(|name| name.0.clone())
            .unwrap_or_else(|| id.to_string());
        let damage = compute_damage(*power, player_defense.melee_defense);
        let mut target = None;
        if confused.is_some() {
            if let Some(vel) = vel {
                let target_pos = *pos + vel.0;
                if let Some(t_id) = grid[target_pos] {
                    if let Some(t) = q_target.fetch_mut(t_id) {
                        target = Some((t.0, t.1.map(|n| n.0.as_str()), t_id));
                    }
                }
            }
        } else if pos.manhatten(*player_pos) <= 1 {
            if let Some(t) = q_target.fetch_mut(player_id) {
                target = Some((t.0, Some("you"), player_id));
            }
        }

        if let Some((target_hp, target_name, target_id)) = target {
            if !skill_check(*skill) {
                log.push(ENEMY_ATTACK, format!("{} misses", name));
                continue;
            }
            target_hp.current -= damage;
            let target_name = target_name.unwrap_or("");
            debug!(
                id = tracing::field::display(id),
                target_id = tracing::field::display(target_id),
                "melee hit"
            );
            log.push(
                ENEMY_ATTACK,
                format!("{} hits {} for {} damage", name, target_name, damage),
            );
        }
    }
}

fn update_player_hp(
    mut cmd: Commands,
    query_player: Query<(EntityId, &Hp), With<PlayerTag>>,
    mut log: ResMut<LogHistory>,
) {
    for (player_id, hp) in query_player.iter() {
        if hp.current <= 0 {
            info!("Player died");
            log.push(PLAYER_DIE, "Player died");
            cmd.entity(player_id)
                .remove::<Inventory>()
                .remove::<Hp>()
                .remove::<PlayerTag>()
                .insert_bundle((
                    icon("tombstone"),
                    StuffTag::Tombstone,
                    Name("RIP".to_string()),
                    Description("Your resting place".to_string()),
                ));
        }
    }
}

fn update_ai_hp(
    mut cmd: Commands,
    query_hp: Query<(EntityId, &Hp, Option<&Name>, Option<&Exp>), (With<Ai>, WithOut<PlayerTag>)>,
    mut query_player: Query<&mut Level, With<PlayerTag>>,
    mut log: ResMut<LogHistory>,
) {
    let mut player = query_player.iter_mut().next();
    for (id, _hp, name, xp) in query_hp.iter().filter(|(_, hp, _, _)| (hp.current <= 0)) {
        debug!("Entity {} died", id);
        if let Some(Name(name)) = name {
            log.push(ENEMY_DIE, format!("{} died", name));
        }

        if let Some(level) = player.as_mut() && let Some(xp) = xp {
            level.add_xp(xp.amount);
            debug!("Gain {} xp. Now: {:?}", xp.amount, level);
        }

        cmd.delete(id);
    }
}

/// Throw a 1D6, if result is <= skill then the check passes
fn skill_check(skill: i32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=6) <= skill
}

fn update_tick(mut t: ResMut<GameTick>) {
    t.0 += 1;
}

fn should_update_world(should_tick: Res<ShouldTick>, r: Res<ShouldUpdateWorld>) -> bool {
    r.0 && should_tick.0
}

pub fn update_camera_pos(mut camera: ResMut<CameraPos>, q: Query<&Pos, With<PlayerTag>>) {
    for pos in q.iter() {
        camera.0 = pos.0;
    }
}

pub fn update_output(
    q_player: Query<(&Pos, &Hp, &Melee, &Level, &Defense), With<PlayerTag>>,
    mut output_cache: ResMut<Output>,
    selected: Res<Selected>,
    history: Res<LogHistory>,
    app_mode: Res<AppMode>,
    dungeon_level: Res<DungeonFloor>,
) {
    let _span = tracing::span!(tracing::Level::DEBUG, "update_output").entered();

    let player = q_player
        .iter()
        .next()
        .map(|(pos, hp, attack, level, defense)| PlayerOutput {
            level: level.current_level,
            current_xp: level.current_xp,
            needed_xp: level.experience_to_next_level(),
            player_hp: *hp,
            player_attack: attack.power,
            player_pos: pos.0,
            defense: *defense,
        });
    let mut log = Vec::with_capacity(128);
    for line in history.items.iter() {
        log.push(line.clone());
    }
    let targeting = matches!(*app_mode, AppMode::Targeting);
    let result = RenderedOutput {
        dungeon_level: dungeon_level.current,
        app_mode: *app_mode,
        player,
        log,
        selected: selected.0,
        targeting,
    };
    output_cache.0 = serde_wasm_bindgen::to_value(&result).unwrap();
}

fn should_update_player(should_tick: Res<ShouldTick>, s: Res<ShouldUpdatePlayer>) -> bool {
    s.0 && should_tick.0
}

fn player_prepare(
    mut should_update: ResMut<ShouldUpdateWorld>,
    q: Query<&(), With<PlayerTag>>,
    mut should_update_player: ResMut<ShouldUpdatePlayer>,
    actions: Res<PlayerActions>,
    should_tick: Res<ShouldTick>,
    mut log: ResMut<LogHistory>,
) {
    if !should_tick.0 {
        should_update_player.0 = false;
        should_update.0 = false;
        return;
    }
    // if no player is found then don't update player logic
    should_update_player.0 = false;
    should_update.0 = true;
    if q.iter().next().is_some() {
        should_update_player.0 = true;
    }
    if should_update_player.0 && actions.wait() {
        log.push(WHITE, "Waiting...");
        should_update_player.0 = false;
    }
}

fn canvas_cell_size(width: f64, height: f64, viewport: Vec2) -> f64 {
    height.min(width) / (viewport.y * 2) as f64
}

pub fn render_into_canvas(
    mut res: ResMut<RenderResources>,
    grid: Res<Grid<Stuff>>,
    viewport: Res<Viewport>,
    camera_pos: Res<CameraPos>,
    visible: Res<Visible>,
    explored: Res<Explored>,
    stuff: Query<(Option<&StaticVisibility>, &Icon, Option<&Color>)>,
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

    let black = "black".into();
    let darkgrey = "darkgrey".into();
    let white = "white".into();
    let yellow = "yellow".into();

    for y in min.y.max(0)..(max.y + 1).min(grid.height()) {
        for x in min.x.max(0)..(max.x + 1).min(grid.width()) {
            let pos = Vec2::new(x, y);
            let explored = explored.0[pos];

            if !explored {
                continue;
            }

            let visible = visible.0[pos];

            let render_pos = pos - min;
            let render_x = render_pos.x as f64 * cell_size;
            let render_y = render_pos.y as f64 * cell_size;

            if visible {
                ctx.set_fill_style(&yellow);
            } else {
                ctx.set_fill_style(&darkgrey);
            }

            if explored {
                match grid[pos].and_then(|id| stuff.fetch(id).map(|x| (id, x))) {
                    Some((_id, (static_vis_tag, icon, color))) => {
                        // icon background
                        if visible {
                            ctx.set_fill_style(&black);
                        } else {
                            ctx.set_fill_style(&darkgrey);
                        }
                        ctx.fill_rect(render_x, render_y, cell_size, cell_size);
                        // render icon
                        if visible || static_vis_tag.is_some() {
                            ctx.fill_rect(render_x, render_y, cell_size, cell_size);
                            match icons.0.get(icon.0) {
                                Some(icon) => {
                                    match color {
                                        Some(Color(ref color)) => {
                                            ctx.set_fill_style(color);
                                        }
                                        None => {
                                            ctx.set_fill_style(&white);
                                        }
                                    }
                                    ctx.save();
                                    ctx.translate(render_x, render_y).unwrap();
                                    ctx.scale(icon_scale, icon_scale).unwrap();
                                    ctx.fill_with_path_2d(icon);
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
                            ctx.set_fill_style(&black);
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
    mut target_pos: ResMut<TargetPos>,
    mode: Res<AppMode>,
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
    if matches!(*mode, AppMode::TargetingPosition) {
        target_pos.pos = Some(pos);
        debug!("targeting position {}", pos);
    }
    if !visible.0.at(pos.x, pos.y).unwrap_or(&false) {
        target.0 = None;
        return;
    }

    let result = grid[pos];
    target.0 = result;
    debug!("targeting entity {:?}", result);
}

fn record_last_pos(mut q: Query<(&mut LastPos, &Pos)>) {
    for (last, current) in q.iter_mut() {
        last.0 = current.0
    }
}

pub fn init_grids(
    static_q: Query<(EntityId, &Pos), With<StaticStuff>>,
    dyn_q: Query<(EntityId, &Pos), WithOut<StaticStuff>>,
    mut static_grid: ResMut<StaticGrid>,
    mut grid: ResMut<Grid<Stuff>>,
) {
    if static_grid.0.dims() != grid.dims() {
        static_grid.0 = Grid::new(grid.dims());
    } else {
        static_grid.0.fill(None);
    }
    for (id, Pos(p)) in static_q.iter() {
        static_grid.0[*p] = Some(id);
    }
    grid.copy(&static_grid.0);
    for (id, Pos(p)) in dyn_q.iter() {
        grid[*p] = Some(id);
    }
}

fn update_should_tick(
    mut dt: ResMut<DeltaTime>,
    mut time: ResMut<BounceOffTime>,
    mut should_tick: ResMut<ShouldTick>,
    actions: Res<PlayerActions>,
    tick_time: Res<TickInMs>,
    q_player: Query<&(), With<PlayerTag>>,
    q_item_use: Query<&(), With<UseItem>>,
) {
    time.0 += dt.0;
    should_tick.0 = !q_player.is_empty()
        && (!q_item_use.is_empty() || !actions.is_empty())
        && (time.0 >= tick_time.0 || tick_time.0.abs_diff(time.0) <= 5); // lag compensation
    if should_tick.0 {
        debug!("Running update after {} ms", time.0);
        time.0 = 0;
    }
    dt.0 = 0;
}

fn handle_targeting(
    mut should_tick: ResMut<ShouldTick>,
    actions: Res<PlayerActions>,
    mut mode: ResMut<AppMode>,
    target_pos: Res<TargetPos>,
) {
    match *mode {
        AppMode::Levelup | AppMode::Game => {}
        AppMode::Targeting => {
            if actions.target().is_some() {
                *mode = AppMode::Game;
            }
        }
        AppMode::TargetingPosition => {
            if target_pos.pos.is_some() {
                *mode = AppMode::Game;
            }
        }
    }
    should_tick.0 = should_tick.0 && matches!(*mode, AppMode::Game);
}

fn clean_inputs(
    should_tick: Res<ShouldTick>,
    mut inputs: ResMut<Vec<InputEvent>>,
    mut actions: ResMut<PlayerActions>,
    mut target_pos: ResMut<TargetPos>,
) {
    if should_tick.0 {
        inputs.clear();
        actions.clear();
        let _ = target_pos.pos.take();
    }
}

pub fn regenerate_dungeon(mut access: WorldAccess) {
    info!("Regenerating dungeon");
    let world = access.world_mut();

    let level = world
        .get_resource::<DungeonFloor>()
        .cloned()
        .unwrap_or_default()
        .desired;

    let dims = match level {
        0 | 1 => Vec2::new(64, 64),
        2 => Vec2::new(80, 80),
        3 => Vec2::new(96, 96),
        4 => Vec2::new(115, 115),
        _ => Vec2::new(128, 128),
    };

    // reset some resources
    world.insert_resource(Visible(Grid::new(dims)));
    world.insert_resource(Explored(Grid::new(dims)));
    world.insert_resource(WorldDims(dims));
    world.insert_resource(DungeonFloor {
        current: level,
        desired: level,
    });
    world.insert_resource(level);
    world.insert_resource(map_gen::MapGenProps::from_level(level));
    world.insert_resource(PlayerActions::new());

    world.run_system(map_gen::generate_map);
    world.run_system(init_grids);

    let log = world.get_resource_mut::<LogHistory>().unwrap();
    log.push(WHITE, format!("You're on level {}", level));
    world
        .run_stage(
            SystemStage::new("initial-post-process")
                .with_system(update_camera_pos)
                .with_system(update_grid)
                .with_system(update_fov)
                .with_system(update_output),
        )
        .unwrap();
}

fn handle_levelup(
    mut app_mode: ResMut<AppMode>,
    mut stat: ResMut<Option<DesiredStat>>,
    mut player_q: Query<(&mut Hp, &mut Melee, &mut Level, &mut Defense), With<PlayerTag>>,
    mut log: ResMut<LogHistory>,
) {
    if let Some((hp, melee, level, defense)) = player_q.iter_mut().next() {
        if !level.needs_levelup() {
            return;
        }
        match stat.take() {
            Some(stat) => {
                debug_assert!(matches!(*app_mode, AppMode::Levelup));
                level.levelup();
                // player _might_ level up multiple times in a single tick
                if level.needs_levelup() {
                    let level = level.current_level + 1;
                    log.push(WHITE, format!("Level up! Your're now level {}", level));
                    log.push(WHITE, "Select a stat to upgrade!");
                } else {
                    *app_mode = AppMode::Game;
                }
                match stat {
                    DesiredStat::Attack => {
                        melee.power += 1;
                    }
                    DesiredStat::Hp => {
                        let amount = 10;
                        hp.current += amount;
                        hp.max += amount;
                    }
                    DesiredStat::MeleeDefense => {
                        defense.melee_defense += 1;
                    }
                }
            }
            None => {
                if !matches!(*app_mode, AppMode::Levelup) {
                    let level = level.current_level + 1;
                    log.push(WHITE, format!("Level up! Your're now level {}", level));
                    log.push(WHITE, "Select a stat to upgrade!");
                    *app_mode = AppMode::Levelup;
                }
            }
        }
    }
}

fn update_poison(
    mut cmd: Commands,
    mut q: Query<(
        EntityId,
        &mut Hp,
        &mut Poisoned,
        Option<&mut Name>,
        Option<&PlayerTag>,
    )>,
    mut log: ResMut<LogHistory>,
) {
    for (id, hp, poison, name, player) in q.iter_mut() {
        if poison.duration <= 0 {
            cmd.entity(id).remove::<Poisoned>();
            continue;
        }
        if let Some(name) = name {
            let color = if player.is_some() {
                ENEMY_ATTACK
            } else {
                PLAYER_ATTACK
            };
            log.push(
                color,
                format!("{} is hit for {} damage by poison", name.0, poison.power),
            );
        }
        poison.duration -= 1;
        // TODO: poison resistance
        hp.current -= compute_damage(poison.power, 0);
    }
}
