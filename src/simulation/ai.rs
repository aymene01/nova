use crate::simulation::entities::{
    Direction, Map, ResourceType, Robot, RobotState, RobotType, Station, MAX_ROBOT_ENERGY,
};
use crate::simulation::pathfinding::Pathfinder;

/// Action that a robot can perform
#[derive(Debug, Clone, PartialEq)]
pub enum RobotAction {
    Move(Direction),
    Idle,
    CollectResource,
    ReturnToStation,
}

/// Defines the behavior interface for robot types
pub trait RobotBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction;
    fn decide_action_with_station(
        &self,
        robot: &Robot,
        map: &Map,
        station: &Station,
    ) -> RobotAction {
        match robot.state() {
            RobotState::ReturningToStation => {
                // Navigate to station intelligently
                if let Some(direction) =
                    Pathfinder::get_direction_to_goal(robot.position(), station.position(), map)
                {
                    RobotAction::Move(direction)
                } else {
                    RobotAction::Idle
                }
            }
            _ => self.decide_action(robot, map),
        }
    }
    #[allow(dead_code)]
    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool;
}

/// Explorer robot behavior - focuses on mapping unknown areas
#[allow(dead_code)]
pub struct ExplorerBehavior;

impl RobotBehavior for ExplorerBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction {
        match robot.state() {
            RobotState::Idle | RobotState::Exploring => {
                // Check if should return to station (will be handled by decide_action_with_station)
                if robot.should_return_to_station() {
                    return RobotAction::ReturnToStation;
                }
                
                // Find nearest unexplored area
                if let Some(unexplored_pos) = Self::find_nearest_unexplored(robot.position(), map) {
                    if let Some(direction) =
                        Pathfinder::get_direction_to_goal(robot.position(), unexplored_pos, map)
                    {
                        return RobotAction::Move(direction);
                    }
                }
                // Fallback to original behavior
                RobotAction::Move(Direction::North)
            }
            _ => RobotAction::Idle,
        }
    }

    fn decide_action_with_station(
        &self,
        robot: &Robot,
        map: &Map,
        station: &Station,
    ) -> RobotAction {
        match robot.state() {
            RobotState::ReturningToStation => {
                // Navigate to station intelligently
                if let Some(direction) =
                    Pathfinder::get_direction_to_goal(robot.position(), station.position(), map)
                {
                    RobotAction::Move(direction)
                } else {
                    RobotAction::Idle
                }
            }
            RobotState::Idle | RobotState::Exploring => {
                // Check if should return based on energy and station distance
                if !robot.should_continue_mission(station.position()) {
                    return RobotAction::ReturnToStation;
                }
                
                // Continue with normal exploration behavior
                self.decide_action(robot, map)
            }
            _ => RobotAction::Idle,
        }
    }

    #[allow(dead_code)]
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
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction {
        match robot.state() {
            RobotState::Idle => {
                // Check if should return to station
                if robot.should_return_to_station() {
                    return RobotAction::ReturnToStation;
                }
                
                // Look for nearest resource
                if let Some(resource_pos) = Self::find_nearest_resource(robot.position(), map) {
                    if let Some(direction) =
                        Pathfinder::get_direction_to_goal(robot.position(), resource_pos, map)
                    {
                        return RobotAction::Move(direction);
                    }
                }
                // Fallback to original behavior
                RobotAction::Move(Direction::South)
            }
            RobotState::MovingToResource => {
                // Check if we're at a resource position
                if robot.detect_resource_at_position(map).is_some() {
                    RobotAction::CollectResource
                } else {
                    // Continue moving towards nearest resource
                    if let Some(resource_pos) = Self::find_nearest_resource(robot.position(), map) {
                        if let Some(direction) =
                            Pathfinder::get_direction_to_goal(robot.position(), resource_pos, map)
                        {
                            return RobotAction::Move(direction);
                        }
                    }
                    RobotAction::Idle
                }
            }
            RobotState::Harvesting => RobotAction::ReturnToStation,
            _ => RobotAction::Idle,
        }
    }

    fn decide_action_with_station(
        &self,
        robot: &Robot,
        map: &Map,
        station: &Station,
    ) -> RobotAction {
        match robot.state() {
            RobotState::ReturningToStation => {
                // Navigate to station intelligently
                if let Some(direction) =
                    Pathfinder::get_direction_to_goal(robot.position(), station.position(), map)
                {
                    RobotAction::Move(direction)
                } else {
                    RobotAction::Idle
                }
            }
            RobotState::Idle => {
                // Check if should return based on energy and station distance
                if !robot.should_continue_mission(station.position()) {
                    return RobotAction::ReturnToStation;
                }
                
                // Continue with normal harvesting behavior
                self.decide_action(robot, map)
            }
            _ => self.decide_action(robot, map),
        }
    }

    #[allow(dead_code)]
    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool {
        match action {
            RobotAction::Move(_) => robot.energy() >= 10, // MOVE_ENERGY_COST
            RobotAction::CollectResource => robot.carrying.is_none(),
            _ => true,
        }
    }
}

impl HarvesterBehavior {
    /// Find the nearest resource position (Energy or Minerals)
    fn find_nearest_resource(from: (usize, usize), map: &Map) -> Option<(usize, usize)> {
        let mut closest = None;
        let mut min_distance = u32::MAX;

        for (pos, (resource_type, _amount)) in &map.resources {
            // Harvesters care about Energy and Minerals
            if matches!(
                resource_type,
                crate::simulation::entities::ResourceType::Energy
                    | crate::simulation::entities::ResourceType::Mineral
            ) {
                let distance = Pathfinder::manhattan_distance(from, *pos);
                if distance < min_distance {
                    min_distance = distance;
                    closest = Some(*pos);
                }
            }
        }

        closest
    }
}

/// Scientist robot behavior - focuses on investigating scientific interest points
#[allow(dead_code)]
pub struct ScientistBehavior;

impl RobotBehavior for ScientistBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction {
        match robot.state() {
            RobotState::Idle => {
                // Check if should return to station
                if robot.should_return_to_station() {
                    return RobotAction::ReturnToStation;
                }
                
                // Look for nearest scientific interest point
                if let Some(science_pos) = Self::find_nearest_science_point(robot.position(), map) {
                    if let Some(direction) =
                        Pathfinder::get_direction_to_goal(robot.position(), science_pos, map)
                    {
                        return RobotAction::Move(direction);
                    }
                }
                // Fallback to original behavior
                RobotAction::Move(Direction::East)
            }
            RobotState::MovingToResource => {
                // Check if we're at a scientific interest point
                if let Some((ResourceType::ScientificInterest, _)) =
                    robot.detect_resource_at_position(map)
                {
                    RobotAction::CollectResource
                } else {
                    // Continue moving towards nearest scientific interest
                    if let Some(science_pos) =
                        Self::find_nearest_science_point(robot.position(), map)
                    {
                        if let Some(direction) =
                            Pathfinder::get_direction_to_goal(robot.position(), science_pos, map)
                        {
                            return RobotAction::Move(direction);
                        }
                    }
                    RobotAction::Idle
                }
            }
            RobotState::Harvesting => RobotAction::ReturnToStation,
            _ => RobotAction::Idle,
        }
    }

    fn decide_action_with_station(
        &self,
        robot: &Robot,
        map: &Map,
        station: &Station,
    ) -> RobotAction {
        match robot.state() {
            RobotState::ReturningToStation => {
                // Navigate to station intelligently
                if let Some(direction) =
                    Pathfinder::get_direction_to_goal(robot.position(), station.position(), map)
                {
                    RobotAction::Move(direction)
                } else {
                    RobotAction::Idle
                }
            }
            RobotState::Idle => {
                // Check if should return based on energy and station distance
                if !robot.should_continue_mission(station.position()) {
                    return RobotAction::ReturnToStation;
                }
                
                // Continue with normal scientific behavior
                self.decide_action(robot, map)
            }
            _ => self.decide_action(robot, map),
        }
    }

    #[allow(dead_code)]
    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool {
        match action {
            RobotAction::Move(_) => robot.energy() >= 10, // MOVE_ENERGY_COST
            RobotAction::CollectResource => robot.carrying.is_none(),
            _ => true,
        }
    }
}

impl ScientistBehavior {
    /// Find the nearest scientific interest point
    fn find_nearest_science_point(from: (usize, usize), map: &Map) -> Option<(usize, usize)> {
        let mut closest = None;
        let mut min_distance = u32::MAX;

        for (pos, (resource_type, _amount)) in &map.resources {
            // Scientists care about Scientific Interest Points
            if matches!(
                resource_type,
                crate::simulation::entities::ResourceType::ScientificInterest
            ) {
                let distance = Pathfinder::manhattan_distance(from, *pos);
                if distance < min_distance {
                    min_distance = distance;
                    closest = Some(*pos);
                }
            }
        }

        closest
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

    /// Execute action with station-aware behavior
    pub fn execute_action_with_station(
        &self,
        robot: &mut Robot,
        map: &Map,
        station: &mut Station,
    ) -> Result<(), String> {
        let behavior = self.get_behavior(&robot.robot_type());
        let action = behavior.decide_action_with_station(robot, map, station);

        // Handle resource delivery and recharging if robot is at station
        if station.robot_at_station(robot.position()) {
            match robot.state() {
                RobotState::ReturningToStation => {
                    // First deliver any resources
                    if robot.carrying.is_some() {
                        robot
                            .deliver_resource(station)
                            .map_err(|e| format!("Delivery failed: {}", e))?;
                    }
                    
                    // Then recharge if needed and possible
                    if robot.energy() < MAX_ROBOT_ENERGY && station.can_recharge() {
                        match station.recharge_robot(robot) {
                            Ok(recharged) => {
                                log::info!("Robot {} recharged {} energy at station", robot.id, recharged);
                            }
                            Err(_) => {
                                // Recharging failed, but that's ok
                            }
                        }
                    }
                    
                    // Robot is now ready for next mission
                    robot.set_state(RobotState::Idle);
                    return Ok(());
                }
                _ => {
                    // Robot at station but not in returning state, might want to recharge
                    if robot.energy() < MAX_ROBOT_ENERGY && station.can_recharge() {
                        let _ = station.recharge_robot(robot);
                    }
                }
            }
        }

        // Execute the decided action
        self.execute_action(robot, action)
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
    use crate::simulation::entities::{Map, ResourceType, Robot, RobotType};

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
        let mut map = Map::new_test_map(10, 10);
        // Place a resource at the robot's position
        map.resources.insert((5, 5), (ResourceType::Energy, 50));

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

    #[test]
    fn harvester_finds_nearest_mineral() {
        let mut map = Map::new_test_map(10, 10);
        // Place minerals at different distances
        map.resources.insert((8, 8), (ResourceType::Mineral, 50));
        map.resources.insert((3, 3), (ResourceType::Mineral, 30));

        let robot = Robot::new(1, RobotType::Harvester, 1, 1, 100);

        // Should find the closer mineral at (3, 3)
        let closest = HarvesterBehavior::find_nearest_resource(robot.position(), &map);
        assert_eq!(closest, Some((3, 3)));
    }

    #[test]
    fn harvester_moves_towards_resource() {
        let mut map = Map::new_test_map(10, 10);
        map.resources.insert((5, 5), (ResourceType::Energy, 50));

        let robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);
        let behavior = HarvesterBehavior;

        let action = behavior.decide_action(&robot, &map);
        // Should attempt to move towards the resource
        assert!(matches!(action, RobotAction::Move(_)));
    }

    #[test]
    fn harvester_ignores_scientific_interest() {
        let mut map = Map::new_test_map(10, 10);
        map.resources
            .insert((3, 3), (ResourceType::ScientificInterest, 100));
        map.resources.insert((8, 8), (ResourceType::Energy, 20));

        let robot = Robot::new(1, RobotType::Harvester, 1, 1, 100);

        // Should find energy resource, not scientific interest
        let closest = HarvesterBehavior::find_nearest_resource(robot.position(), &map);
        assert_eq!(closest, Some((8, 8)));
    }

    #[test]
    fn harvester_collects_when_at_resource() {
        let mut map = Map::new_test_map(10, 10);
        map.resources.insert((5, 5), (ResourceType::Mineral, 50));

        let mut robot = Robot::new(1, RobotType::Harvester, 5, 5, 100);
        robot.set_state(RobotState::MovingToResource);
        let behavior = HarvesterBehavior;

        let action = behavior.decide_action(&robot, &map);
        assert_eq!(action, RobotAction::CollectResource);
    }

    #[test]
    fn scientist_finds_nearest_interest_point() {
        let mut map = Map::new_test_map(10, 10);
        // Place science points at different distances
        map.resources
            .insert((8, 8), (ResourceType::ScientificInterest, 100));
        map.resources
            .insert((3, 3), (ResourceType::ScientificInterest, 80));

        let robot = Robot::new(1, RobotType::Scientist, 1, 1, 100);

        // Should find the closer science point at (3, 3)
        let closest = ScientistBehavior::find_nearest_science_point(robot.position(), &map);
        assert_eq!(closest, Some((3, 3)));
    }

    #[test]
    fn scientist_moves_towards_science_point() {
        let mut map = Map::new_test_map(10, 10);
        map.resources
            .insert((5, 5), (ResourceType::ScientificInterest, 100));

        let robot = Robot::new(1, RobotType::Scientist, 2, 2, 100);
        let behavior = ScientistBehavior;

        let action = behavior.decide_action(&robot, &map);
        // Should attempt to move towards the science point
        assert!(matches!(action, RobotAction::Move(_)));
    }

    #[test]
    fn scientist_ignores_energy_and_minerals() {
        let mut map = Map::new_test_map(10, 10);
        map.resources.insert((3, 3), (ResourceType::Energy, 50));
        map.resources.insert((4, 4), (ResourceType::Mineral, 30));
        map.resources
            .insert((8, 8), (ResourceType::ScientificInterest, 100));

        let robot = Robot::new(1, RobotType::Scientist, 1, 1, 100);

        // Should find scientific interest, not energy or minerals
        let closest = ScientistBehavior::find_nearest_science_point(robot.position(), &map);
        assert_eq!(closest, Some((8, 8)));
    }

    #[test]
    fn scientist_collects_when_at_science_point() {
        let mut map = Map::new_test_map(10, 10);
        map.resources
            .insert((5, 5), (ResourceType::ScientificInterest, 100));

        let mut robot = Robot::new(1, RobotType::Scientist, 5, 5, 100);
        robot.set_state(RobotState::MovingToResource);
        let behavior = ScientistBehavior;

        let action = behavior.decide_action(&robot, &map);
        assert_eq!(action, RobotAction::CollectResource);
    }

    #[test]
    fn robot_navigates_to_station_when_returning() {
        let map = Map::new_test_map(10, 10);
        let station = Station::new(8, 8);

        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);
        robot.set_state(RobotState::ReturningToStation);
        robot.carrying = Some((ResourceType::Energy, 50));

        let behavior = HarvesterBehavior;
        let action = behavior.decide_action_with_station(&robot, &map, &station);

        // Should attempt to move towards station
        assert!(matches!(action, RobotAction::Move(_)));
    }

    #[test]
    fn robot_executor_handles_station_delivery() {
        let map = Map::new_test_map(10, 10);
        let mut station = Station::new(5, 5);

        let mut robot = Robot::new(1, RobotType::Harvester, 5, 5, 100);
        robot.set_state(RobotState::ReturningToStation);
        robot.carrying = Some((ResourceType::Mineral, 30));

        let executor = RobotExecutor::new();
        let result = executor.execute_action_with_station(&mut robot, &map, &mut station);

        assert!(result.is_ok());
        assert!(robot.carrying.is_none());
        assert_eq!(robot.state(), RobotState::Idle);
        assert_eq!(station.get_resource_amount(&ResourceType::Mineral), 30);
    }

    #[test]
    fn robot_continues_navigation_when_not_at_station() {
        let map = Map::new_test_map(10, 10);
        let mut station = Station::new(8, 8);

        let mut robot = Robot::new(1, RobotType::Explorer, 2, 2, 100);
        robot.set_state(RobotState::ReturningToStation);

        let executor = RobotExecutor::new();
        let result = executor.execute_action_with_station(&mut robot, &map, &mut station);

        assert!(result.is_ok());
        assert_eq!(robot.state(), RobotState::ReturningToStation);
        // Robot should have moved closer to station
        assert_ne!(robot.position(), (2, 2));
    }

    #[test]
    fn explorer_returns_when_energy_low() {
        let map = Map::new_test_map(10, 10);
        let station = Station::new(5, 5);
        
        let robot = Robot::new(1, RobotType::Explorer, 2, 2, 25); // Low energy
        let behavior = ExplorerBehavior;
        
        let action = behavior.decide_action_with_station(&robot, &map, &station);
        assert_eq!(action, RobotAction::ReturnToStation);
    }
    
    #[test]
    fn harvester_returns_when_carrying_resource() {
        let map = Map::new_test_map(10, 10);
        let station = Station::new(5, 5);
        
        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 80);
        robot.carrying = Some((ResourceType::Energy, 50));
        let behavior = HarvesterBehavior;
        
        let action = behavior.decide_action(&robot, &map);
        assert_eq!(action, RobotAction::ReturnToStation);
    }
    
    #[test]
    fn scientist_continues_when_energy_sufficient() {
        let mut map = Map::new_test_map(10, 10);
        map.resources.insert((7, 7), (ResourceType::ScientificInterest, 100));
        let station = Station::new(5, 5);
        
        let robot = Robot::new(1, RobotType::Scientist, 2, 2, 90); // Good energy
        let behavior = ScientistBehavior;
        
        let action = behavior.decide_action_with_station(&robot, &map, &station);
        // Should continue exploring, not return
        assert!(matches!(action, RobotAction::Move(_)));
    }
    
    #[test]
    fn robot_makes_intelligent_energy_decisions() {
        let map = Map::new_test_map(10, 10);
        let station = Station::new(9, 9); // Far station
        
        // Robot with just enough energy to return safely
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 45);
        let behavior = ExplorerBehavior;
        
        let action = behavior.decide_action_with_station(&robot, &map, &station);
        // Should return to station due to energy concerns
        assert_eq!(action, RobotAction::ReturnToStation);
    }
    
    #[test]
    fn robot_continues_mission_when_safe() {
        let map = Map::new_test_map(10, 10);
        let station = Station::new(2, 2); // Close station
        
        // Robot with plenty of energy for a close station
        let robot = Robot::new(1, RobotType::Explorer, 1, 1, 90);
        let behavior = ExplorerBehavior;
        
        let action = behavior.decide_action_with_station(&robot, &map, &station);
        // Should continue exploring
        assert!(matches!(action, RobotAction::Move(_)));
    }

    #[test]
    fn robot_executor_handles_recharging_at_station() {
        let map = Map::new_test_map(10, 10);
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100); // Station has energy
        
        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30); // Low energy robot
        robot.set_state(RobotState::ReturningToStation);
        
        let executor = RobotExecutor::new();
        let result = executor.execute_action_with_station(&mut robot, &map, &mut station);
        
        assert!(result.is_ok());
        assert_eq!(robot.state(), RobotState::Idle); // Ready for next mission
        assert_eq!(robot.energy(), 80); // 30 + 50 (recharged)
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50); // 100 - 50
    }
    
    #[test]
    fn robot_executor_handles_delivery_and_recharging() {
        let map = Map::new_test_map(10, 10);
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);
        
        let mut robot = Robot::new(1, RobotType::Harvester, 5, 5, 40);
        robot.set_state(RobotState::ReturningToStation);
        robot.carrying = Some((ResourceType::Mineral, 25)); // Carrying resource
        
        let executor = RobotExecutor::new();
        let result = executor.execute_action_with_station(&mut robot, &map, &mut station);
        
        assert!(result.is_ok());
        assert_eq!(robot.state(), RobotState::Idle);
        assert!(robot.carrying.is_none()); // Resource delivered
        assert_eq!(robot.energy(), 90); // 40 + 50 (recharged)
        assert_eq!(station.get_resource_amount(&ResourceType::Mineral), 25); // Resource received
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50); // Energy used for recharge
    }
}
