use crate::simulation::robot_ai::robot::Robot;
use crate::domain::values::resource::ResourceType;
use noise::Perlin;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

pub const STATION_RECHARGE_RATE: u32 = 50;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationInfo {
    pub position: (usize, usize),
    pub terrain_type: u8,
    pub resource: Option<(ResourceType, u32)>,
    pub discovered_by: usize,
    pub discovery_time: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InformationConflict {
    pub position: (usize, usize),
    pub current_info: LocationInfo,
    pub new_info: LocationInfo,
    pub conflict_type: ConflictType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    ResourceAmountDifference,
    ResourceTypeConflict,
    TerrainMismatch,
    ConfidenceConflict,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepCurrent,
    AcceptNew,
    Merge,
    RequiresManualReview,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<Vec<u8>>,
    pub resources: HashMap<(usize, usize), (ResourceType, u32)>,
    pub discovered: Vec<Vec<bool>>,
    pub discovered_resources: HashMap<(usize, usize), (ResourceType, u32)>,
    pub noise: Perlin,
    pub seed: u64,
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Map", 6)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("terrain", &self.terrain)?;

        let resources_vec: Vec<((usize, usize), (ResourceType, u32))> = self
            .resources
            .iter()
            .map(|(&k, v)| (k, v.clone()))
            .collect();
        state.serialize_field("resources", &resources_vec)?;

        state.serialize_field("discovered", &self.discovered)?;
        state.serialize_field("seed", &self.seed)?;
        state.end()
    }
}

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
            resources: Vec<((usize, usize), (ResourceType, u32))>,
            discovered: Vec<Vec<bool>>,
            seed: u64,
        }

        let helper = MapHelper::deserialize(deserializer)?;
        let resources: HashMap<(usize, usize), (ResourceType, u32)> =
            helper.resources.into_iter().collect();
        let noise = Perlin::new(helper.seed as u32);

        Ok(Map {
            width: helper.width,
            height: helper.height,
            terrain: helper.terrain,
            resources,
            discovered: helper.discovered,
            discovered_resources: HashMap::new(),
            noise,
            seed: helper.seed,
        })
    }
}
pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
}

impl Station {
    pub fn new(x: usize, y: usize) -> Self {
        let mut resources = HashMap::new();
        resources.insert(ResourceType::Energy, 100);
        Self {
            resources,
            discoveries: 0,
            x,
            y,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn receive_resource(&mut self, resource_type: ResourceType, amount: u32) {
        let resource_type_clone = resource_type.clone();
        *self.resources.entry(resource_type_clone).or_insert(0) += amount;
        if resource_type == ResourceType::ScientificInterest {
            self.discoveries += 1;
        }
    }

    pub fn get_resource_amount(&self, resource_type: &ResourceType) -> u32 {
        *self.resources.get(resource_type).unwrap_or(&0)
    }

    pub fn robot_at_station(&self, robot_position: (usize, usize)) -> bool {
        robot_position == self.position()
    }

    pub fn recharge_robot(&mut self, robot: &mut Robot) -> Result<u32, &'static str> {
        let energy_available = self.get_resource_amount(&ResourceType::Energy);
        if energy_available == 0 {
            return Err("No energy available for recharging");
        }

        let energy_needed = robot.max_energy() - robot.energy();
        if energy_needed == 0 {
            return Err("Robot is already at full energy");
        }

        let energy_to_transfer = std::cmp::min(energy_needed, STATION_RECHARGE_RATE);
        let actual_transfer = std::cmp::min(energy_to_transfer, energy_available);

        robot.recharge(actual_transfer);
        self.resources
            .insert(ResourceType::Energy, energy_available - actual_transfer);

        Ok(actual_transfer)
    }

    pub fn can_recharge(&self) -> bool {
        self.get_resource_amount(&ResourceType::Energy) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::robot_ai::types::{Direction, RobotType};

    #[test]
    fn robot_creation_works() {
        let robot = Robot::new(1, RobotType::Explorer, 5, 10, 100);

        assert_eq!(robot.id, 1);
        assert_eq!(robot.position(), (5, 10));
        assert_eq!(robot.energy(), 100);
        assert_eq!(robot.robot_type, RobotType::Explorer);
        assert!(robot.carrying.is_none());
    }

    #[test]
    fn robot_move_north_with_sufficient_energy() {
        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 50);

        let mut map = Map::new(10, 10, 42);
        for row in map.terrain.iter_mut() {
            for cell in row.iter_mut() {
                *cell = 0;
            }
        }

        let result = robot.move_in_direction(Direction::North, &map);
        assert!(result.is_ok());
        if let Ok(movement_cost) = result {
            robot.consume_energy_for_movement(movement_cost).unwrap();
        }

        assert_eq!(robot.position(), (5, 4));
        assert_eq!(robot.energy(), 50 - robot.energy_consumption_rate());
    }

    #[test]
    fn robot_detects_low_energy() {
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 15);

        assert!(robot.is_low_energy());
    }

    #[test]
    fn robot_cannot_collect_when_already_carrying() {
        let mut map = Map::new(5, 5, 42);
        map.resources.insert((2, 2), (ResourceType::Energy, 50));

        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);
        robot.carrying = Some((ResourceType::Mineral, 30));

        let result = robot.collect_resource(&mut map);
        assert!(result.is_err());
    }

    #[test]
    fn robot_can_collect_resource_successfully() {
        let mut map = Map::new(5, 5, 42);
        map.resources.insert((3, 3), (ResourceType::Mineral, 75));

        let mut robot = Robot::new(1, RobotType::Harvester, 3, 3, 100);

        let result = robot.collect_resource(&mut map);
        assert!(result.is_ok());
        assert_eq!(robot.carrying, Some((ResourceType::Mineral, 75)));

        assert!(!map.resources.contains_key(&(3, 3)));
    }

    #[test]
    fn robot_can_collect_and_remove_resource_from_map() {
        let mut map = Map::new(5, 5, 42);
        map.resources.insert((2, 2), (ResourceType::Mineral, 30));

        let mut robot = Robot::new(1, RobotType::Harvester, 2, 2, 100);

        let result = robot.collect_resource(&mut map);
        assert!(result.is_ok());
        assert!(robot.carrying.is_some());

        let (resource_type, amount) = robot.carrying.unwrap();
        assert_eq!(resource_type, ResourceType::Mineral);
        assert_eq!(amount, 30);

        assert!(!map.resources.contains_key(&(2, 2)));
    }

    #[test]
    fn station_creation_works() {
        let station = Station::new(5, 5);

        assert_eq!(station.position(), (5, 5));
        assert_eq!(station.discoveries, 0);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 100);
    }

    #[test]
    fn station_receives_energy_resource() {
        let mut station = Station::new(0, 0);

        station.receive_resource(ResourceType::Energy, 50);

        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50);
        assert_eq!(station.discoveries, 0);
    }

    #[test]
    fn station_receives_scientific_interest_and_tracks_discoveries() {
        let mut station = Station::new(0, 0);

        station.receive_resource(ResourceType::ScientificInterest, 100);

        assert_eq!(
            station.get_resource_amount(&ResourceType::ScientificInterest),
            100
        );
        assert_eq!(station.discoveries, 1);
    }

    #[test]
    fn station_accumulates_multiple_resources() {
        let mut station = Station::new(0, 0);

        station.receive_resource(ResourceType::Energy, 30);
        station.receive_resource(ResourceType::Energy, 20);
        station.receive_resource(ResourceType::Mineral, 40);

        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50);
        assert_eq!(station.get_resource_amount(&ResourceType::Mineral), 40);
    }

    #[test]
    fn robot_cannot_deliver_when_not_carrying() {
        let mut robot = Robot::new(1, RobotType::Harvester, 0, 0, 100);
        let mut station = Station::new(0, 0);

        let result = robot.deliver_resource(&mut station);

        assert!(result.is_err());
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 100);
    }

    #[test]
    fn station_detects_robot_at_position() {
        let station = Station::new(3, 4);

        assert!(station.robot_at_station((3, 4)));
        assert!(!station.robot_at_station((3, 5)));
        assert!(!station.robot_at_station((2, 4)));
    }

    #[test]
    fn station_recharges_robot_successfully() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_ok());
        let recharged = result.unwrap();
        assert_eq!(recharged, 50);
        assert_eq!(robot.energy(), 80);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 50);
    }

    #[test]
    fn station_cannot_recharge_without_energy() {
        let mut station = Station::new(5, 5);
        // Remove all energy to test the failure case
        station.resources.insert(ResourceType::Energy, 0);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_err());
        assert_eq!(robot.energy(), 30);
    }

    #[test]
    fn station_cannot_recharge_full_energy_robot() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 100);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_err());
        assert_eq!(robot.energy(), robot.max_energy());
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 100);
    }

    #[test]
    fn station_recharges_partial_when_limited_energy() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 20);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 30);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_ok());
        let recharged = result.unwrap();
        assert_eq!(recharged, 20);
        assert_eq!(robot.energy(), 50);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 0);
    }

    #[test]
    fn station_recharges_partial_when_near_full() {
        let mut station = Station::new(5, 5);
        station.receive_resource(ResourceType::Energy, 100);

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 90);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_ok());
        let recharged = result.unwrap();
        assert_eq!(recharged, 10);
        assert_eq!(robot.energy(), 100);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 90);
    }

    #[test]
    fn station_can_recharge_check_works() {
        let mut station = Station::new(0, 0);
        // Start with no energy for this test
        station.resources.insert(ResourceType::Energy, 0);

        assert!(!station.can_recharge());

        station.receive_resource(ResourceType::Energy, 100);
        assert!(station.can_recharge());

        let mut robot1 = Robot::new(1, RobotType::Explorer, 0, 0, 50);
        let mut robot2 = Robot::new(2, RobotType::Explorer, 0, 0, 50);
        let _ = station.recharge_robot(&mut robot1);
        let _ = station.recharge_robot(&mut robot2);

        assert!(!station.can_recharge());
    }
}
