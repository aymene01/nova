use crate::simulation::entities::{Map, ResourceType, Robot, Station};
use crate::simulation::robot_ai::behavior::RobotBehavior;
use crate::simulation::robot_ai::types::{ExploreTask, Task, TaskType};
use crate::simulation::robot_ai::utils::SearchUtils;

pub struct ExplorerBehavior;

impl RobotBehavior for ExplorerBehavior {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task> {
        if robot.energy < 20 || robot.carrying.is_some() {
            return Some(Task {
                task_type: TaskType::ReturnToStation,
                target_position: Some((station.x, station.y)),
                priority: 9,
            });
        }

        if let Some(unexplored_area) = self.find_unexplored_area(robot, map) {
            return Some(Task {
                task_type: TaskType::Explore(ExploreTask {
                    target_area: unexplored_area,
                    radius: 3,
                }),
                target_position: Some(unexplored_area),
                priority: 7,
            });
        }

        if let Some(random_target) = self.get_random_exploration_target(robot, map) {
            return Some(Task {
                task_type: TaskType::Explore(ExploreTask {
                    target_area: random_target,
                    radius: 2,
                }),
                target_position: Some(random_target),
                priority: 5,
            });
        }

        None
    }

    fn get_preferred_resources(&self) -> Vec<ResourceType> {
        vec![]
    }

    fn get_energy_consumption_rate(&self) -> u32 {
        2
    }

    fn can_perform_task(&self, task: &Task) -> bool {
        matches!(
            task.task_type,
            TaskType::Explore(_) | TaskType::ReturnToStation
        )
    }
}

impl ExplorerBehavior {
    fn find_unexplored_area(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        SearchUtils::find_nearest_unexplored(robot.x, robot.y, 5, map)
    }

    fn get_random_exploration_target(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        robot.id.hash(&mut hasher);
        robot.x.hash(&mut hasher);
        robot.y.hash(&mut hasher);
        let seed = hasher.finish();

        let target_x = ((seed % map.width as u64) as usize)
            .max(1)
            .min(map.width - 2);
        let target_y = (((seed >> 16) % map.height as u64) as usize)
            .max(1)
            .min(map.height - 2);

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
        AnalysisType, AnalyzeTask, ExploreTask, HarvestTask, Task, TaskType,
    };
    use std::collections::HashMap;

    fn create_test_robot(x: usize, y: usize, energy: u32) -> Robot {
        Robot {
            id: 1,
            robot_type: RobotType::Explorer,
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

    fn create_test_map() -> Map {
        Map::new_test_map(10, 10)
    }

    #[test]
    fn test_explorer_behavior_characteristics() {
        let explorer = ExplorerBehavior;
        assert_eq!(explorer.get_energy_consumption_rate(), 2);
        assert_eq!(explorer.get_preferred_resources(), vec![]);
    }

    #[test]
    fn test_explorer_low_energy_returns_to_station() {
        let explorer = ExplorerBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let low_energy_robot = create_test_robot(3, 3, 15);

        let task = explorer.decide_next_action(&low_energy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
        assert_eq!(task.priority, 9);
    }

    #[test]
    fn test_explorer_carrying_resources_returns_to_station() {
        let explorer = ExplorerBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let mut carrying_robot = create_test_robot(3, 3, 50);
        carrying_robot.carrying = Some((ResourceType::ScientificInterest, 5));

        let task = explorer.decide_next_action(&carrying_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
    }

    #[test]
    fn test_explorer_can_perform_exploration_tasks() {
        let explorer = ExplorerBehavior;

        let explore_task = Task {
            task_type: TaskType::Explore(ExploreTask {
                target_area: (5, 5),
                radius: 3,
            }),
            target_position: Some((5, 5)),
            priority: 7,
        };

        let return_task = Task {
            task_type: TaskType::ReturnToStation,
            target_position: Some((5, 5)),
            priority: 9,
        };

        assert!(explorer.can_perform_task(&explore_task));
        assert!(explorer.can_perform_task(&return_task));
    }

    #[test]
    fn test_explorer_cannot_perform_non_exploration_tasks() {
        let explorer = ExplorerBehavior;

        let harvest_task = Task {
            task_type: TaskType::Harvest(HarvestTask {
                resource_type: ResourceType::Energy,
                target_position: (3, 3),
            }),
            target_position: Some((3, 3)),
            priority: 8,
        };

        let analyze_task = Task {
            task_type: TaskType::Analyze(AnalyzeTask {
                target_position: (2, 2),
                analysis_type: AnalysisType::Chemical,
            }),
            target_position: Some((2, 2)),
            priority: 6,
        };

        assert!(!explorer.can_perform_task(&harvest_task));
        assert!(!explorer.can_perform_task(&analyze_task));
    }

    #[test]
    fn test_explorer_finds_unexplored_areas() {
        let explorer = ExplorerBehavior;
        let mut map = create_test_map();
        let station = create_test_station();
        let robot = create_test_robot(3, 3, 50);

        map.discovered[3][4] = true;
        map.discovered[3][5] = true;

        let task = explorer.decide_next_action(&robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();

        if let TaskType::Explore(explore_task) = task.task_type {
            assert_eq!(explore_task.radius, 3);
            assert!(task.target_position.is_some());
        } else {
            panic!("Expected explore task, got {:?}", task.task_type);
        }
    }

    #[test]
    fn test_explorer_healthy_robot_explores() {
        let explorer = ExplorerBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let healthy_robot = create_test_robot(1, 1, 50);

        let task = explorer.decide_next_action(&healthy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();

        assert_ne!(task.task_type, TaskType::ReturnToStation);
        assert!(matches!(task.task_type, TaskType::Explore(_)));
    }

    #[test]
    fn test_find_unexplored_area_private_method() {
        let explorer = ExplorerBehavior;
        let mut map = create_test_map();
        let robot = create_test_robot(5, 5, 50);

        map.discovered[5][5] = true;

        let result = explorer.find_unexplored_area(&robot, &map);

        if let Some((x, y)) = result {
            assert!(!map.discovered[y][x]);
            assert_eq!(map.terrain[y][x], 0);
        }
    }

    #[test]
    fn test_get_random_exploration_target_deterministic() {
        let explorer = ExplorerBehavior;
        let map = create_test_map();
        let robot = create_test_robot(2, 2, 50);

        let target1 = explorer.get_random_exploration_target(&robot, &map);
        let target2 = explorer.get_random_exploration_target(&robot, &map);

        assert_eq!(target1, target2);

        if let Some((x, y)) = target1 {
            assert!(x < map.width);
            assert!(y < map.height);
            assert_eq!(map.terrain[y][x], 0);
        }
    }
}
