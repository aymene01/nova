use crate::simulation::entities::{Direction, Robot, RobotState, RobotType, Map};
use crate::simulation::pathfinding::Pathfinder;

/// Action that a robot can perform
#[derive(Debug, Clone, PartialEq)]
pub enum RobotAction {
    Move(Direction),
    Idle,
    CollectResource,
    ReturnToStation,
}

/// Trait for robot AI behavior
#[allow(dead_code)]
pub trait RobotBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction;
    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool;
}

/// Explorer robot behavior - focuses on mapping unknown areas
#[allow(dead_code)]
pub struct ExplorerBehavior;

impl RobotBehavior for ExplorerBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction {
        match robot.state() {
            RobotState::Idle | RobotState::Exploring => {
                // Find nearest unexplored area
                if let Some(unexplored_pos) = Self::find_nearest_unexplored(robot.position(), map) {
                    if let Some(direction) = Pathfinder::get_direction_to_goal(
                        robot.position(),
                        unexplored_pos,
                        map,
                    ) {
                        return RobotAction::Move(direction);
                    }
                }
                // Fallback to original behavior if no pathfinding possible
                RobotAction::Move(Direction::North)
            }
            _ => RobotAction::Idle,
        }
    }

    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool {
        match action {
            RobotAction::Move(_) => robot.energy() >= 10, // MOVE_ENERGY_COST
            _ => true,
        }
    }
}

impl ExplorerBehavior {
    /// Find the nearest unexplored position
    fn find_nearest_unexplored(from: (usize, usize), map: &Map) -> Option<(usize, usize)> {
        let mut closest = None;
        let mut min_distance = u32::MAX;

        // Simple search in a spiral pattern around the robot
        for radius in 1..=5 {
            for dx in 0..=radius {
                for dy in 0..=radius {
                    let positions = vec![
                        (from.0.saturating_add(dx), from.1.saturating_add(dy)),
                        (from.0.saturating_sub(dx), from.1.saturating_add(dy)),
                        (from.0.saturating_add(dx), from.1.saturating_sub(dy)),
                        (from.0.saturating_sub(dx), from.1.saturating_sub(dy)),
                    ];

                    for pos in positions {
                        if pos.0 < map.width && pos.1 < map.height {
                            if let Ok(false) = map.is_discovered(pos.0, pos.1) {
                                let distance = Pathfinder::manhattan_distance(from, pos);
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest = Some(pos);
                                }
                            }
                        }
                    }
                }
            }
            if closest.is_some() {
                break;
            }
        }

        closest
    }
}

/// Harvester robot behavior - focuses on collecting energy and minerals
#[allow(dead_code)]
pub struct HarvesterBehavior;

impl RobotBehavior for HarvesterBehavior {
    fn decide_action(&self, robot: &Robot, _map: &Map) -> RobotAction {
        match robot.state() {
            RobotState::Idle => RobotAction::Move(Direction::South),
            RobotState::MovingToResource => RobotAction::CollectResource,
            RobotState::Harvesting => RobotAction::ReturnToStation,
            _ => RobotAction::Idle,
        }
    }

    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool {
        match action {
            RobotAction::Move(_) => robot.energy() >= 10, // MOVE_ENERGY_COST
            RobotAction::CollectResource => robot.carrying.is_none(),
            _ => true,
        }
    }
}

/// Scientist robot behavior - focuses on investigating scientific interest points
#[allow(dead_code)]
pub struct ScientistBehavior;

impl RobotBehavior for ScientistBehavior {
    fn decide_action(&self, robot: &Robot, _map: &Map) -> RobotAction {
        match robot.state() {
            RobotState::Idle => RobotAction::Move(Direction::East),
            RobotState::Exploring => RobotAction::Move(Direction::West),
            RobotState::MovingToResource => RobotAction::CollectResource,
            _ => RobotAction::ReturnToStation,
        }
    }

    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool {
        match action {
            RobotAction::Move(_) => robot.energy() >= 10, // MOVE_ENERGY_COST
            RobotAction::CollectResource => robot.carrying.is_none(),
            _ => true,
        }
    }
}

/// Robot executor that applies AI decisions to robots
#[allow(dead_code)]
pub struct RobotExecutor;

#[allow(dead_code)]
impl RobotExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute an action on a robot, returning result
    pub fn execute_action(&self, robot: &mut Robot, action: RobotAction) -> Result<(), String> {
        match action {
            RobotAction::Move(direction) => robot
                .move_in_direction(direction)
                .map_err(|e| format!("Movement failed: {}", e)),
            RobotAction::Idle => {
                robot.set_state(RobotState::Idle);
                Ok(())
            }
            RobotAction::CollectResource => {
                robot.set_state(RobotState::Harvesting);
                Ok(())
            }
            RobotAction::ReturnToStation => {
                robot.set_state(RobotState::ReturningToStation);
                Ok(())
            }
        }
    }

    /// Get the appropriate behavior for a robot type
    pub fn get_behavior(&self, robot_type: &RobotType) -> Box<dyn RobotBehavior> {
        match robot_type {
            RobotType::Explorer => Box::new(ExplorerBehavior),
            RobotType::Harvester => Box::new(HarvesterBehavior),
            RobotType::Scientist => Box::new(ScientistBehavior),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::Robot;

    #[test]
    fn explorer_decides_to_move_when_idle() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 5, 100);
        let behavior = ExplorerBehavior;
        let map = Map::new_test_map(10, 10);

        let action = behavior.decide_action(&robot, &map);

        assert_eq!(action, RobotAction::Move(Direction::North));
    }

    #[test]
    fn explorer_cannot_move_with_insufficient_energy() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 5, 5);
        let behavior = ExplorerBehavior;
        let action = RobotAction::Move(Direction::North);

        let can_execute = behavior.can_execute(&robot, &action);

        assert!(!can_execute);
    }

    #[test]
    fn harvester_decides_to_move_when_idle() {
        let robot = Robot::new(2, RobotType::Harvester, 5, 5, 100);
        let behavior = HarvesterBehavior;
        let map = Map::new_test_map(10, 10);

        let action = behavior.decide_action(&robot, &map);

        assert_eq!(action, RobotAction::Move(Direction::South));
    }

    #[test]
    fn harvester_decides_to_collect_when_at_resource() {
        let mut robot = Robot::new(2, RobotType::Harvester, 5, 5, 100);
        robot.set_state(RobotState::MovingToResource);
        let behavior = HarvesterBehavior;
        let map = Map::new_test_map(10, 10);

        let action = behavior.decide_action(&robot, &map);

        assert_eq!(action, RobotAction::CollectResource);
    }

    #[test]
    fn scientist_decides_to_move_when_idle() {
        let robot = Robot::new(3, RobotType::Scientist, 5, 5, 100);
        let behavior = ScientistBehavior;
        let map = Map::new_test_map(10, 10);

        let action = behavior.decide_action(&robot, &map);

        assert_eq!(action, RobotAction::Move(Direction::East));
    }

    #[test]
    fn robot_executor_can_execute_move_action() {
        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 100);
        let executor = RobotExecutor::new();
        let action = RobotAction::Move(Direction::North);

        let result = executor.execute_action(&mut robot, action);

        assert!(result.is_ok());
        assert_eq!(robot.position(), (5, 4));
    }

    #[test]
    fn robot_executor_gets_correct_behavior() {
        let executor = RobotExecutor::new();

        let explorer_behavior = executor.get_behavior(&RobotType::Explorer);
        let harvester_behavior = executor.get_behavior(&RobotType::Harvester);
        let scientist_behavior = executor.get_behavior(&RobotType::Scientist);

        // Test that each behavior type works correctly
        let explorer_robot = Robot::new(1, RobotType::Explorer, 0, 0, 100);
        let harvester_robot = Robot::new(2, RobotType::Harvester, 0, 0, 100);
        let scientist_robot = Robot::new(3, RobotType::Scientist, 0, 0, 100);
        let map = Map::new_test_map(10, 10);

        assert_eq!(
            explorer_behavior.decide_action(&explorer_robot, &map),
            RobotAction::Move(Direction::North)
        );
        assert_eq!(
            harvester_behavior.decide_action(&harvester_robot, &map),
            RobotAction::Move(Direction::South)
        );
        assert_eq!(
            scientist_behavior.decide_action(&scientist_robot, &map),
            RobotAction::Move(Direction::East)
        );
    }
}
