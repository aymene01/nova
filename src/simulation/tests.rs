#[cfg(test)]
mod map_tests {
    use crate::simulation::entities::{Map, ResourceType};
    use crate::simulation::map::TerrainType;
    use crate::simulation::robot_ai::robot::Robot;
    use crate::simulation::robot_ai::types::{Direction, RobotType};

    #[test]
    fn test_map_creation() {
        let width = 20;
        let height = 15;
        let seed = 42;

        let map = Map::new(width, height, seed);

        assert_eq!(map.width, width);
        assert_eq!(map.height, height);
        assert_eq!(map.seed, seed);

        assert!(!map.terrain.is_empty());
        assert_eq!(map.terrain.len(), height);
        assert_eq!(map.terrain[0].len(), width);

        assert!(!map.discovered.is_empty());
        assert_eq!(map.discovered.len(), height);
        assert_eq!(map.discovered[0].len(), width);

        assert!(!map.resources.is_empty());
    }

    #[test]
    fn test_terrain_distribution() {
        let width = 50;
        let height = 50;
        let seed = 42;

        let map = Map::new(width, height, seed);

        let mut plains = 0;
        let mut hills = 0;
        let mut mountains = 0;
        let mut canyons = 0;

        for y in 0..height {
            for x in 0..width {
                match TerrainType::from(map.terrain[y][x]) {
                    TerrainType::Plain => plains += 1,
                    TerrainType::Hill => hills += 1,
                    TerrainType::Mountain => mountains += 1,
                    TerrainType::Canyon => canyons += 1,
                }
            }
        }

        let total = width * height;

        assert!(plains > 0);
        assert!(hills > 0);
        assert!(mountains > 0);
        assert!(canyons > 0);

        // Verify terrain distribution (approximate)
        assert!((plains as f64 / total as f64) > 0.2); // At least 20% plains
        assert!((canyons as f64 / total as f64) < 0.3); // Less than 30% canyons
    }

    #[test]
    fn test_resource_distribution() {
        let width = 50;
        let height = 50;
        let seed = 42;

        let map = Map::new(width, height, seed);

        let mut energy = 0;
        let mut minerals = 0;
        let mut scientific = 0;

        for ((_, _), (res_type, _)) in &map.resources {
            match res_type {
                ResourceType::Energy => energy += 1,
                ResourceType::Mineral => minerals += 1,
                ResourceType::ScientificInterest => scientific += 1,
            }
        }

        assert!(energy > 0);
        assert!(minerals > 0);
        assert!(scientific > 0);

        assert!(energy > minerals);
        assert!(minerals > scientific);
    }

    #[test]
    fn test_map_reproducibility() {
        let width = 30;
        let height = 30;
        let seed = 12345;

        let map1 = Map::new(width, height, seed);
        let map2 = Map::new(width, height, seed);

        for y in 0..height {
            for x in 0..width {
                assert_eq!(map1.terrain[y][x], map2.terrain[y][x]);
            }
        }

        assert_eq!(map1.resources.len(), map2.resources.len());

        for ((x, y), (res_type, amount)) in &map1.resources {
            let resource2 = map2.resources.get(&(*x, *y));
            assert!(resource2.is_some());
            let (res_type2, amount2) = resource2.unwrap();
            assert_eq!(res_type, res_type2);
            assert_eq!(amount, amount2);
        }
    }

    #[test]
    fn test_resource_collection() {
        let mut map = Map::new_test_map(5, 5);
        map.resources.insert((2, 2), (ResourceType::Energy, 10));
        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);

        robot.collect_resource(&mut map).unwrap();
        assert_eq!(robot.carrying.unwrap().0, ResourceType::Energy);

        assert!(!map.resources.contains_key(&(2, 2)));
    }

    #[test]
    fn test_robots_never_leave_map_bounds() {
        let map = Map::new_test_map(10, 10);
        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 100);

        for step in 0..100 {
            let directions = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ];
            let direction = directions[step % 4];

            let _result = robot.move_in_direction(direction, &map);

            assert!(
                robot.x < map.width,
                "Robot left map horizontally at step {}",
                step
            );
            assert!(
                robot.y < map.height,
                "Robot left map vertically at step {}",
                step
            );
        }
    }

    #[test]
    fn test_robots_can_walk_on_resources() {
        let mut map = Map::new_test_map(5, 5);
        map.resources.insert((2, 2), (ResourceType::Energy, 10));
        map.resources.insert((3, 2), (ResourceType::Mineral, 15));
        map.resources
            .insert((2, 3), (ResourceType::ScientificInterest, 8));

        let mut explorer = Robot::new(1, RobotType::Explorer, 1, 1, 100);

        let result = explorer.move_in_direction(Direction::East, &map);
        assert!(
            result.is_ok(),
            "Explorer should be able to walk on energy resource"
        );
        assert_eq!(explorer.x, 2);
        assert_eq!(explorer.y, 1);

        let result = explorer.move_in_direction(Direction::South, &map);
        assert!(
            result.is_ok(),
            "Explorer should be able to walk on energy resource"
        );
        assert_eq!(explorer.x, 2);
        assert_eq!(explorer.y, 2);

        assert!(map.resources.contains_key(&(2, 2)));

        let mut harvester = Robot::new(2, RobotType::Harvester, 1, 1, 100);

        harvester.move_in_direction(Direction::East, &map).unwrap();
        harvester.move_in_direction(Direction::South, &map).unwrap();

        let collect_result = harvester.collect_resource(&mut map);
        assert!(
            collect_result.is_ok(),
            "Harvester should be able to collect energy"
        );
        assert!(harvester.carrying.is_some());
    }
}
