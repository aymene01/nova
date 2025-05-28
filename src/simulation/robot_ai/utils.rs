use crate::simulation::entities::{Map, ResourceType};

/// Utility functions for robot behaviors to reduce code duplication
pub struct SearchUtils;

impl SearchUtils {
    /// Performs a radius-based search around a robot's position
    /// Returns the first position that matches the given predicate function
    pub fn radius_search<F>(
        robot_x: usize,
        robot_y: usize,
        search_radius: i32,
        map: &Map,
        predicate: F,
    ) -> Option<(usize, usize)>
    where
        F: Fn(usize, usize, &Map) -> bool,
    {
        let robot_x = robot_x as i32;
        let robot_y = robot_y as i32;

        for radius in 1..=search_radius {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    let x = robot_x + dx;
                    let y = robot_y + dy;
                    
                    if x >= 0 && y >= 0 && (x as usize) < map.width && (y as usize) < map.height {
                        let x = x as usize;
                        let y = y as usize;
                        
                        if predicate(x, y, map) {
                            return Some((x, y));
                        }
                    }
                }
            }
        }
        None
    }

    /// Finds the nearest resource of specific types within a search radius
    pub fn find_nearest_resource(
        robot_x: usize,
        robot_y: usize,
        search_radius: i32,
        map: &Map,
        allowed_types: &[ResourceType],
    ) -> Option<((usize, usize), ResourceType)> {
        Self::radius_search(robot_x, robot_y, search_radius, map, |x, y, map| {
            if let Some((resource_type, _amount)) = map.resources.get(&(x, y)) {
                allowed_types.contains(resource_type)
            } else {
                false
            }
        }).and_then(|(x, y)| {
            map.resources.get(&(x, y)).map(|(resource_type, _)| ((x, y), resource_type.clone()))
        })
    }

    /// Finds the nearest unexplored area within a search radius
    pub fn find_nearest_unexplored(
        robot_x: usize,
        robot_y: usize,
        search_radius: i32,
        map: &Map,
    ) -> Option<(usize, usize)> {
        Self::radius_search(robot_x, robot_y, search_radius, map, |x, y, map| {
            !map.discovered[y][x] && map.terrain[y][x] == 0
        })
    }

    /// Finds the nearest scientific interest within a search radius
    pub fn find_nearest_scientific_interest(
        robot_x: usize,
        robot_y: usize,
        search_radius: i32,
        map: &Map,
    ) -> Option<(usize, usize)> {
        Self::radius_search(robot_x, robot_y, search_radius, map, |x, y, map| {
            if let Some((ResourceType::ScientificInterest, _)) = map.resources.get(&(x, y)) {
                true
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::{Map, ResourceType};

    fn create_test_map() -> Map {
        Map::new_test_map(10, 10)
    }

    #[test]
    fn test_radius_search_basic_functionality() {
        let map = create_test_map();
        
        let result = SearchUtils::radius_search(5, 5, 2, &map, |x, y, _| {
            x == 6 && y == 6
        });
        
        assert_eq!(result, Some((6, 6)));
    }

    #[test]
    fn test_radius_search_respects_bounds() {
        let map = create_test_map();
        
        let result = SearchUtils::radius_search(0, 0, 2, &map, |x, y, _| {
            x > 10 || y > 10
        });
        
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_nearest_resource() {
        let mut map = create_test_map();
        map.resources.insert((3, 3), (ResourceType::Energy, 10));
        map.resources.insert((7, 7), (ResourceType::Mineral, 15));
        
        let allowed_types = vec![ResourceType::Energy, ResourceType::Mineral];
        let result = SearchUtils::find_nearest_resource(2, 2, 5, &map, &allowed_types);
        
        assert!(result.is_some());
        let ((x, y), resource_type) = result.unwrap();
        assert_eq!((x, y), (3, 3));
        assert_eq!(resource_type, ResourceType::Energy);
    }

    #[test]
    fn test_find_nearest_unexplored() {
        let mut map = create_test_map();
        map.discovered[5][5] = true;
        
        let result = SearchUtils::find_nearest_unexplored(5, 5, 3, &map);
        
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        assert!(!map.discovered[y][x]);
        assert_eq!(map.terrain[y][x], 0);
    }

    #[test]
    fn test_find_nearest_scientific_interest() {
        let mut map = create_test_map();
        map.resources.insert((4, 4), (ResourceType::ScientificInterest, 8));
        
        let result = SearchUtils::find_nearest_scientific_interest(3, 3, 5, &map);
        
        assert_eq!(result, Some((4, 4)));
    }
} 