use noise::Perlin;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

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
#[derive(Serialize, Deserialize)]
pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
}
