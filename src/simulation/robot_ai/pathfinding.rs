use crate::simulation::entities::Map;
use crate::simulation::robot_ai::types::Direction;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

pub struct Pathfinder;

impl Pathfinder {
    pub fn new() -> Self {
        Pathfinder
    }

    pub fn find_path(
        &self,
        start: (usize, usize),
        goal: (usize, usize),
        map: &Map,
    ) -> Option<Vec<(usize, usize)>> {
        #[derive(Debug)]
        struct Node {
            position: (usize, usize),
            g_cost: u32,
            f_cost: u32,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                other.f_cost.cmp(&self.f_cost)
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for Node {
            fn eq(&self, other: &Self) -> bool {
                self.f_cost == other.f_cost && self.g_cost == other.g_cost
            }
        }

        impl Eq for Node {}

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), u32> = HashMap::new();

        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            g_cost: 0,
            f_cost: self.heuristic(start, goal),
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                let mut path = vec![current.position];
                let mut current_pos = current.position;
                
                while let Some(&parent) = came_from.get(&current_pos) {
                    path.push(parent);
                    current_pos = parent;
                }
                
                path.reverse();
                return Some(path);
            }

            for direction in Direction::all() {
                let (dx, dy) = direction.to_delta();
                let new_x = current.position.0 as i32 + dx;
                let new_y = current.position.1 as i32 + dy;

                if new_x < 0 || new_y < 0 || 
                   new_x >= map.width as i32 || new_y >= map.height as i32 {
                    continue;
                }

                let neighbor = (new_x as usize, new_y as usize);
                
                if map.terrain[neighbor.1][neighbor.0] != 0 {
                    continue;
                }

                let tentative_g_score = g_score[&current.position] + 1;

                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g_score);
                    
                    let f_score = tentative_g_score + self.heuristic(neighbor, goal);
                    open_set.push(Node {
                        position: neighbor,
                        g_cost: tentative_g_score,
                        f_cost: f_score,
                    });
                }
            }
        }

        None
    }

    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> u32 {
        let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
        let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
        (dx + dy) as u32
    }

    pub fn get_next_move(&self, current: (usize, usize), target: (usize, usize), map: &Map) -> Option<Direction> {
        if let Some(path) = self.find_path(current, target, map) {
            if path.len() > 1 {
                let next_pos = path[1];
                let dx = next_pos.0 as i32 - current.0 as i32;
                let dy = next_pos.1 as i32 - current.1 as i32;
                
                match (dx, dy) {
                    (0, -1) => Some(Direction::North),
                    (0, 1) => Some(Direction::South),
                    (1, 0) => Some(Direction::East),
                    (-1, 0) => Some(Direction::West),
                    (1, -1) => Some(Direction::Northeast),
                    (-1, -1) => Some(Direction::Northwest),
                    (1, 1) => Some(Direction::Southeast),
                    (-1, 1) => Some(Direction::Southwest),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::Map;

    fn create_test_map(width: usize, height: usize) -> Map {
        Map::new_test_map(width, height)
    }

    fn create_map_with_obstacles(width: usize, height: usize, obstacles: Vec<(usize, usize)>) -> Map {
        let mut map = create_test_map(width, height);
        for (x, y) in obstacles {
            if x < width && y < height {
                map.terrain[y][x] = 1;
            }
        }
        map
    }

    #[test]
    fn test_pathfinder_creation() {
        let pathfinder = Pathfinder::new();
        assert_eq!(std::mem::size_of_val(&pathfinder), 0);
    }

    #[test]
    fn test_find_path_same_position() {
        let pathfinder = Pathfinder::new();
        let map = create_test_map(5, 5);
        let start = (2, 2);
        let goal = (2, 2);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], start);
    }

    #[test]
    fn test_find_path_adjacent_positions() {
        let pathfinder = Pathfinder::new();
        let map = create_test_map(5, 5);
        let start = (2, 2);
        let goal = (3, 2);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], start);
        assert_eq!(path[1], goal);
    }

    #[test]
    fn test_find_path_straight_line() {
        let pathfinder = Pathfinder::new();
        let map = create_test_map(10, 10);
        let start = (0, 0);
        let goal = (5, 0);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 6);
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], goal);
    }

    #[test]
    fn test_find_path_with_obstacles() {
        let pathfinder = Pathfinder::new();
        let obstacles = vec![(1, 0), (1, 1), (1, 2)];
        let map = create_map_with_obstacles(5, 5, obstacles);
        let start = (0, 1);
        let goal = (2, 1);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], goal);
        
        for &(x, y) in &path {
            assert_eq!(map.terrain[y][x], 0, "Path goes through obstacle at ({}, {})", x, y);
        }
    }

    #[test]
    fn test_find_path_impossible() {
        let pathfinder = Pathfinder::new();
        let mut obstacles = Vec::new();
        for x in 1..4 {
            for y in 1..4 {
                if x != 2 || y != 2 {
                    obstacles.push((x, y));
                }
            }
        }
        let map = create_map_with_obstacles(5, 5, obstacles);
        let start = (0, 0);
        let goal = (2, 2);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_none(), "Should not find path to completely blocked goal");
    }

    #[test]
    fn test_find_path_out_of_bounds() {
        let pathfinder = Pathfinder::new();
        let map = create_test_map(5, 5);
        let start = (2, 2);
        let goal = (10, 10);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_none(), "Should not find path to out-of-bounds goal");
    }

    #[test]
    fn test_get_next_move_same_position() {
        let pathfinder = Pathfinder::new();
        let current = (2, 2);
        let target = (2, 2);

        let direction = pathfinder.get_next_move(current, target, &create_test_map(5, 5));
        
        assert!(direction.is_none(), "Should not move when already at target");
    }

    #[test]
    fn test_get_next_move_cardinal_directions() {
        let pathfinder = Pathfinder::new();
        let current = (2, 2);

        let direction = pathfinder.get_next_move(current, (2, 1), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::North));

        let direction = pathfinder.get_next_move(current, (2, 3), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::South));

        let direction = pathfinder.get_next_move(current, (3, 2), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::East));

        let direction = pathfinder.get_next_move(current, (1, 2), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::West));
    }

    #[test]
    fn test_get_next_move_diagonal_directions() {
        let pathfinder = Pathfinder::new();
        let current = (2, 2);

        let direction = pathfinder.get_next_move(current, (3, 1), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::Northeast));

        let direction = pathfinder.get_next_move(current, (1, 1), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::Northwest));

        let direction = pathfinder.get_next_move(current, (3, 3), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::Southeast));

        let direction = pathfinder.get_next_move(current, (1, 3), &create_test_map(5, 5));
        assert_eq!(direction, Some(Direction::Southwest));
    }

    #[test]
    fn test_get_next_move_distant_target() {
        let pathfinder = Pathfinder::new();
        let current = (1, 1);
        let target = (4, 4);

        let direction = pathfinder.get_next_move(current, target, &create_test_map(6, 6));
        
        assert!(direction.is_some());
        assert_eq!(direction, Some(Direction::Southeast));
    }

    #[test]
    fn test_heuristic_function() {
        let pathfinder = Pathfinder::new();
        
        assert_eq!(pathfinder.heuristic((0, 0), (3, 4)), 7);
        assert_eq!(pathfinder.heuristic((2, 2), (2, 2)), 0);
        assert_eq!(pathfinder.heuristic((1, 1), (4, 5)), 7);
    }

    #[test]
    fn test_pathfinding_performance_small_map() {
        let pathfinder = Pathfinder::new();
        let map = create_test_map(20, 20);
        let start = (0, 0);
        let goal = (19, 19);

        let start_time = std::time::Instant::now();
        let path = pathfinder.find_path(start, goal, &map);
        let duration = start_time.elapsed();

        assert!(path.is_some());
        assert!(duration.as_millis() < 100, "Pathfinding should be fast on small maps");
    }

    #[test]
    fn test_path_optimality() {
        let pathfinder = Pathfinder::new();
        let map = create_test_map(5, 5);
        let start = (0, 0);
        let goal = (2, 2);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        
        assert!(path.len() <= 5, "Path should be reasonably optimal");
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], goal);
    }

    #[test]
    fn test_pathfinding_with_complex_maze() {
        let pathfinder = Pathfinder::new();
        
        let mut obstacles = Vec::new();
        for x in 1..4 {
            obstacles.push((x, 1));
            obstacles.push((x, 3));
        }
        obstacles.push((1, 2));
        obstacles.push((3, 2));
        
        let map = create_map_with_obstacles(5, 5, obstacles);
        let start = (0, 0);
        let goal = (4, 4);

        let path = pathfinder.find_path(start, goal, &map);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], goal);
        
        for &(x, y) in &path {
            assert_eq!(map.terrain[y][x], 0, "Path should not go through obstacles");
        }
    }
} 