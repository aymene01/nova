use crate::simulation::entities::{Map, ResourceType, Robot, RobotType, Station};
use crate::simulation::robot_ai::behaviors::{
    ExplorerBehavior, HarvesterBehavior, ScientistBehavior,
};
use crate::simulation::robot_ai::types::Task;

pub trait RobotBehavior {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task>;
    fn get_preferred_resources(&self) -> Vec<ResourceType>;
    fn get_energy_consumption_rate(&self) -> u32;
    fn can_perform_task(&self, task: &Task) -> bool;
}

pub fn create_behavior(robot_type: &RobotType) -> Box<dyn RobotBehavior> {
    match robot_type {
        RobotType::Explorer => Box::new(ExplorerBehavior),
        RobotType::Harvester => Box::new(HarvesterBehavior),
        RobotType::Scientist => Box::new(ScientistBehavior),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::robot_ai::types::{
        AnalysisType, AnalyzeTask, ExploreTask, HarvestTask, TaskType,
    };
    use std::collections::HashMap;

    fn create_test_robot(robot_type: RobotType, x: usize, y: usize, energy: u32) -> Robot {
        Robot {
            id: 1,
            robot_type,
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

    fn create_minimal_map() -> Map {
        Map::new_test_map(10, 10)
    }

    #[test]
    fn test_create_explorer_behavior() {
        let behavior = create_behavior(&RobotType::Explorer);
        assert_eq!(behavior.get_energy_consumption_rate(), 2);
        assert_eq!(behavior.get_preferred_resources(), vec![]);
    }

    #[test]
    fn test_create_harvester_behavior() {
        let behavior = create_behavior(&RobotType::Harvester);
        assert_eq!(behavior.get_energy_consumption_rate(), 3);
        assert_eq!(
            behavior.get_preferred_resources(),
            vec![ResourceType::Energy, ResourceType::Mineral]
        );
    }

    #[test]
    fn test_create_scientist_behavior() {
        let behavior = create_behavior(&RobotType::Scientist);
        assert_eq!(behavior.get_energy_consumption_rate(), 4);
        assert_eq!(
            behavior.get_preferred_resources(),
            vec![ResourceType::ScientificInterest]
        );
    }

    #[test]
    fn test_behavior_factory_returns_different_types() {
        let explorer = create_behavior(&RobotType::Explorer);
        let harvester = create_behavior(&RobotType::Harvester);
        let scientist = create_behavior(&RobotType::Scientist);

        assert_ne!(
            explorer.get_energy_consumption_rate(),
            harvester.get_energy_consumption_rate()
        );
        assert_ne!(
            harvester.get_energy_consumption_rate(),
            scientist.get_energy_consumption_rate()
        );
        assert_ne!(
            explorer.get_energy_consumption_rate(),
            scientist.get_energy_consumption_rate()
        );
    }

    #[test]
    fn test_behavior_task_capabilities() {
        let explorer = create_behavior(&RobotType::Explorer);
        let harvester = create_behavior(&RobotType::Harvester);
        let scientist = create_behavior(&RobotType::Scientist);

        let explore_task = Task {
            task_type: TaskType::Explore(ExploreTask {
                target_area: (5, 5),
                radius: 3,
            }),
            target_position: Some((5, 5)),
            priority: 7,
        };

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

        let return_task = Task {
            task_type: TaskType::ReturnToStation,
            target_position: Some((5, 5)),
            priority: 10,
        };

        assert!(explorer.can_perform_task(&explore_task));
        assert!(explorer.can_perform_task(&return_task));
        assert!(!explorer.can_perform_task(&harvest_task));
        assert!(!explorer.can_perform_task(&analyze_task));

        assert!(harvester.can_perform_task(&harvest_task));
        assert!(harvester.can_perform_task(&explore_task));
        assert!(harvester.can_perform_task(&return_task));
        assert!(!harvester.can_perform_task(&analyze_task));

        assert!(scientist.can_perform_task(&analyze_task));
        assert!(scientist.can_perform_task(&explore_task));
        assert!(scientist.can_perform_task(&return_task));
        assert!(!scientist.can_perform_task(&harvest_task));
    }

    #[test]
    fn test_low_energy_behavior_decisions() {
        let map = create_minimal_map();
        let station = create_test_station();

        let low_energy_explorer = create_test_robot(RobotType::Explorer, 2, 2, 15);
        let explorer_behavior = create_behavior(&RobotType::Explorer);
        let task = explorer_behavior.decide_next_action(&low_energy_explorer, &map, &station);

        assert!(task.is_some());
        if let Some(task) = task {
            assert_eq!(task.task_type, TaskType::ReturnToStation);
            assert_eq!(task.target_position, Some((station.x, station.y)));
        }

        let low_energy_harvester = create_test_robot(RobotType::Harvester, 2, 2, 10);
        let harvester_behavior = create_behavior(&RobotType::Harvester);
        let task = harvester_behavior.decide_next_action(&low_energy_harvester, &map, &station);

        assert!(task.is_some());
        if let Some(task) = task {
            assert_eq!(task.task_type, TaskType::ReturnToStation);
        }

        let low_energy_scientist = create_test_robot(RobotType::Scientist, 2, 2, 20);
        let scientist_behavior = create_behavior(&RobotType::Scientist);
        let task = scientist_behavior.decide_next_action(&low_energy_scientist, &map, &station);

        assert!(task.is_some());
        if let Some(task) = task {
            assert_eq!(task.task_type, TaskType::ReturnToStation);
        }
    }

    #[test]
    fn test_behavior_trait_object_functionality() {
        let behaviors: Vec<Box<dyn RobotBehavior>> = vec![
            create_behavior(&RobotType::Explorer),
            create_behavior(&RobotType::Harvester),
            create_behavior(&RobotType::Scientist),
        ];

        for (i, behavior) in behaviors.iter().enumerate() {
            assert!(behavior.get_energy_consumption_rate() > 0);
            if i == 0 {
                assert!(behavior.get_preferred_resources().is_empty());
            } else {
                assert!(!behavior.get_preferred_resources().is_empty());
            }
        }
    }

    #[test]
    fn test_robot_with_carrying_returns_to_station() {
        let map = create_minimal_map();
        let station = create_test_station();

        let mut carrying_robot = create_test_robot(RobotType::Harvester, 3, 3, 50);
        carrying_robot.carrying = Some((ResourceType::Energy, 25));

        let behavior = create_behavior(&RobotType::Harvester);
        let task = behavior.decide_next_action(&carrying_robot, &map, &station);

        assert!(task.is_some());
        if let Some(task) = task {
            assert_eq!(task.task_type, TaskType::ReturnToStation);
        }
    }

    #[test]
    fn test_healthy_robot_behavior_decisions() {
        let map = create_minimal_map();
        let station = create_test_station();

        let healthy_explorer = create_test_robot(RobotType::Explorer, 1, 1, 50);
        let explorer_behavior = create_behavior(&RobotType::Explorer);
        let task = explorer_behavior.decide_next_action(&healthy_explorer, &map, &station);

        assert!(task.is_some());
        if let Some(task) = task {
            assert_ne!(task.task_type, TaskType::ReturnToStation);
        }
    }
}
