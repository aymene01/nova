use crate::domain::entities::map::Map;
use crate::domain::values::position::Position;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq)]
struct PathNode {
    cost: u32,
    position: (usize, usize),
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(dead_code)]
pub fn find_path(
    map: &Map,
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut heap = BinaryHeap::new();
    let mut distances: HashMap<(usize, usize), u32> = HashMap::new();
    let mut parents: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut visited: HashSet<(usize, usize)> = HashSet::new();

    distances.insert(start, 0);
    heap.push(PathNode {
        cost: 0,
        position: start,
    });

    while let Some(PathNode { cost, position }) = heap.pop() {
        if visited.contains(&position) {
            continue;
        }
        visited.insert(position);

        if position == goal {
            return Some(reconstruct_path(&parents, start, goal));
        }

        for neighbor in get_neighbors(map, position) {
            if visited.contains(&neighbor) {
                continue;
            }

            let movement_cost = get_movement_cost(map, neighbor);
            let new_cost = cost + movement_cost;

            if new_cost < *distances.get(&neighbor).unwrap_or(&u32::MAX) {
                distances.insert(neighbor, new_cost);
                parents.insert(neighbor, position);

                let heuristic = Position::from_tuple(neighbor)
                    .manhattan_distance_to(Position::from_tuple(goal));

                heap.push(PathNode {
                    cost: new_cost + heuristic,
                    position: neighbor,
                });
            }
        }
    }

    None
}

#[allow(dead_code)]
fn get_neighbors(map: &Map, position: (usize, usize)) -> Vec<(usize, usize)> {
    let (x, y) = position;
    let mut neighbors = Vec::new();

    let directions = [
        (0, -1), // North
        (1, 0),  // East
        (0, 1),  // South
        (-1, 0), // West
    ];

    for (dx, dy) in directions {
        let new_x = x as i32 + dx;
        let new_y = y as i32 + dy;

        if new_x >= 0 && new_y >= 0 && (new_x as usize) < map.width && (new_y as usize) < map.height
        {
            neighbors.push((new_x as usize, new_y as usize));
        }
    }

    neighbors
}

#[allow(dead_code)]
fn get_movement_cost(map: &Map, position: (usize, usize)) -> u32 {
    let terrain_type = map.terrain[position.1][position.0];
    match terrain_type {
        0 => 1, // Plain
        1 => 2, // Hill
        2 => 3, // Mountain
        3 => 1, // Canyon
        _ => 1, // Default
    }
}

#[allow(dead_code)]
fn reconstruct_path(
    parents: &HashMap<(usize, usize), (usize, usize)>,
    start: (usize, usize),
    goal: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    let mut current = goal;

    while current != start {
        path.push(current);
        current = parents[&current];
    }
    path.push(start);
    path.reverse();
    path
}
