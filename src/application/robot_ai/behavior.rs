use crate::simulation::entities::{Map, Station};
use crate::application::robot_ai::robot::Robot;
use crate::application::robot_ai::types::Task;

pub trait RobotBehavior: Send {
    fn decide_next_action(&self, robot: &Robot, map: &Map, station: &Station) -> Option<Task>;
    fn get_energy_consumption_rate(&self) -> u32;
    fn get_max_energy(&self) -> u32;
    fn get_low_energy_threshold(&self) -> u32;
}

pub fn create_behavior(
    robot_type: &crate::application::robot_ai::types::RobotType,
) -> Box<dyn RobotBehavior> {
    match robot_type {
        crate::application::robot_ai::types::RobotType::Explorer => {
            Box::new(crate::application::robot_ai::behaviors::ExplorerBehavior)
        }
        crate::application::robot_ai::types::RobotType::Harvester => {
            Box::new(crate::application::robot_ai::behaviors::HarvesterBehavior)
        }
        crate::application::robot_ai::types::RobotType::Scientist => {
            Box::new(crate::application::robot_ai::behaviors::ScientistBehavior)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::robot_ai::types::RobotType;

    #[test]
    fn test_create_explorer_behavior() {
        let behavior = create_behavior(&RobotType::Explorer);
        assert_eq!(behavior.get_energy_consumption_rate(), 2);
    }

    #[test]
    fn test_create_harvester_behavior() {
        let behavior = create_behavior(&RobotType::Harvester);
        assert_eq!(behavior.get_energy_consumption_rate(), 3);
    }

    #[test]
    fn test_create_scientist_behavior() {
        let behavior = create_behavior(&RobotType::Scientist);
        assert_eq!(behavior.get_energy_consumption_rate(), 4);
    }
}
