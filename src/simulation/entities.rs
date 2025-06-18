use noise::Perlin;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Movement directions for robots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Movement constants
pub const MOVE_ENERGY_COST: u32 = 10;
#[allow(dead_code)]
pub const HARVEST_ENERGY_COST: u32 = 5;
#[allow(dead_code)]
pub const STARTING_ENERGY: u32 = 100;
pub const LOW_ENERGY_THRESHOLD: u32 = 20;
pub const RETURN_TO_STATION_THRESHOLD: u32 = 30; // When to head back to station
pub const STATION_RECHARGE_RATE: u32 = 50; // Energy recharged per station visit
pub const MAX_ROBOT_ENERGY: u32 = 100; // Maximum robot energy capacity

/// Robot states for behavior management
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RobotState {
    Idle,
    Exploring,
    MovingToResource,
    Harvesting,
    ReturningToStation,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Energy,
    Mineral,
    ScientificInterest,
}

#[allow(dead_code)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<Vec<u8>>,
    pub resources: HashMap<(usize, usize), (ResourceType, u32)>,
    pub discovered: Vec<Vec<bool>>,
    pub noise: Perlin,
    pub seed: u64,
}

// Custom serialization for Map to handle tuple keys
impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert resources to a format with string keys
        let resources_serializable: HashMap<String, (ResourceType, u32)> = self
            .resources
            .iter()
            .map(|((x, y), value)| (format!("{},{}", x, y), value.clone()))
            .collect();

        // Create a struct with the expected number of fields (note: skipping noise field)
        let mut map_struct = serializer.serialize_struct("Map", 6)?;
        map_struct.serialize_field("width", &self.width)?;
        map_struct.serialize_field("height", &self.height)?;
        map_struct.serialize_field("terrain", &self.terrain)?;
        map_struct.serialize_field("resources", &resources_serializable)?;
        map_struct.serialize_field("discovered", &self.discovered)?;
        map_struct.serialize_field("seed", &self.seed)?;
        map_struct.end()
    }
}

// Custom deserialization for Map to handle string keys back to tuple keys
impl<'de> Deserialize<'de> for Map {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MapHelper {
            width: usize,
            height: usize,
            terrain: Vec<Vec<u8>>,
            resources: HashMap<String, (ResourceType, u32)>,
            discovered: Vec<Vec<bool>>,
            seed: u64,
        }

        let helper = MapHelper::deserialize(deserializer)?;

        // Convert string keys back to tuple keys
        let resources = helper
            .resources
            .into_iter()
            .map(|(key, value)| {
                let coords: Vec<&str> = key.split(',').collect();
                if coords.len() != 2 {
                    return Err(serde::de::Error::custom("Invalid coordinate format"));
                }

                let x = coords[0]
                    .parse::<usize>()
                    .map_err(|_| serde::de::Error::custom("Invalid x coordinate"))?;
                let y = coords[1]
                    .parse::<usize>()
                    .map_err(|_| serde::de::Error::custom("Invalid y coordinate"))?;

                Ok(((x, y), value))
            })
            .collect::<Result<HashMap<(usize, usize), (ResourceType, u32)>, D::Error>>()?;

        Ok(Map {
            width: helper.width,
            height: helper.height,
            terrain: helper.terrain,
            resources,
            discovered: helper.discovered,
            noise: Perlin::new(helper.seed as u32),
            seed: helper.seed,
        })
    }
}

impl Map {
    /// Creates a new Map for testing purposes
    #[allow(dead_code)]
    pub fn new_test_map(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            terrain: vec![vec![0; width]; height],
            resources: HashMap::new(),
            discovered: vec![vec![false; width]; height],
            noise: Perlin::new(42),
            seed: 42,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,
    Harvester,
    Scientist,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Robot {
    pub id: usize,
    pub robot_type: RobotType,
    pub x: usize,
    pub y: usize,
    pub energy: u32,
    pub carrying: Option<(ResourceType, u32)>,
    pub state: RobotState,
}

#[allow(dead_code)]
impl Robot {
    pub fn new(id: usize, robot_type: RobotType, x: usize, y: usize, energy: u32) -> Self {
        Self {
            id,
            robot_type,
            x,
            y,
            energy,
            carrying: None,
            state: RobotState::Idle,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn energy(&self) -> u32 {
        self.energy
    }

    pub fn robot_type(&self) -> RobotType {
        self.robot_type.clone()
    }

    pub fn state(&self) -> RobotState {
        self.state.clone()
    }

    pub fn set_state(&mut self, new_state: RobotState) {
        self.state = new_state;
    }

    pub fn is_low_energy(&self) -> bool {
        self.energy <= LOW_ENERGY_THRESHOLD
    }

    pub fn consume_energy(&mut self, amount: u32) -> Result<(), &'static str> {
        if self.energy >= amount {
            self.energy -= amount;
            Ok(())
        } else {
            Err("Insufficient energy")
        }
    }

    pub fn recharge(&mut self, amount: u32) {
        self.energy = self.energy.saturating_add(amount);
    }

    pub fn move_in_direction(&mut self, direction: Direction) -> Result<(), &'static str> {
        if self.energy < MOVE_ENERGY_COST {
            return Err("Insufficient energy");
        }

        let (new_x, new_y) = match direction {
            Direction::North if self.y > 0 => (self.x, self.y - 1),
            Direction::South => (self.x, self.y + 1),
            Direction::East => (self.x + 1, self.y),
            Direction::West if self.x > 0 => (self.x - 1, self.y),
            _ => return Err("Invalid move: out of bounds"),
        };

        self.x = new_x;
        self.y = new_y;
        self.energy -= MOVE_ENERGY_COST;
        Ok(())
    }

    pub fn detect_resource_at_position(&self, map: &Map) -> Option<(ResourceType, u32)> {
        map.resources.get(&(self.x, self.y)).cloned()
    }

    pub fn collect_resource(&mut self, map: &mut Map) -> Result<(), &'static str> {
        if self.carrying.is_some() {
            return Err("Already carrying a resource");
        }

        if let Some(resource) = map.resources.remove(&(self.x, self.y)) {
            self.carrying = Some(resource);
            Ok(())
        } else {
            Err("No resource found at position")
        }
    }

    pub fn deliver_resource(&mut self, station: &mut Station) -> Result<(), &'static str> {
        if let Some((resource_type, amount)) = self.carrying.take() {
            station.receive_resource(resource_type, amount);
            self.set_state(RobotState::Idle);
            Ok(())
        } else {
            Err("No resource to deliver")
        }
    }

    /// Determine if robot should return to station based on energy and carrying status
    pub fn should_return_to_station(&self) -> bool {
        // Return if carrying something
        if self.carrying.is_some() {
            return true;
        }

        // Return if energy is below threshold
        if self.energy <= RETURN_TO_STATION_THRESHOLD {
            return true;
        }

        false
    }

    /// Calculate estimated energy needed to return to station
    pub fn energy_to_return(&self, station_position: (usize, usize)) -> u32 {
        let distance = self.manhattan_distance_to(station_position);
        distance * MOVE_ENERGY_COST
    }

    /// Check if robot has enough energy to return to station
    pub fn can_return_to_station(&self, station_position: (usize, usize)) -> bool {
        let energy_needed = self.energy_to_return(station_position);
        self.energy >= energy_needed
    }

    /// Calculate Manhattan distance to a position
    pub fn manhattan_distance_to(&self, target: (usize, usize)) -> u32 {
        let dx = if self.x > target.0 {
            self.x - target.0
        } else {
            target.0 - self.x
        };
        let dy = if self.y > target.1 {
            self.y - target.1
        } else {
            target.1 - self.y
        };
        (dx + dy) as u32
    }

    /// Smart decision about whether to continue exploring or return
    pub fn should_continue_mission(&self, station_position: (usize, usize)) -> bool {
        // If already carrying something, should return
        if self.carrying.is_some() {
            return false;
        }

        // If can't safely return, must return now
        if !self.can_return_to_station(station_position) {
            return false;
        }

        // If energy is getting low but still safe, consider returning
        let energy_to_return = self.energy_to_return(station_position);
        let safety_margin = 20; // Extra energy buffer

        self.energy > energy_to_return + safety_margin
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
}

impl Station {
    #[allow(dead_code)]
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            resources: HashMap::new(),
            discoveries: 0,
            x,
            y,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    /// Accept a resource delivery from a robot
    pub fn receive_resource(&mut self, resource_type: ResourceType, amount: u32) {
        // Track discoveries for scientific interest before moving resource_type
        if resource_type == ResourceType::ScientificInterest {
            self.discoveries += 1;
        }

        *self.resources.entry(resource_type).or_insert(0) += amount;
    }

    /// Get the current amount of a specific resource
    #[allow(dead_code)]
    pub fn get_resource_amount(&self, resource_type: &ResourceType) -> u32 {
        self.resources.get(resource_type).copied().unwrap_or(0)
    }

    /// Get total resource count across all types
    #[allow(dead_code)]
    pub fn total_resources(&self) -> u32 {
        self.resources.values().sum()
    }

    /// Check if robot is at station position
    pub fn robot_at_station(&self, robot_position: (usize, usize)) -> bool {
        self.position() == robot_position
    }

    /// Recharge a robot's energy if station has energy resources
    pub fn recharge_robot(&mut self, robot: &mut Robot) -> Result<u32, &'static str> {
        let energy_available = self.get_resource_amount(&ResourceType::Energy);

        if energy_available == 0 {
            return Err("No energy available at station");
        }

        let current_energy = robot.energy();
        if current_energy >= MAX_ROBOT_ENERGY {
            return Err("Robot already at full energy");
        }

        let energy_needed = MAX_ROBOT_ENERGY - current_energy;
        let recharge_amount = energy_needed
            .min(STATION_RECHARGE_RATE)
            .min(energy_available);

        // Use station's energy to recharge robot
        *self.resources.entry(ResourceType::Energy).or_insert(0) -= recharge_amount;
        robot.recharge(recharge_amount);

        Ok(recharge_amount)
    }

    /// Check if station can recharge robots (has energy)
    pub fn can_recharge(&self) -> bool {
        self.get_resource_amount(&ResourceType::Energy) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robot_creation_works() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 10, 100);

        assert_eq!(robot.id, 1);
        assert_eq!(robot.position(), (5, 10));
        assert_eq!(robot.energy(), 100);
        assert_eq!(robot.robot_type(), RobotType::Explorer);
        assert!(robot.carrying.is_none());
    }

    #[test]
    fn robot_move_north_with_sufficient_energy() {
        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 50);

        let result = robot.move_in_direction(Direction::North);

        assert!(result.is_ok());
        assert_eq!(robot.position(), (5, 4)); // North reduces Y
        assert_eq!(robot.energy(), 50 - MOVE_ENERGY_COST);
    }

    #[test]
    fn robot_move_fails_with_insufficient_energy() {
        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 5); // Only 5 energy

        let result = robot.move_in_direction(Direction::North);

        assert!(result.is_err());
        assert_eq!(robot.position(), (5, 5)); // Position unchanged
        assert_eq!(robot.energy(), 5); // Energy unchanged
    }

    #[test]
    fn robot_state_management_works() {
        let mut robot = Robot::new(1, RobotType::Explorer, 0, 0, 100);

        assert_eq!(robot.state(), RobotState::Idle);

        robot.set_state(RobotState::Exploring);
        assert_eq!(robot.state(), RobotState::Exploring);

        robot.set_state(RobotState::ReturningToStation);
        assert_eq!(robot.state(), RobotState::ReturningToStation);
    }

    #[test]
    fn robot_energy_management_works() {
        let mut robot = Robot::new(1, RobotType::Explorer, 0, 0, 100);

        assert!(!robot.is_low_energy());
        assert_eq!(robot.energy(), 100);

        let result = robot.consume_energy(50);
        assert!(result.is_ok());
        assert_eq!(robot.energy(), 50);

        robot.recharge(30);
        assert_eq!(robot.energy(), 80);
    }

    #[test]
    fn robot_detects_low_energy() {
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 15);

        assert!(robot.is_low_energy());
    }

    #[test]
    fn robot_cannot_consume_more_energy_than_available() {
        let mut robot = Robot::new(1, RobotType::Explorer, 0, 0, 10);

        let result = robot.consume_energy(15);

        assert!(result.is_err());
        assert_eq!(robot.energy(), 10); // Energy unchanged
    }

    #[test]
    fn robot_can_detect_resources_at_position() {
        let mut map = Map::new_test_map(5, 5);
        map.resources.insert((2, 2), (ResourceType::Energy, 50));

        let robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);

        let resource = robot.detect_resource_at_position(&map);
        assert!(resource.is_some());
        let (resource_type, amount) = resource.unwrap();
        assert_eq!(resource_type, ResourceType::Energy);
        assert_eq!(amount, 50);
    }

    #[test]
    fn robot_cannot_collect_when_already_carrying() {
        let mut map = Map::new_test_map(5, 5);
        map.resources.insert((2, 2), (ResourceType::Energy, 50));

        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);
        robot.carrying = Some((ResourceType::Mineral, 30));

        let result = robot.collect_resource(&mut map);
        assert!(result.is_err());
    }

    #[test]
    fn robot_can_collect_resource_successfully() {
        let mut map = Map::new_test_map(5, 5);
        map.resources.insert((3, 3), (ResourceType::Mineral, 75));

        let mut robot = Robot::new(1, RobotType::Harvester, 3, 3, 100);

        let result = robot.collect_resource(&mut map);
        assert!(result.is_ok());
        assert_eq!(robot.carrying, Some((ResourceType::Mineral, 75)));

        // Resource should be removed from map
        assert!(!map.resources.contains_key(&(3, 3)));
    }

    #[test]
    fn robot_can_collect_and_remove_resource_from_map() {
        let mut map = Map::new_test_map(5, 5);
        map.resources.insert((2, 2), (ResourceType::Mineral, 30));

        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);

        let result = robot.collect_resource(&mut map);
        assert!(result.is_ok());
        assert!(robot.carrying.is_some());

        let (resource_type, amount) = robot.carrying.unwrap();
        assert_eq!(resource_type, ResourceType::Mineral);
        assert_eq!(amount, 30);

        // Resource should be removed from map
        assert!(!map.resources.contains_key(&(2, 2)));
    }

    #[test]
    fn station_creation_works() {
        let station = Station::new(5, 5);

        assert_eq!(station.position(), (5, 5));
        assert_eq!(station.discoveries, 0);
        assert_eq!(station.total_resources(), 0);
    }

    #[test]
    fn station_receives_energy_resource() {
        let mut station = Station::new(0, 0);

        station.receive_resource(ResourceType::Energy, 50);

        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50);
        assert_eq!(station.total_resources(), 50);
        assert_eq!(station.discoveries, 0); // Energy doesn't count as discovery
    }

    #[test]
    fn station_receives_scientific_interest_and_tracks_discoveries() {
        let mut station = Station::new(0, 0);

        station.receive_resource(ResourceType::ScientificInterest, 100);

        assert_eq!(
            station.get_resource_amount(&ResourceType::ScientificInterest),
            100
        );
        assert_eq!(station.total_resources(), 100);
        assert_eq!(station.discoveries, 1); // Should increment discoveries
    }

    #[test]
    fn station_accumulates_multiple_resources() {
        let mut station = Station::new(0, 0);

        station.receive_resource(ResourceType::Energy, 30);
        station.receive_resource(ResourceType::Energy, 20);
        station.receive_resource(ResourceType::Mineral, 40);

        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50);
        assert_eq!(station.get_resource_amount(&ResourceType::Mineral), 40);
        assert_eq!(station.total_resources(), 90);
    }

    #[test]
    fn robot_can_deliver_resource_to_station() {
        let mut robot = Robot::new(1, RobotType::Harvester, 0, 0, 100);
        robot.carrying = Some((ResourceType::Mineral, 25));

        let mut station = Station::new(0, 0);

        let result = robot.deliver_resource(&mut station);

        assert!(result.is_ok());
        assert!(robot.carrying.is_none());
        assert_eq!(robot.state(), RobotState::Idle);
        assert_eq!(station.get_resource_amount(&ResourceType::Mineral), 25);
    }

    #[test]
    fn robot_cannot_deliver_when_not_carrying() {
        let mut robot = Robot::new(1, RobotType::Harvester, 0, 0, 100);
        let mut station = Station::new(0, 0);

        let result = robot.deliver_resource(&mut station);

        assert!(result.is_err());
        assert_eq!(station.total_resources(), 0);
    }

    #[test]
    fn station_detects_robot_at_position() {
        let station = Station::new(3, 4);

        assert!(station.robot_at_station((3, 4)));
        assert!(!station.robot_at_station((3, 5)));
        assert!(!station.robot_at_station((2, 4)));
    }

    #[test]
    fn robot_should_return_when_carrying_resource() {
        let mut robot = Robot::new(1, RobotType::Harvester, 5, 5, 80);
        robot.carrying = Some((ResourceType::Energy, 50));

        assert!(robot.should_return_to_station());
    }

    #[test]
    fn robot_should_return_when_energy_low() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 5, 25); // Below threshold

        assert!(robot.should_return_to_station());
    }

    #[test]
    fn robot_should_not_return_when_energy_sufficient() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 5, 80); // Above threshold

        assert!(!robot.should_return_to_station());
    }

    #[test]
    fn robot_calculates_manhattan_distance_correctly() {
        let robot = Robot::new(1, RobotType::Explorer, 2, 3, 100);

        assert_eq!(robot.manhattan_distance_to((2, 3)), 0); // Same position
        assert_eq!(robot.manhattan_distance_to((5, 3)), 3); // East
        assert_eq!(robot.manhattan_distance_to((2, 7)), 4); // South
        assert_eq!(robot.manhattan_distance_to((5, 7)), 7); // Diagonal
    }

    #[test]
    fn robot_calculates_energy_to_return() {
        let robot = Robot::new(1, RobotType::Explorer, 2, 2, 100);
        let station_pos = (5, 6);

        let expected_distance = 3 + 4; // Manhattan distance
        let expected_energy = expected_distance * MOVE_ENERGY_COST;

        assert_eq!(robot.energy_to_return(station_pos), expected_energy);
    }

    #[test]
    fn robot_can_return_with_sufficient_energy() {
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 100);
        let station_pos = (3, 3); // Distance 6, needs 60 energy

        assert!(robot.can_return_to_station(station_pos));
    }

    #[test]
    fn robot_cannot_return_with_insufficient_energy() {
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 30);
        let station_pos = (5, 5); // Distance 10, needs 100 energy

        assert!(!robot.can_return_to_station(station_pos));
    }

    #[test]
    fn robot_should_continue_mission_when_safe() {
        let robot = Robot::new(1, RobotType::Explorer, 1, 1, 100);
        let station_pos = (2, 2); // Close station, low energy requirement

        assert!(robot.should_continue_mission(station_pos));
    }

    #[test]
    fn robot_should_not_continue_when_carrying() {
        let mut robot = Robot::new(1, RobotType::Harvester, 1, 1, 100);
        robot.carrying = Some((ResourceType::Mineral, 50));
        let station_pos = (2, 2);

        assert!(!robot.should_continue_mission(station_pos));
    }

    #[test]
    fn robot_should_not_continue_when_energy_insufficient() {
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 40);
        let station_pos = (5, 5); // Far station, high energy requirement

        assert!(!robot.should_continue_mission(station_pos));
    }

    #[test]
    fn station_recharges_robot_successfully() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30); // Low energy

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_ok());
        let recharged = result.unwrap();
        assert_eq!(recharged, 50); // STATION_RECHARGE_RATE
        assert_eq!(robot.energy(), 80); // 30 + 50
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50); // 100 - 50
    }

    #[test]
    fn station_cannot_recharge_without_energy() {
        let mut station = Station::new(5, 5);
        // No energy in station

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_err());
        assert_eq!(robot.energy(), 30); // Unchanged
    }

    #[test]
    fn station_cannot_recharge_full_energy_robot() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, MAX_ROBOT_ENERGY);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_err());
        assert_eq!(robot.energy(), MAX_ROBOT_ENERGY);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 100); // Unchanged
    }

    #[test]
    fn station_recharges_partial_when_limited_energy() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 20); // Limited energy

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_ok());
        let recharged = result.unwrap();
        assert_eq!(recharged, 20); // Limited by station energy
        assert_eq!(robot.energy(), 50); // 30 + 20
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 0); // All used
    }

    #[test]
    fn station_recharges_partial_when_near_full() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 90); // Near full

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_ok());
        let recharged = result.unwrap();
        assert_eq!(recharged, 10); // Only what's needed to fill up
        assert_eq!(robot.energy(), MAX_ROBOT_ENERGY);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 90); // 100 - 10
    }

    #[test]
    fn station_can_recharge_check_works() {
        let mut station = Station::new(5, 5);

        assert!(!station.can_recharge()); // No energy

        station.receive_resource(ResourceType::Energy, 50);
        assert!(station.can_recharge()); // Has energy
    }
}
