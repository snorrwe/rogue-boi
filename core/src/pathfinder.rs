use std::collections::{BinaryHeap, HashMap};

use arrayvec::ArrayVec;
use smallvec::SmallVec;
use tracing::debug;

use crate::{grid::Grid, math::Vec2, Stuff};

#[derive(Eq, Clone, Copy, Debug)]
struct Node {
    hcost: i32,
    gcost: i32,
    pos: Vec2,
}

impl Node {
    pub fn fcost(self) -> f32 {
        // it's used in a max-heap so use the reciprocal of the cost
        1.0 / ((self.hcost + self.gcost) as f32).min(0.1)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fcost().partial_cmp(&other.fcost())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fcost()
            .partial_cmp(&other.fcost())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// return wether a path was found
///
/// uses manhatten distance
///
/// stops at 1 tile from `to`
///
/// the path is returned in reverse order, to walk it pop() the path vector
pub fn find_path(
    from: Vec2,
    to: Vec2,
    grid: &Grid<Stuff>,
    path: &mut SmallVec<[Vec2; 32]>,
) -> bool {
    let mut open_set = BinaryHeap::with_capacity(from.manhatten(to) as usize);
    let mut came_from = HashMap::new();
    let mut gcost = HashMap::new();

    open_set.push(Node {
        hcost: from.manhatten(to),
        gcost: 0,
        pos: from,
    });
    gcost.insert(from, 0);

    while let Some(current) = open_set.pop() {
        if current.pos.manhatten(to) <= 1 {
            reconstruct_path(current.pos, &came_from, path);
            return true;
        }
        let new_g = current.gcost + 1;

        let new_neighbours: ArrayVec<_, 4> = [
            Vec2::new(1, 0),
            Vec2::new(-1, 0),
            Vec2::new(0, 1),
            Vec2::new(0, -1),
        ]
        .iter()
        .map(|x| current.pos + *x)
        .filter(|pos| grid[*pos].is_none())
        // if it's a new node, or if it's cheaper than the previous visit
        // this is required because we only check the cross neighbours, so our `f` const function is inconsistent
        .filter(|pos| gcost.get(pos).map(|cost| new_g < *cost).unwrap_or(true))
        .collect();

        for neighbour in new_neighbours.into_iter() {
            came_from.insert(neighbour, current.pos);
            gcost.insert(neighbour, new_g);
            open_set.push(Node {
                hcost: neighbour.manhatten(to),
                gcost: new_g,
                pos: neighbour,
            });
        }
    }
    debug!(
        "Failed to find path, from: {} to: {}\ncame_from: {:#?}",
        from, to, came_from
    );
    false
}

fn reconstruct_path(
    mut pos: Vec2,
    came_from: &HashMap<Vec2, Vec2>,
    path: &mut SmallVec<[Vec2; 32]>,
) {
    path.push(pos);
    while let Some(p) = came_from.get(&pos) {
        path.push(*p);
        pos = *p;
    }
}
