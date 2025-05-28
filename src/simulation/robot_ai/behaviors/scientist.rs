use crate::simulation::entities::{Robot, ResourceType, Map, Station};
use crate::simulation::robot_ai::behavior::RobotBehavior;
use crate::simulation::robot_ai::types::{Task, TaskType, ExploreTask, AnalyzeTask, AnalysisType};
use crate::simulation::robot_ai::utils::SearchUtils;

pub struct ScientistBehavior;

impl RobotBehavior for ScientistBehavior {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task> {
        if robot.carrying.is_some() || robot.energy < 25 {
            return Some(Task {
                task_type: TaskType::ReturnToStation,
                target_position: Some((station.x, station.y)),
                priority: 9,
            });
        }

        if let Some(poi_pos) = self.find_scientific_interest(robot, map) {
            return Some(Task {
                task_type: TaskType::Analyze(AnalyzeTask {
                    target_position: poi_pos,
                    analysis_type: self.determine_analysis_type(poi_pos, map),
                }),
                target_position: Some(poi_pos),
                priority: 8,
            });
        }

        if let Some(exploration_target) = self.get_scientific_exploration_target(robot, map) {
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
        vec![ResourceType::ScientificInterest]
    }

    fn get_energy_consumption_rate(&self) -> u32 {
        4
    }

    fn can_perform_task(&self, task: &Task) -> bool {
        matches!(task.task_type, TaskType::Analyze(_) | TaskType::Explore(_) | TaskType::ReturnToStation)
    }
}

impl ScientistBehavior {
    fn find_scientific_interest(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        SearchUtils::find_nearest_scientific_interest(robot.x, robot.y, 6, map)
    }

    fn determine_analysis_type(&self, pos: (usize, usize), map: &Map) -> AnalysisType {
        let terrain_value = map.terrain[pos.1][pos.0];
        let nearby_resources = self.count_nearby_resources(pos, map);
        
        match (terrain_value, nearby_resources) {
            (0, count) if count > 2 => AnalysisType::Chemical,
            _ => AnalysisType::Chemical,
        }
    }

    fn count_nearby_resources(&self, pos: (usize, usize), map: &Map) -> usize {
        let search_radius = 2;
        let mut count = 0;
        let pos_x = pos.0 as i32;
        let pos_y = pos.1 as i32;

        for radius in 1..=search_radius {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    let x = pos_x + dx;
                    let y = pos_y + dy;
                    
                    if x >= 0 && y >= 0 && (x as usize) < map.width && (y as usize) < map.height {
                        let x = x as usize;
                        let y = y as usize;
                        
                        if map.resources.contains_key(&(x, y)) {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }

    fn get_scientific_exploration_target(&self, robot: &Robot, map: &Map) -> Option<(usize, usize)> {
        let pattern_x = (robot.x + 3) % map.width;
        let pattern_y = (robot.y + 2) % map.height;
        
        if map.terrain[pattern_y][pattern_x] == 0 {
            Some((pattern_x, pattern_y))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::{Robot, Station, Map, ResourceType, RobotType};
    use crate::simulation::robot_ai::types::{Task, TaskType, HarvestTask, AnalyzeTask, AnalysisType};
    use std::collections::HashMap;

    fn create_test_robot(x: usize, y: usize, energy: u32) -> Robot {
        Robot {
            id: 1,
            robot_type: RobotType::Scientist,
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

    fn create_test_map_with_scientific_interests() -> Map {
        let mut map = Map::new_test_map(10, 10);
        map.resources.insert((3, 3), (ResourceType::ScientificInterest, 8));
        map.resources.insert((7, 7), (ResourceType::ScientificInterest, 12));
        map.resources.insert((2, 8), (ResourceType::Energy, 5));
        map
    }

    #[test]
    fn test_scientist_behavior_characteristics() {
        let scientist = ScientistBehavior;
        assert_eq!(scientist.get_energy_consumption_rate(), 4);
        assert_eq!(scientist.get_preferred_resources(), vec![ResourceType::ScientificInterest]);
    }

    #[test]
    fn test_scientist_low_energy_returns_to_station() {
        let scientist = ScientistBehavior;
        let map = create_test_map_with_scientific_interests();
        let station = create_test_station();
        let low_energy_robot = create_test_robot(3, 3, 15);

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
        let map = create_test_map_with_scientific_interests();
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
    fn test_scientist_finds_scientific_interests() {
        let scientist = ScientistBehavior;
        let map = create_test_map_with_scientific_interests();
        let station = create_test_station();
        let robot = create_test_robot(1, 1, 50);

        let task = scientist.decide_next_action(&robot, &map, &station);
        
        assert!(task.is_some());
        let task = task.unwrap();
        
        if let TaskType::Analyze(analyze_task) = task.task_type {
            assert_eq!(analyze_task.analysis_type, AnalysisType::Chemical);
            assert!(task.target_position.is_some());
        } else {
            panic!("Expected analyze task, got {:?}", task.task_type);
        }
    }

    #[test]
    fn test_scientist_can_perform_analysis_tasks() {
        let scientist = ScientistBehavior;
        
        let analyze_task = Task {
            task_type: TaskType::Analyze(AnalyzeTask {
                target_position: (3, 3),
                analysis_type: AnalysisType::Chemical,
            }),
            target_position: Some((3, 3)),
            priority: 7,
        };

        let return_task = Task {
            task_type: TaskType::ReturnToStation,
            target_position: Some((5, 5)),
            priority: 9,
        };

        assert!(scientist.can_perform_task(&analyze_task));
        assert!(scientist.can_perform_task(&return_task));
    }

    #[test]
    fn test_scientist_cannot_perform_non_analysis_tasks() {
        let scientist = ScientistBehavior;
        
        let harvest_task = Task {
            task_type: TaskType::Harvest(HarvestTask {
                resource_type: ResourceType::Energy,
                target_position: (3, 3),
            }),
            target_position: Some((3, 3)),
            priority: 8,
        };

        assert!(!scientist.can_perform_task(&harvest_task));
    }

    #[test]
    fn test_find_scientific_interest_method() {
        let scientist = ScientistBehavior;
        let map = create_test_map_with_scientific_interests();
        let robot = create_test_robot(4, 4, 50);

        let result = scientist.find_scientific_interest(&robot, &map);
        
        assert!(result.is_some());
        let pos = result.unwrap();
        
        assert!(map.resources.contains_key(&pos));
        if let Some((resource_type, _)) = map.resources.get(&pos) {
            assert_eq!(*resource_type, ResourceType::ScientificInterest);
        }
        
        assert!(pos.0 < map.width);
        assert!(pos.1 < map.height);
    }

    #[test]
    fn test_determine_analysis_type_consistency() {
        let scientist = ScientistBehavior;
        let position = (3, 3);
        let map = create_test_map_with_scientific_interests();
        
        let analysis1 = scientist.determine_analysis_type(position, &map);
        let analysis2 = scientist.determine_analysis_type(position, &map);
        
        assert_eq!(analysis1, analysis2);
        assert_eq!(analysis1, AnalysisType::Chemical);
    }

    #[test]
    fn test_scientist_with_no_scientific_interests() {
        let scientist = ScientistBehavior;
        let map = Map::new_test_map(10, 10);

        let station = create_test_station();
        let robot = create_test_robot(4, 3, 50);

        let task = scientist.decide_next_action(&robot, &map, &station);
        
        assert!(task.is_some());
        let task = task.unwrap();
        assert!(matches!(task.task_type, TaskType::Explore(_)));
    }

    #[test]
    fn test_scientist_energy_efficiency() {
        let scientist = ScientistBehavior;
        
        assert_eq!(scientist.get_energy_consumption_rate(), 4);
        assert!(scientist.get_energy_consumption_rate() > 2);
    }

    #[test]
    fn test_scientist_resource_specialization() {
        let scientist = ScientistBehavior;
        let preferred = scientist.get_preferred_resources();
        
        assert_eq!(preferred.len(), 1);
        assert!(preferred.contains(&ResourceType::ScientificInterest));
        assert!(!preferred.contains(&ResourceType::Energy));
        assert!(!preferred.contains(&ResourceType::Mineral));
    }

    #[test]
    fn test_scientist_healthy_robot_analyzes() {
        let scientist = ScientistBehavior;
        let map = create_test_map_with_scientific_interests();
        let station = create_test_station();
        let healthy_robot = create_test_robot(1, 1, 50);

        let task = scientist.decide_next_action(&healthy_robot, &map, &station);
        
        assert!(task.is_some());
        let task = task.unwrap();
        
        if !map.resources.is_empty() {
            assert!(matches!(task.task_type, TaskType::Analyze(_)));
        }
    }

    #[test]
    fn test_analysis_type_distribution() {
        let scientist = ScientistBehavior;
        let map = create_test_map_with_scientific_interests();
        
        let positions = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)];
        
        for pos in positions.iter() {
            let analysis_type = scientist.determine_analysis_type(*pos, &map);
            assert_eq!(analysis_type, AnalysisType::Chemical);
        }
    }
} 