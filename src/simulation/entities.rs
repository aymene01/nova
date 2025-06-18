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
#[derive(Serialize, Deserialize)]
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
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
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
}
