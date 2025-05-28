use crate::simulation::entities::{Map, ResourceType, Robot, Station};
use crate::simulation::robot_ai::behavior::RobotBehavior;
use crate::simulation::robot_ai::types::{ExploreTask, HarvestTask, Task, TaskType};
use crate::simulation::robot_ai::utils::SearchUtils;

pub struct HarvesterBehavior;

impl RobotBehavior for HarvesterBehavior {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task> {
        if robot.carrying.is_some() || robot.energy < 15 {
            return Some(Task {
                task_type: TaskType::ReturnToStation,
                target_position: Some((station.x, station.y)),
                priority: 10,
            });
        }

        if let Some((pos, resource_type)) = self.find_nearby_resource(robot, map) {
            return Some(Task {
                task_type: TaskType::Harvest(HarvestTask {
                    resource_type,
                    target_position: pos,
                }),
                target_position: Some(pos),
                priority: 8,
            });
        }

        if let Some(exploration_target) = self.get_resource_exploration_target(robot, map) {
            return Some(Task {
                task_type: TaskType::Explore(ExploreTask {
                    target_area: exploration_target,
                    radius: 2,
                }),
                target_position: Some(exploration_target),
                priority: 6,
            });
        }

        None
    }

    fn get_preferred_resources(&self) -> Vec<ResourceType> {
        vec![ResourceType::Energy, ResourceType::Mineral]
    }

    fn get_energy_consumption_rate(&self) -> u32 {
        3
    }

    fn can_perform_task(&self, task: &Task) -> bool {
        matches!(
            task.task_type,
            TaskType::Harvest(_) | TaskType::Explore(_) | TaskType::ReturnToStation
        )
    }
}

impl HarvesterBehavior {
    fn find_nearby_resource(
        &self,
        robot: &Robot,
        map: &Map,
    ) -> Option<((usize, usize), ResourceType)> {
        let preferred_types = vec![ResourceType::Energy, ResourceType::Mineral];
        SearchUtils::find_nearest_resource(robot.x, robot.y, 4, map, &preferred_types)
    }

    fn get_resource_exploration_target(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        let target_x = if robot.x < map.width / 2 {
            robot.x + 2
        } else {
            robot.x.saturating_sub(2)
        };
        let target_y = if robot.y < map.height / 2 {
            robot.y + 2
        } else {
            robot.y.saturating_sub(2)
        };

        let target_x = target_x.min(map.width - 1);
        let target_y = target_y.min(map.height - 1);

        if map.terrain[target_y][target_x] == 0 {
            Some((target_x, target_y))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::{Map, ResourceType, Robot, RobotType, Station};
    use crate::simulation::robot_ai::types::{
        AnalysisType, AnalyzeTask, HarvestTask, Task, TaskType,
    };
    use std::collections::HashMap;

    fn create_test_robot(x: usize, y: usize, energy: u32) -> Robot {
        Robot {
            id: 1,
            robot_type: RobotType::Harvester,
            x,
            y,
            energy,
            carrying: None,
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

    fn create_test_map_with_resources() -> Map {
        let mut map = Map::new_test_map(10, 10);
        map.resources.insert((3, 3), (ResourceType::Energy, 10));
        map.resources.insert((7, 7), (ResourceType::Mineral, 15));
        map.resources
            .insert((2, 8), (ResourceType::ScientificInterest, 5));
        map
    }

    #[test]
    fn test_harvester_behavior_characteristics() {
        let harvester = HarvesterBehavior;
        assert_eq!(harvester.get_energy_consumption_rate(), 3);
        assert_eq!(
            harvester.get_preferred_resources(),
            vec![ResourceType::Energy, ResourceType::Mineral]
        );
    }

    #[test]
    fn test_harvester_low_energy_returns_to_station() {
        let harvester = HarvesterBehavior;
        let map = create_test_map_with_resources();
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
        let map = create_test_map_with_resources();
        let station = create_test_station();
        let mut carrying_robot = create_test_robot(3, 3, 50);
        carrying_robot.carrying = Some((ResourceType::Mineral, 10));

        let task = harvester.decide_next_action(&carrying_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
    }

    #[test]
    fn test_harvester_finds_preferred_resources() {
        let harvester = HarvesterBehavior;
        let map = create_test_map_with_resources();
        let station = create_test_station();
        let robot = create_test_robot(1, 1, 50);

        let task = harvester.decide_next_action(&robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();

        if let TaskType::Harvest(harvest_task) = task.task_type {
            assert!(matches!(
                harvest_task.resource_type,
                ResourceType::Energy | ResourceType::Mineral
            ));
            assert!(task.target_position.is_some());
        } else {
            panic!("Expected harvest task, got {:?}", task.task_type);
        }
    }

    #[test]
    fn test_harvester_can_perform_harvest_tasks() {
        let harvester = HarvesterBehavior;

        let harvest_task = Task {
            task_type: TaskType::Harvest(HarvestTask {
                resource_type: ResourceType::Energy,
                target_position: (3, 3),
            }),
            target_position: Some((3, 3)),
            priority: 8,
        };

        let return_task = Task {
            task_type: TaskType::ReturnToStation,
            target_position: Some((5, 5)),
            priority: 9,
        };

        assert!(harvester.can_perform_task(&harvest_task));
        assert!(harvester.can_perform_task(&return_task));
    }

    #[test]
    fn test_harvester_cannot_perform_non_harvest_tasks() {
        let harvester = HarvesterBehavior;

        let analyze_task = Task {
            task_type: TaskType::Analyze(AnalyzeTask {
                target_position: (2, 2),
                analysis_type: AnalysisType::Chemical,
            }),
            target_position: Some((2, 2)),
            priority: 7,
        };

        assert!(!harvester.can_perform_task(&analyze_task));
    }

    #[test]
    fn test_find_nearby_resource_prioritizes_preferred() {
        let harvester = HarvesterBehavior;
        let map = create_test_map_with_resources();
        let robot = create_test_robot(4, 4, 50);

        let result = harvester.find_nearby_resource(&robot, &map);

        assert!(result.is_some());
        let (pos, resource_type) = result.unwrap();

        assert!(matches!(
            resource_type,
            ResourceType::Energy | ResourceType::Mineral
        ));

        assert!(pos.0 < map.width);
        assert!(pos.1 < map.height);
    }

    #[test]
    fn test_harvester_with_no_resources_available() {
        let harvester = HarvesterBehavior;
        let map = Map::new_test_map(10, 10);
        let station = create_test_station();
        let robot = create_test_robot(3, 3, 50);

        let task = harvester.decide_next_action(&robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert!(matches!(task.task_type, TaskType::Explore(_)));
    }

    #[test]
    fn test_harvester_energy_efficiency() {
        let harvester = HarvesterBehavior;

        assert!(harvester.get_energy_consumption_rate() > 2);

        assert_eq!(harvester.get_energy_consumption_rate(), 3);
    }

    #[test]
    fn test_harvester_resource_priority() {
        let harvester = HarvesterBehavior;
        let preferred = harvester.get_preferred_resources();

        assert!(preferred.contains(&ResourceType::Energy));
        assert!(preferred.contains(&ResourceType::Mineral));
        assert!(!preferred.contains(&ResourceType::ScientificInterest));
    }

    #[test]
    fn test_harvester_healthy_robot_harvests() {
        let harvester = HarvesterBehavior;
        let map = create_test_map_with_resources();
        let station = create_test_station();
        let healthy_robot = create_test_robot(1, 1, 50);

        let task = harvester.decide_next_action(&healthy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();

        if !map.resources.is_empty() {
            assert!(matches!(task.task_type, TaskType::Harvest(_)));
        }
    }
}
