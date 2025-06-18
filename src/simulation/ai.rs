use crate::simulation::entities::{Direction, Robot, RobotState, RobotType};

/// Action that a robot can perform
#[derive(Debug, Clone, PartialEq)]
pub enum RobotAction {
    Move(Direction),
    Idle,
    CollectResource,
    ReturnToStation,
}

/// Trait for robot AI behavior
pub trait RobotBehavior {
    fn decide_action(&self, robot: &Robot) -> RobotAction;
    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool;
}

/// Explorer robot behavior - focuses on mapping unknown areas
pub struct ExplorerBehavior;

impl RobotBehavior for ExplorerBehavior {
    fn decide_action(&self, robot: &Robot) -> RobotAction {
        match robot.state() {
            RobotState::Idle => RobotAction::Move(Direction::North),
            RobotState::Exploring => RobotAction::Move(Direction::East),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::entities::Robot;

    #[test]
    fn explorer_decides_to_move_when_idle() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 5, 100);
        let behavior = ExplorerBehavior;
        
        let action = behavior.decide_action(&robot);
        
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
} 