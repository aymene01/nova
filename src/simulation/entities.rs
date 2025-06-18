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
}
