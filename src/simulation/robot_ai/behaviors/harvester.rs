use crate::simulation::entities::{Map, ResourceType, Station};
use crate::simulation::robot_ai::behavior::RobotBehavior;
use crate::simulation::robot_ai::robot::Robot;
use crate::simulation::robot_ai::types::{HarvestTask, Task, TaskType};
use crate::simulation::robot_ai::utils::SearchUtils;

pub struct HarvesterBehavior;

impl RobotBehavior for HarvesterBehavior {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task> {
        if robot.carrying.is_some() || robot.is_low_energy() {
            return Some(Task {
                task_type: TaskType::ReturnToStation,
                target_position: Some((station.x, station.y)),
                priority: 10,
            });
        }

        if let Some((resource_type, position)) = self.find_preferred_resource(robot, map) {
            return Some(Task {
                task_type: TaskType::Harvest(HarvestTask {
                    resource_type,
                    target_position: position,
                }),
                target_position: Some(position),
                priority: 8,
            });
        }

        if let Some(exploration_target) = self.find_exploration_target(robot, map) {
            return Some(Task {
                task_type: TaskType::Explore(crate::simulation::robot_ai::types::ExploreTask {
                    target_area: exploration_target,
                    radius: 2,
                }),
                target_position: Some(exploration_target),
                priority: 6,
            });
        }

        None
    }

    fn get_energy_consumption_rate(&self) -> u32 {
        3
    }

    fn get_max_energy(&self) -> u32 {
        100
    }

    fn get_low_energy_threshold(&self) -> u32 {
        15
    }
}

impl HarvesterBehavior {
    fn find_preferred_resource(
        &self,
        robot: &Robot,
        map: &Map,
    ) -> Option<(ResourceType, (usize, usize))> {
        let preferred_types = vec![ResourceType::Energy, ResourceType::Mineral];

        for resource_type in preferred_types {
            if let Some((position, found_type)) = SearchUtils::find_nearest_resource(
                robot.x,
                robot.y,
                4,
                map,
                &[resource_type.clone()],
            ) {
                return Some((found_type, position));
            }
        }

        None
    }

    fn find_exploration_target(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        SearchUtils::find_nearest_unexplored(robot.x, robot.y, 4, map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::{Map, ResourceType, Station};
    use crate::simulation::robot_ai::types::{RobotState, RobotType, TaskType};
    use std::collections::HashMap;

    fn create_test_robot(x: usize, y: usize, energy: u32) -> Robot {
        Robot {
            id: 1,
            robot_type: RobotType::Harvester,
            x,
            y,
            energy,
            carrying: None,
            state: RobotState::Idle,
            behavior: Box::new(HarvesterBehavior),
        }
    }

    fn create_test_station() -> Station {
        Station {
            resources: HashMap::new(),
            discoveries: 0,
            x: 5,
            y: 5,
        }
    }

    fn create_test_map() -> Map {
        Map::new(10, 10, 42)
    }

    #[test]
    fn test_harvester_behavior_characteristics() {
        let harvester = HarvesterBehavior;
        assert_eq!(harvester.get_energy_consumption_rate(), 3);
    }

    #[test]
    fn test_harvester_low_energy_returns_to_station() {
        let harvester = HarvesterBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let low_energy_robot = create_test_robot(3, 3, 10);

        let task = harvester.decide_next_action(&low_energy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
        assert_eq!(task.priority, 10);
    }

    #[test]
    fn test_harvester_carrying_resources_returns_to_station() {
        let harvester = HarvesterBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let mut carrying_robot = create_test_robot(3, 3, 50);
        carrying_robot.carrying = Some((ResourceType::Energy, 25));

        let task = harvester.decide_next_action(&carrying_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
    }

    #[test]
    fn test_harvester_healthy_robot_harvests() {
        let harvester = HarvesterBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let healthy_robot = create_test_robot(3, 3, 80);

        let task = harvester.decide_next_action(&healthy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_ne!(task.task_type, TaskType::ReturnToStation);
    }

    #[test]
    fn test_harvester_finds_preferred_resources() {
        let harvester = HarvesterBehavior;
        let map = create_test_map();
        let robot = create_test_robot(3, 3, 50);

        let result = harvester.find_preferred_resource(&robot, &map);
        assert!(result.is_some());
    }

    #[test]
    fn test_harvester_finds_exploration_target() {
        let harvester = HarvesterBehavior;
        let mut map = create_test_map();
        let robot = create_test_robot(3, 3, 50);

        for row in map.discovered.iter_mut() {
            for cell in row.iter_mut() {
                *cell = true;
            }
        }
        map.discovered[3][2] = false;
        map.terrain[3][2] = 0;

        let result = harvester.find_exploration_target(&robot, &map);
        assert!(result.is_some());
    }
}
