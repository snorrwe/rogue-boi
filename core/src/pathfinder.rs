use std::collections::{BinaryHeap, HashMap};

use arrayvec::ArrayVec;
use cecs::query::Query;
use smallvec::SmallVec;
use tracing::debug;

use crate::components::Walkable;
use crate::{grid::Grid, math::Vec2, Stuff};

pub type Path = SmallVec<[Vec2; 16]>;

#[derive(Eq, Clone, Copy, Debug)]
struct Node {
    hcost: i32,
    gcost: i32,
    pos: Vec2,
}

impl Node {
    pub fn fcost(self) -> i32 {
        // it's used in a max-heap
        -(self.hcost + self.gcost)
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
    walkies: &Query<&Walkable>,
    path: &mut Path,
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
            reconstruct_path(from, current.pos, &came_from, path);
            return true;
        }
        let new_g = current.gcost + 1;

        let new_neighbours: ArrayVec<_, 4> = [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y]
            .iter()
            .map(|x| current.pos + *x)
            .filter(|pos| grid.contains(pos.x, pos.y))
            .map(move |pos| {
                let mut cost = new_g;
                // add additional cost for positions that have an entity on them
                // so there is always a path to the target
                if grid[pos].is_some() && walkies.fetch(grid[pos].unwrap()).is_none() {
                    cost += 50;
                }
                (pos, cost)
            })
            // if it's a new node, or if it's cheaper than the previous visit
            // this is required because we only check the cross neighbours, so our `f` const function is inconsistent
            .filter(|(pos, new_g)| gcost.get(pos).map(|cost| new_g < cost).unwrap_or(true))
            .collect();

        for (neighbour, new_g) in new_neighbours.into_iter() {
            came_from.insert(neighbour, current.pos);
            gcost.insert(neighbour, new_g);
            open_set.push(Node {
                hcost: neighbour.manhatten(to),
                gcost: new_g,
                pos: neighbour,
            });
        }
    }
    debug!("Failed to find path, from: {} to: {}", from, to,);
    false
}

fn reconstruct_path(target: Vec2, mut pos: Vec2, came_from: &HashMap<Vec2, Vec2>, path: &mut Path) {
    path.push(pos);
    while let Some(p) = came_from.get(&pos) {
        pos = *p;
        path.push(pos);
        if pos.manhatten(target) <= 1 {
            return;
        }
    }
}
