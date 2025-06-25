use crate::simulation::entities::{Map, Station};
use crate::application::robot_ai::behavior::RobotBehavior;
use crate::application::robot_ai::robot::Robot;
use crate::application::robot_ai::types::{AnalysisType, AnalyzeTask, Task, TaskType};
use crate::application::robot_ai::utils::SearchUtils;

pub struct ScientistBehavior;

impl RobotBehavior for ScientistBehavior {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task> {
        if robot.carrying.is_some() || robot.is_low_energy() {
            return Some(Task {
                task_type: TaskType::ReturnToStation,
                target_position: Some((station.x, station.y)),
                priority: 9,
            });
        }

        if let Some(scientific_position) = self.find_scientific_interest(robot, map) {
            return Some(Task {
                task_type: TaskType::Analyze(AnalyzeTask {
                    target_position: scientific_position,
                    analysis_type: AnalysisType::Chemical,
                }),
                target_position: Some(scientific_position),
                priority: 8,
            });
        }

        if let Some(exploration_target) = self.find_systematic_exploration_target(robot, map) {
            return Some(Task {
                task_type: TaskType::Explore(crate::application::robot_ai::types::ExploreTask {
                    target_area: exploration_target,
                    radius: 3,
                }),
                target_position: Some(exploration_target),
                priority: 6,
            });
        }

        None
    }

    fn get_energy_consumption_rate(&self) -> u32 {
        4
    }

    fn get_max_energy(&self) -> u32 {
        100
    }

    fn get_low_energy_threshold(&self) -> u32 {
        25
    }
}

impl ScientistBehavior {
    fn find_scientific_interest(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        SearchUtils::find_nearest_scientific_interest(robot.x, robot.y, 6, map)
    }

    fn find_systematic_exploration_target(
        &self,
        robot: &Robot,
        map: &Map,
    ) -> Option<(usize, usize)> {
        SearchUtils::find_nearest_unexplored(robot.x, robot.y, 6, map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::{Map, ResourceType, Station};
    use crate::application::robot_ai::types::{RobotState, RobotType, TaskType};
    use std::collections::HashMap;

    fn create_test_robot(x: usize, y: usize, energy: u32) -> Robot {
        Robot {
            id: 1,
            robot_type: RobotType::Scientist,
            x,
            y,
            energy,
            carrying: None,
            state: RobotState::Idle,
            behavior: Box::new(ScientistBehavior),
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
    fn test_scientist_behavior_characteristics() {
        let scientist = ScientistBehavior;
        assert_eq!(scientist.get_energy_consumption_rate(), 4);
    }

    #[test]
    fn test_scientist_low_energy_returns_to_station() {
        let scientist = ScientistBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let low_energy_robot = create_test_robot(3, 3, 20);

        let task = scientist.decide_next_action(&low_energy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
        assert_eq!(task.priority, 9);
    }

    #[test]
    fn test_scientist_carrying_data_returns_to_station() {
        let scientist = ScientistBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let mut carrying_robot = create_test_robot(3, 3, 50);
        carrying_robot.carrying = Some((ResourceType::ScientificInterest, 5));

        let task = scientist.decide_next_action(&carrying_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::ReturnToStation);
        assert_eq!(task.target_position, Some((station.x, station.y)));
    }

    #[test]
    fn test_scientist_healthy_robot_analyzes() {
        let scientist = ScientistBehavior;
        let map = create_test_map();
        let station = create_test_station();
        let healthy_robot = create_test_robot(3, 3, 80);

        let task = scientist.decide_next_action(&healthy_robot, &map, &station);

        assert!(task.is_some());
        let task = task.unwrap();
        assert_ne!(task.task_type, TaskType::ReturnToStation);
    }

    #[test]
    fn test_scientist_finds_scientific_interest() {
        let scientist = ScientistBehavior;
        let map = create_test_map();
        let robot = create_test_robot(3, 3, 50);

        let result = scientist.find_scientific_interest(&robot, &map);
        assert!(result.is_some());
    }

    #[test]
    fn test_scientist_finds_exploration_target() {
        let scientist = ScientistBehavior;
        let map = create_test_map();
        let robot = create_test_robot(3, 3, 50);

        let result = scientist.find_systematic_exploration_target(&robot, &map);
        assert!(result.is_some());
    }
}
