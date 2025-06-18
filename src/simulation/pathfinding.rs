use crate::simulation::entities::{Direction, Map};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Represents a node in the pathfinding algorithm
#[derive(Debug, Clone, PartialEq, Eq)]
struct PathNode {
    position: (usize, usize),
    cost: u32,
    heuristic: u32,
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap behavior
        (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Pathfinding utilities for robots
pub struct Pathfinder;

impl Pathfinder {
    /// Calculate Manhattan distance between two points
    pub fn manhattan_distance(a: (usize, usize), b: (usize, usize)) -> u32 {
        let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
        let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
        (dx + dy) as u32
    }

    /// Find a path from start to goal using A* algorithm
    pub fn find_path(
        start: (usize, usize),
        goal: (usize, usize),
        map: &Map,
    ) -> Option<Vec<(usize, usize)>> {
        if start == goal {
            return Some(vec![start]);
        }

        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), u32> = HashMap::new();

        g_score.insert(start, 0);
        open_set.push(PathNode {
            position: start,
            cost: 0,
            heuristic: Self::manhattan_distance(start, goal),
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                return Some(Self::reconstruct_path(&came_from, current.position));
            }

            if closed_set.contains(&current.position) {
                continue;
            }
            closed_set.insert(current.position);

            // Check all neighbors
            for neighbor in Self::get_neighbors(current.position, map) {
                if closed_set.contains(&neighbor) {
                    continue;
                }

                let tentative_g_score = g_score[&current.position] + Self::movement_cost(neighbor, map);

                if !g_score.contains_key(&neighbor) || tentative_g_score < g_score[&neighbor] {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g_score);
                    
                    open_set.push(PathNode {
                        position: neighbor,
                        cost: tentative_g_score,
                        heuristic: Self::manhattan_distance(neighbor, goal),
                    });
                }
            }
        }

        None // No path found
    }

    /// Get valid neighboring positions
    fn get_neighbors(pos: (usize, usize), map: &Map) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let (x, y) = pos;

        // Check all four directions
        if x > 0 {
            neighbors.push((x - 1, y)); // West
        }
        if x + 1 < map.width {
            neighbors.push((x + 1, y)); // East
        }
        if y > 0 {
            neighbors.push((x, y - 1)); // North
        }
        if y + 1 < map.height {
            neighbors.push((x, y + 1)); // South
        }

        neighbors
    }

    /// Get movement cost for a position based on terrain
    fn movement_cost(pos: (usize, usize), map: &Map) -> u32 {
        // For now, use simple movement cost based on terrain type
        match map.get_terrain(pos.0, pos.1) {
            Ok(terrain) => map.movement_cost(terrain),
            Err(_) => u32::MAX, // Invalid position, very high cost
        }
    }

    /// Reconstruct the path from the came_from map
    fn reconstruct_path(
        came_from: &HashMap<(usize, usize), (usize, usize)>,
        mut current: (usize, usize),
    ) -> Vec<(usize, usize)> {
        let mut path = vec![current];
        
        while let Some(&prev) = came_from.get(&current) {
            current = prev;
            path.push(current);
        }
        
        path.reverse();
        path
    }

    /// Get the next direction to move towards a goal
    pub fn get_direction_to_goal(
        from: (usize, usize),
        to: (usize, usize),
        map: &Map,
    ) -> Option<Direction> {
        if let Some(path) = Self::find_path(from, to, map) {
            if path.len() >= 2 {
                let current = path[0];
                let next = path[1];
                return Self::position_to_direction(current, next);
            }
        }
        None
    }

    /// Convert two positions to a direction
    fn position_to_direction(from: (usize, usize), to: (usize, usize)) -> Option<Direction> {
        let dx = to.0 as i32 - from.0 as i32;
        let dy = to.1 as i32 - from.1 as i32;

        match (dx, dy) {
            (0, -1) => Some(Direction::North),
            (0, 1) => Some(Direction::South),
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::Map;

    #[test]
    fn manhattan_distance_calculation_works() {
        assert_eq!(Pathfinder::manhattan_distance((0, 0), (3, 4)), 7);
        assert_eq!(Pathfinder::manhattan_distance((5, 5), (5, 5)), 0);
        assert_eq!(Pathfinder::manhattan_distance((2, 1), (1, 3)), 3);
    }

    #[test]
    fn pathfinding_finds_simple_path() {
        let map = Map::new_test_map(5, 5);
        
        let path = Pathfinder::find_path((0, 0), (2, 2), &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&(0, 0)));
        assert_eq!(path.last(), Some(&(2, 2)));
        assert!(path.len() >= 3); // At least start, middle, end
    }

    #[test]
    fn pathfinding_returns_none_for_impossible_path() {
        // This test would need a map with obstacles, but for now we test the basic structure
        let map = Map::new_test_map(3, 3);
        
        let path = Pathfinder::find_path((0, 0), (10, 10), &map);
        
        // Should return None for out-of-bounds destination
        assert!(path.is_none());
    }

    #[test]
    fn direction_calculation_works() {
        let map = Map::new_test_map(5, 5);
        
        let direction = Pathfinder::get_direction_to_goal((1, 1), (1, 0), &map);
        assert_eq!(direction, Some(Direction::North));
        
        let direction = Pathfinder::get_direction_to_goal((1, 1), (2, 1), &map);
        assert_eq!(direction, Some(Direction::East));
    }
} 