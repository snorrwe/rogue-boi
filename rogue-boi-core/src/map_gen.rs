mod rect_room;
mod tunnel_iter;

use std::collections::HashMap;

use self::rect_room::RectRoom;
use self::tunnel_iter::TunnelIter;
use crate::game_config::{ENEMY_CHANCES, ITEM_CHANCES};
use cecs::prelude::*;
use rand::{
    prelude::{Distribution, SliceRandom},
    Rng,
};
use tracing::debug;

use crate::{
    archetypes::init_entity,
    components::{DungeonFloor, PlayerTag, Pos, StuffTag, WorldDims},
    grid::Grid,
    math::Vec2,
    Stuff,
};

#[derive(Clone)]
pub struct MapGenProps {
    pub room_min_size: u32,
    pub room_max_size: u32,
    pub max_rooms: u32,
    pub max_items_per_floor: u32,
    pub max_monsters_per_floor: u32,
}

impl MapGenProps {
    pub fn from_level(level: u32) -> Self {
        // sorted from high to low
        const MAX_ITEMS: &[[u32; 2]] = &[[4, 2], [1, 1]];
        const MAX_MONSTERS: &[[u32; 2]] = &[[6, 5], [4, 3], [1, 2]];

        let max_items_per_floor = MAX_ITEMS
            .iter()
            .find(|[x, _]| x <= &level)
            .map(|[_, c]| *c)
            .unwrap_or(0);

        let max_monsters_per_floor = MAX_MONSTERS
            .iter()
            .find(|[x, _]| x <= &level)
            .map(|[_, c]| *c)
            .unwrap_or(0);

        MapGenProps {
            max_monsters_per_floor,
            max_items_per_floor,
            room_min_size: 6,
            room_max_size: 10,
            max_rooms: 50,
        }
    }
}

#[derive(Default, Debug)]
struct EntityChances {
    enemy_tags: Vec<StuffTag>,
    enemy_weights: Vec<i32>,
    item_tags: Vec<StuffTag>,
    item_weights: Vec<i32>,
}

pub type EntityChanceList<'a> = &'a [(u32, &'a [(StuffTag, i32)])];
fn entity_weighted_chances(floor: u32, chances: EntityChanceList) -> (Vec<StuffTag>, Vec<i32>) {
    // allow overriding chances on higher floors
    let mut weighted_chances = HashMap::new();
    for (key, weights) in chances {
        if key > &floor {
            break;
        }
        for (tag, weight) in *weights {
            weighted_chances.insert(*tag, *weight);
        }
    }

    (
        weighted_chances.keys().copied().collect(),
        weighted_chances.values().copied().collect(),
    )
}

impl EntityChances {
    pub fn from_level(level: u32) -> Self {
        let mut result = Self::default();

        (result.enemy_tags, result.enemy_weights) = entity_weighted_chances(level, ENEMY_CHANCES);
        (result.item_tags, result.item_weights) = entity_weighted_chances(level, ITEM_CHANCES);

        debug_assert_eq!(result.enemy_weights.len(), result.enemy_tags.len());
        debug_assert_eq!(result.item_weights.len(), result.item_tags.len());

        result
    }
}

/// return number of monsters placed
fn place_entities(
    rng: &mut impl Rng,
    grid: &mut Grid<Option<StuffTag>>,
    room: &RectRoom,
    max_monsters: u32,
    weights: &EntityChances,
) -> u32 {
    let n_monsters_a = rng.gen_range(0..=max_monsters);
    let n_monsters_b = rng.gen_range(0..=max_monsters);
    let n_monsters = n_monsters_a.max(n_monsters_b); // bias towards more monsters

    let dist = rand::distributions::WeightedIndex::new(&weights.enemy_weights[..]).unwrap();

    for _ in 0..n_monsters {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = weights.enemy_tags[dist.sample(rng)];
            grid[pos] = Some(tag);
            debug!("Placing {:?} at {}", tag, pos);
        }
    }
    n_monsters
}

fn place_stairs(rng: &mut impl Rng, grid: &mut Grid<Option<StuffTag>>, room: &RectRoom) {
    loop {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            debug!("Placing end at {}", pos);
            grid[pos] = Some(StuffTag::Stairs);
            return;
        }
    }
}

fn place_items(
    rng: &mut impl Rng,
    grid: &mut Grid<Option<StuffTag>>,
    room: &RectRoom,
    min_items: u32,
    max_items: u32,
    weights: &EntityChances,
) {
    let n_items = rng.gen_range(min_items..=max_items);

    let dist = rand::distributions::WeightedIndex::new(&weights.item_weights[..]).unwrap();

    for _ in 0..n_items {
        let x = rng.gen_range(room.min.x + 1..room.max.x + 1);
        let y = rng.gen_range(room.min.y + 1..room.max.y + 1);

        let pos = Vec2::new(x, y);
        if grid[pos].is_none() {
            let tag = weights.item_tags[dist.sample(rng)];
            grid[pos] = Some(tag);
            debug!("Placing {:?} at {}", tag, pos);
        }
    }
}

pub fn generate_map(
    player_q: Query<EntityId, With<PlayerTag>>,
    entities: Query<EntityId>,
    mut cmd: Commands,
    mut grid: ResMut<Grid<Stuff>>,
    props: Res<MapGenProps>,
    dims: Res<WorldDims>,
    floor: Res<DungeonFloor>,
) {
    // player may or may not exist at this point
    let player_id = player_q.iter().next();
    for (_p, stuff) in grid.iter_mut() {
        if let Some(id) = stuff {
            // delete all but player entities from the database
            // player is preserved between levels
            // ignore stale ids
            let id = *id;
            if entities.fetch(id).is_some() && Some(id) != player_id {
                cmd.delete(id);
            }
        }
        *stuff = None;
    }
    let mut working_grid = Grid::new(dims.0);
    working_grid.fill(Some(StuffTag::Wall));
    build_rooms(&mut working_grid, &props, floor.current);

    // insert entities into db
    //
    if dims.0 != grid.dims() {
        *grid = Grid::new(dims.0);
    }
    'insert_loop: for (pos, tag) in working_grid.iter().filter_map(|(p, t)| t.map(|t| (p, t))) {
        match tag {
            StuffTag::Player => {
                if let Some(player_id) = player_id {
                    // update player pos
                    cmd.entity(player_id).insert(Pos(pos));
                    grid[pos] = Some(player_id);
                } else {
                    init_entity(pos, tag, &mut cmd, &mut grid);
                }
            }
            StuffTag::Wall => {
                // clear invisible walls
                for y in -1..=1 {
                    for x in -1..=1 {
                        let Some(stuff) = working_grid.at(pos.x + x, pos.y + y) else {
                            // walls at the outer edge are removed
                            continue;
                        };
                        match stuff.as_ref() {
                            Some(StuffTag::Wall) => {
                                // if all neighbours are walls then this wall is removed
                            }
                            _ => {
                                init_entity(pos, tag, &mut cmd, &mut grid);
                                continue 'insert_loop;
                            }
                        }
                    }
                }
            }
            _ => {
                init_entity(pos, tag, &mut cmd, &mut grid);
            }
        }
    }
}

struct Mst {
    pub edges: Vec<[u32; 2]>,
    #[allow(unused)]
    pub parents: Vec<i32>,
}

/// return edges, which are indices into `points`
///
/// the edges in the fully connected graph are sorted by the rooms' manhatten distance
fn minimum_spanning_tree(points: &[RectRoom]) -> Mst {
    let mut f = Vec::with_capacity(points.len());
    let mut edges = points
        .iter()
        .enumerate()
        .skip(1)
        .flat_map(|(i, a)| {
            points[..i]
                .iter()
                .enumerate()
                .map(move |(j, b)| (a.center().manhatten(b.center()), [i, j]))
        })
        .collect::<Vec<_>>();
    edges.sort_by_key(|(w, _)| *w);

    let mut parents = vec![-1; points.len()];

    'edges: for (_, [i, j]) in edges {
        for mut ii in [i, j] {
            while parents[ii] != -1 {
                if parents[ii] == j as i32 {
                    // graph would have a circle
                    continue 'edges;
                }
                ii = parents[ii] as usize;
            }
        }
        if parents[j] == -1 {
            parents[j] = i as i32;
            f.push([i as u32, j as u32]);
            if f.len() == points.len() {
                break 'edges;
            }
        }
    }

    debug_assert_eq!(
        1,
        parents.iter().filter(|i| i == &&-1).count(),
        "there must be exactly 1 node with no parent in the minimum spanning tree"
    );

    Mst { edges: f, parents }
}

fn adjacency_set(matrix: &mut Grid<i8>, i: i32, j: i32, val: i8) {
    matrix[Vec2::new(i, j)] = val;
    matrix[Vec2::new(j, i)] = val;
}

fn build_rooms(grid: &mut Grid<Option<StuffTag>>, props: &MapGenProps, floor: u32) {
    let mut rng = rand::thread_rng();
    let mut rooms = Vec::<RectRoom>::with_capacity(props.max_rooms as usize);

    'outer: for _ in 0..props.max_rooms {
        let width = rng.gen_range(props.room_min_size..props.room_max_size) as i32;
        let height = rng.gen_range(props.room_min_size..props.room_max_size) as i32;

        // -3 so all rooms have walls, even those that touch the end of the map
        const PADDING: i32 = 3;
        let x = rng.gen_range(PADDING..grid.width() - 1 - PADDING - width);
        let y = rng.gen_range(PADDING..grid.height() - 1 - PADDING - height);

        let room = RectRoom::new(x, y, width, height);
        for r in rooms.iter() {
            if room.touches(r) {
                continue 'outer;
            }
        }
        room.carve(grid);
        rooms.push(room);
    }

    debug!(?rooms, "Rooms");

    let tree = minimum_spanning_tree(&rooms);
    let mut edges = tree.edges;
    let mut adjacency = Grid::new(Vec2::splat(rooms.len() as i32));
    for [i, j] in edges.iter() {
        adjacency_set(&mut adjacency, *i as i32, *j as i32, 1);
    }

    // let's put some edges back into the graph to produce some circles
    // the way we connect can produce circles by itself, but it's not that reliable
    for _ in 0..(rooms.len() / 4).max(1) {
        let [i, j] = loop {
            let [i, j] = [
                rng.gen_range(0..rooms.len()) as i32,
                rng.gen_range(0..rooms.len()) as i32,
            ];
            if i != j && adjacency[Vec2::new(i, j)] == 0 {
                break [i, j];
            }
        };
        adjacency_set(&mut adjacency, i, j, 1);
        edges.push([i as u32, j as u32]);
    }

    'edges: while let Some([i, j]) = edges.pop() {
        let r1 = &rooms[i as usize];
        let r2 = &rooms[j as usize];
        let c1 = r1.center();
        let c2 = r2.center();

        // if the tunnel would cut through a room replace it with two tunnels
        let tunnel = tunnel_between(&mut rng, c1, c2);
        for (k, room) in rooms.iter().enumerate() {
            if k == i as usize || k == j as usize {
                continue;
            }
            if room.intersects_segment(tunnel.current, tunnel.corner)
                || room.intersects_segment(tunnel.end, tunnel.corner)
            {
                debug!(i, j, k, "Splitting tunnel");
                // avoid duplicate edges to the same pair of rooms
                adjacency_set(&mut adjacency, i as i32, j as i32, 0);

                if adjacency[Vec2::new(i as i32, k as i32)] == 0 {
                    adjacency_set(&mut adjacency, i as i32, k as i32, 1);
                    edges.push([i, k as u32]);
                }
                if adjacency[Vec2::new(j as i32, k as i32)] == 0 {
                    adjacency_set(&mut adjacency, i as i32, k as i32, 1);
                    edges.push([j, k as u32]);
                }
                continue 'edges;
            }
        }

        for p in tunnel {
            grid[p] = None;
        }
    }

    // place doors in room gaps
    // tunnels between room A and B may cut through room C so use a separate loop to fill doors
    // instead of eagerly in the connector loop
    for room in rooms.iter() {
        for p in iter_edge(room) {
            if grid[p].is_none() {
                grid[p] = Some(StuffTag::Door);
            }
        }
        for p in iter_corners(room) {
            if grid[p].map(|x| x == StuffTag::Door).unwrap_or(false) {
                debug!(?p, "Door in corner");
                // if there's a door in the corner we're going to have a door-chain

                // clear consecutive doors, leaving the corner door
                let center = room.center();
                let d = (center - p).as_direction();
                for y in 1..room.max.y - room.min.y {
                    let p = Vec2::new(p.x, p.y + y * d.y);
                    if grid[p] == Some(StuffTag::Door) {
                        grid[p].take();
                    } else {
                        break;
                    }
                }
                for x in 1..room.max.x - room.min.x {
                    let p = Vec2::new(p.x + x * d.x, p.y);
                    if grid[p] == Some(StuffTag::Door) {
                        grid[p].take();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    let entity_weights = EntityChances::from_level(floor);
    if floor == 1 {
        // give a starting item on floor 1
        place_items(&mut rng, grid, &rooms[0], 1, 1, &entity_weights);
    }
    // spawn the player in the first room
    grid[rooms[0].center()] = Some(StuffTag::Player);

    // place stuff
    for room in rooms.iter().skip(1) {
        let n = place_entities(
            &mut rng,
            grid,
            room,
            props.max_monsters_per_floor,
            &entity_weights,
        );
        // ensure no room is empty by placing at least 1 item in rooms with no monsters
        let min_items = (n == 0) as u32;
        place_items(
            &mut rng,
            grid,
            room,
            min_items,
            props.max_items_per_floor,
            &entity_weights,
        );
    }

    let end_room = rooms[1..]
        .choose(&mut rng)
        .expect("Expected more than 1 room");
    place_stairs(&mut rng, grid, end_room);
}

fn tunnel_between(mut rng: impl Rng, start: Vec2, end: Vec2) -> TunnelIter {
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

fn iter_edge(room: &RectRoom) -> impl Iterator<Item = Vec2> + '_ {
    // +-1 in x to touch the corners
    (room.min.x - 1..=room.max.x + 1)
        .flat_map(|x| [Vec2::new(x, room.min.y - 1), Vec2::new(x, room.max.y + 1)])
        .chain(
            (room.min.y..=room.max.y)
                .flat_map(|y| [Vec2::new(room.min.x - 1, y), Vec2::new(room.max.x + 1, y)]),
        )
}

fn iter_corners(room: &RectRoom) -> impl Iterator<Item = Vec2> + '_ {
    [room.min.x - 1, room.max.x + 1]
        .into_iter()
        .flat_map(|x| [Vec2::new(x, room.min.y - 1), Vec2::new(x, room.max.y + 1)].into_iter())
}
