use crate::simulation::robot_ai::robot::Robot;
use noise::Perlin;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Movement constants
#[allow(dead_code)]
pub const HARVEST_ENERGY_COST: u32 = 5;
#[allow(dead_code)]
pub const STARTING_ENERGY: u32 = 100;
pub const STATION_RECHARGE_RATE: u32 = 50; // Energy recharged per station visit

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Energy,
    Mineral,
    ScientificInterest,
}

/// Information about a discovered location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationInfo {
    pub position: (usize, usize),
    pub terrain_type: u8,
    pub resource: Option<(ResourceType, u32)>,
    pub discovered_by: usize, // robot ID
    pub discovery_time: u64,  // simulation tick when discovered
    pub confidence: f32,      // confidence level (0.0 to 1.0)
}

/// Represents a conflict between two pieces of information
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

/// Resolution strategy for conflicts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepCurrent,
    AcceptNew,
    Merge,
    RequiresManualReview,
}

/// Station's knowledge base of discovered locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationKnowledge {
    pub locations: HashMap<(usize, usize), LocationInfo>,
    pub conflicts: Vec<InformationConflict>,
    pub merge_statistics: MergeStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MergeStatistics {
    pub total_merges: u32,
    pub successful_merges: u32,
    pub conflicts_resolved: u32,
    pub manual_reviews_required: u32,
}

impl StationKnowledge {
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
            conflicts: Vec::new(),
            merge_statistics: MergeStatistics::default(),
        }
    }

    /// Merge new information with existing knowledge
    #[allow(dead_code)]
    pub fn merge_information(
        &mut self,
        new_info: LocationInfo,
    ) -> Result<ConflictResolution, String> {
        self.merge_statistics.total_merges += 1;

        if let Some(existing_info) = self.locations.get(&new_info.position) {
            // Check for conflicts
            if let Some(conflict) = self.detect_conflict(existing_info, &new_info) {
                let resolution = self.resolve_conflict(&conflict)?;

                match resolution {
                    ConflictResolution::AcceptNew => {
                        self.locations.insert(new_info.position, new_info);
                        self.merge_statistics.successful_merges += 1;
                        self.merge_statistics.conflicts_resolved += 1;
                    }
                    ConflictResolution::Merge => {
                        let merged_info = self.merge_location_info(existing_info, &new_info)?;
                        self.locations.insert(new_info.position, merged_info);
                        self.merge_statistics.successful_merges += 1;
                        self.merge_statistics.conflicts_resolved += 1;
                    }
                    ConflictResolution::KeepCurrent => {
                        self.merge_statistics.successful_merges += 1;
                        self.merge_statistics.conflicts_resolved += 1;
                    }
                    ConflictResolution::RequiresManualReview => {
                        self.conflicts.push(conflict);
                        self.merge_statistics.manual_reviews_required += 1;
                    }
                }

                Ok(resolution)
            } else {
                // No conflict, update with newer/better information
                let updated_info = self.update_location_info(existing_info, &new_info);
                self.locations.insert(new_info.position, updated_info);
                self.merge_statistics.successful_merges += 1;
                Ok(ConflictResolution::Merge)
            }
        } else {
            // New location, add it directly
            self.locations.insert(new_info.position, new_info);
            self.merge_statistics.successful_merges += 1;
            Ok(ConflictResolution::AcceptNew)
        }
    }

    /// Detect conflicts between existing and new information
    #[allow(dead_code)]
    fn detect_conflict(
        &self,
        existing: &LocationInfo,
        new: &LocationInfo,
    ) -> Option<InformationConflict> {
        // Check for terrain mismatch
        if existing.terrain_type != new.terrain_type {
            return Some(InformationConflict {
                position: new.position,
                current_info: existing.clone(),
                new_info: new.clone(),
                conflict_type: ConflictType::TerrainMismatch,
            });
        }

        // Check for resource conflicts
        match (&existing.resource, &new.resource) {
            (Some((existing_type, existing_amount)), Some((new_type, new_amount))) => {
                if existing_type != new_type {
                    return Some(InformationConflict {
                        position: new.position,
                        current_info: existing.clone(),
                        new_info: new.clone(),
                        conflict_type: ConflictType::ResourceTypeConflict,
                    });
                }

                // Check for significant amount difference (>20% difference)
                let amount_diff = (*existing_amount as f32 - *new_amount as f32).abs();
                let avg_amount = (*existing_amount + *new_amount) as f32 / 2.0;
                if amount_diff / avg_amount > 0.2 {
                    return Some(InformationConflict {
                        position: new.position,
                        current_info: existing.clone(),
                        new_info: new.clone(),
                        conflict_type: ConflictType::ResourceAmountDifference,
                    });
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                // One has resource, other doesn't - potential conflict
                let confidence_diff = (existing.confidence - new.confidence).abs();
                if confidence_diff > 0.3 {
                    return Some(InformationConflict {
                        position: new.position,
                        current_info: existing.clone(),
                        new_info: new.clone(),
                        conflict_type: ConflictType::ConfidenceConflict,
                    });
                }
            }
            _ => {} // Both None, no conflict
        }

        None
    }

    /// Resolve conflicts using intelligent strategies
    #[allow(dead_code)]
    fn resolve_conflict(
        &self,
        conflict: &InformationConflict,
    ) -> Result<ConflictResolution, String> {
        match conflict.conflict_type {
            ConflictType::ResourceAmountDifference => {
                // Use weighted average based on confidence and recency
                Ok(ConflictResolution::Merge)
            }
            ConflictType::ResourceTypeConflict => {
                // Prefer higher confidence, or newer information if confidence is similar
                if (conflict.new_info.confidence - conflict.current_info.confidence).abs() < 0.1 {
                    // Similar confidence, prefer newer
                    if conflict.new_info.discovery_time > conflict.current_info.discovery_time {
                        Ok(ConflictResolution::AcceptNew)
                    } else {
                        Ok(ConflictResolution::KeepCurrent)
                    }
                } else if conflict.new_info.confidence > conflict.current_info.confidence {
                    Ok(ConflictResolution::AcceptNew)
                } else {
                    Ok(ConflictResolution::KeepCurrent)
                }
            }
            ConflictType::TerrainMismatch => {
                // Terrain should be consistent - this is a serious conflict
                Ok(ConflictResolution::RequiresManualReview)
            }
            ConflictType::ConfidenceConflict => {
                // Prefer higher confidence
                if conflict.new_info.confidence > conflict.current_info.confidence {
                    Ok(ConflictResolution::AcceptNew)
                } else {
                    Ok(ConflictResolution::KeepCurrent)
                }
            }
        }
    }

    /// Merge two LocationInfo objects intelligently
    #[allow(dead_code)]
    fn merge_location_info(
        &self,
        existing: &LocationInfo,
        new: &LocationInfo,
    ) -> Result<LocationInfo, String> {
        let mut merged = existing.clone();

        // Use weighted average for resource amounts
        if let (Some((existing_type, existing_amount)), Some((new_type, new_amount))) =
            (&existing.resource, &new.resource)
        {
            if existing_type == new_type {
                // Weighted average based on confidence
                let total_confidence = existing.confidence + new.confidence;
                let weighted_amount = ((*existing_amount as f32 * existing.confidence)
                    + (*new_amount as f32 * new.confidence))
                    / total_confidence;

                merged.resource = Some((existing_type.clone(), weighted_amount as u32));
            }
        }

        // Update confidence to average
        merged.confidence = (existing.confidence + new.confidence) / 2.0;

        // Update discovery time to most recent
        merged.discovery_time = merged.discovery_time.max(new.discovery_time);

        // Update discovered_by to most confident robot
        if new.confidence > existing.confidence {
            merged.discovered_by = new.discovered_by;
        }

        Ok(merged)
    }

    /// Update location info with newer/better information
    #[allow(dead_code)]
    fn update_location_info(&self, existing: &LocationInfo, new: &LocationInfo) -> LocationInfo {
        let mut updated = existing.clone();

        // Update if new information is more recent or more confident
        if new.discovery_time > existing.discovery_time || new.confidence > existing.confidence {
            updated.confidence = new.confidence.max(existing.confidence);
            updated.discovery_time = new.discovery_time.max(existing.discovery_time);

            if new.confidence > existing.confidence {
                updated.discovered_by = new.discovered_by;
                if new.resource.is_some() {
                    updated.resource = new.resource.clone();
                }
            }
        }

        updated
    }

    /// Get information about a specific location
    #[allow(dead_code)]
    pub fn get_location_info(&self, position: (usize, usize)) -> Option<&LocationInfo> {
        self.locations.get(&position)
    }

    /// Get all discovered locations
    #[allow(dead_code)]
    pub fn get_all_locations(&self) -> &HashMap<(usize, usize), LocationInfo> {
        &self.locations
    }

    /// Get pending conflicts
    #[allow(dead_code)]
    pub fn get_conflicts(&self) -> &Vec<InformationConflict> {
        &self.conflicts
    }

    /// Get merge statistics
    #[allow(dead_code)]
    pub fn get_merge_statistics(&self) -> &MergeStatistics {
        &self.merge_statistics
    }

    /// Resolve a pending conflict manually
    #[allow(dead_code)]
    pub fn resolve_manual_conflict(
        &mut self,
        conflict_index: usize,
        resolution: ConflictResolution,
    ) -> Result<(), String> {
        if conflict_index >= self.conflicts.len() {
            return Err("Invalid conflict index".to_string());
        }

        let conflict = self.conflicts.remove(conflict_index);

        match resolution {
            ConflictResolution::AcceptNew => {
                self.locations.insert(conflict.position, conflict.new_info);
            }
            ConflictResolution::KeepCurrent => {
                // Keep existing, no change needed
            }
            ConflictResolution::Merge => {
                let merged =
                    self.merge_location_info(&conflict.current_info, &conflict.new_info)?;
                self.locations.insert(conflict.position, merged);
            }
            ConflictResolution::RequiresManualReview => {
                // Put it back
                self.conflicts.push(conflict);
                return Err("Cannot resolve to manual review".to_string());
            }
        }

        self.merge_statistics.conflicts_resolved += 1;
        Ok(())
    }
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
#[derive(Serialize, Deserialize)]
pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
    pub knowledge: StationKnowledge,
}

impl Station {
    #[allow(dead_code)]
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            resources: HashMap::new(),
            discoveries: 0,
            x,
            y,
            knowledge: StationKnowledge::new(),
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

    pub fn robot_at_station(&self, robot_position: (usize, usize)) -> bool {
        self.position() == robot_position
    }

    pub fn recharge_robot(&mut self, robot: &mut Robot) -> Result<u32, &'static str> {
        let energy_available = self.get_resource_amount(&ResourceType::Energy);

        if energy_available == 0 {
            return Err("No energy available at station");
        }

        let current_energy = robot.energy();
        if current_energy >= robot.max_energy() {
            return Err("Robot already at full energy");
        }

        let energy_needed = robot.max_energy() - current_energy;
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

    /// Process robot discoveries and merge with station knowledge
    #[allow(dead_code)]
    pub fn process_robot_discovery(
        &mut self,
        robot_id: usize,
        position: (usize, usize),
        terrain_type: u8,
        resource: Option<(ResourceType, u32)>,
        discovery_time: u64,
        confidence: f32,
    ) -> Result<ConflictResolution, String> {
        let location_info = LocationInfo {
            position,
            terrain_type,
            resource,
            discovered_by: robot_id,
            discovery_time,
            confidence,
        };

        self.knowledge.merge_information(location_info)
    }

    /// Get station's knowledge about a location
    #[allow(dead_code)]
    pub fn get_location_knowledge(&self, position: (usize, usize)) -> Option<&LocationInfo> {
        self.knowledge.get_location_info(position)
    }

    /// Get all discovered locations
    #[allow(dead_code)]
    pub fn get_all_knowledge(&self) -> &HashMap<(usize, usize), LocationInfo> {
        self.knowledge.get_all_locations()
    }

    /// Get pending conflicts that need resolution
    #[allow(dead_code)]
    pub fn get_pending_conflicts(&self) -> &Vec<InformationConflict> {
        self.knowledge.get_conflicts()
    }

    /// Get merge statistics
    #[allow(dead_code)]
    pub fn get_knowledge_statistics(&self) -> &MergeStatistics {
        self.knowledge.get_merge_statistics()
    }

    /// Manually resolve a conflict
    #[allow(dead_code)]
    pub fn resolve_conflict(
        &mut self,
        conflict_index: usize,
        resolution: ConflictResolution,
    ) -> Result<(), String> {
        self.knowledge
            .resolve_manual_conflict(conflict_index, resolution)
    }

    /// Get confidence-weighted resource estimates for planning
    #[allow(dead_code)]
    pub fn get_resource_estimates(
        &self,
        resource_type: &ResourceType,
    ) -> Vec<(usize, usize, u32, f32)> {
        self.knowledge
            .locations
            .iter()
            .filter_map(|((x, y), info)| {
                if let Some((res_type, amount)) = &info.resource {
                    if res_type == resource_type {
                        Some((*x, *y, *amount, info.confidence))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get exploration recommendations based on knowledge gaps
    #[allow(dead_code)]
    pub fn get_exploration_recommendations(
        &self,
        map_width: usize,
        map_height: usize,
    ) -> Vec<(usize, usize)> {
        let mut recommendations = Vec::new();

        // Find areas with low coverage or conflicting information
        for y in 0..map_height {
            for x in 0..map_width {
                let position = (x, y);

                // Check if we have knowledge about this position
                if let Some(info) = self.knowledge.get_location_info(position) {
                    // Recommend re-exploration if confidence is low
                    if info.confidence < 0.7 {
                        recommendations.push(position);
                    }
                } else {
                    // Recommend exploration of unknown areas
                    recommendations.push(position);
                }
            }
        }

        // Sort by priority (unknown areas first, then low confidence)
        recommendations.sort_by(|a, b| {
            let a_info = self.knowledge.get_location_info(*a);
            let b_info = self.knowledge.get_location_info(*b);

            match (a_info, b_info) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Less, // Unknown areas first
                (Some(_), None) => std::cmp::Ordering::Greater,
                (Some(a_info), Some(b_info)) => {
                    // Lower confidence first
                    a_info
                        .confidence
                        .partial_cmp(&b_info.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        });

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::robot_ai::types::{Direction, RobotState, RobotType};

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

        let result = robot.move_in_direction(Direction::North, &Map::new_test_map(10, 10));
        robot.consume_energy().unwrap();

        assert!(result.is_ok());
        assert_eq!(robot.position(), (5, 4)); // North reduces Y
        assert_eq!(robot.energy(), 50 - robot.energy_consumption_rate());
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

        let result = robot.consume_energy();
        assert!(result.is_ok());
        assert_eq!(robot.energy(), 100 - robot.energy_consumption_rate());

        robot.recharge(30);
        assert_eq!(robot.energy(), 100 - robot.energy_consumption_rate() + 30);
    }

    #[test]
    fn robot_detects_low_energy() {
        let robot = Robot::new(1, RobotType::Explorer, 0, 0, 15);

        assert!(robot.is_low_energy());
    }

    #[test]
    fn robot_cannot_consume_more_energy_than_available() {
        let mut robot = Robot::new(1, RobotType::Explorer, 0, 0, 1);

        let result = robot.consume_energy();

        assert!(result.is_err());
        assert_eq!(robot.energy(), 1); // Energy unchanged
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

        let mut robot = Robot::new(1, RobotType::Explorer, 5, 5, 100);

        let result = station.recharge_robot(&mut robot);

        assert!(result.is_err());
        assert_eq!(robot.energy(), robot.max_energy());
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
        assert_eq!(robot.energy(), 100);
        assert_eq!(station.get_resource_amount(&ResourceType::Energy), 90); // 100 - 10
    }

    #[test]
    fn station_can_recharge_check_works() {
        let mut station = Station::new(0, 0);

        // Initially no energy
        assert!(!station.can_recharge());

        // Add energy
        station.receive_resource(ResourceType::Energy, 100);
        assert!(station.can_recharge());

        // Consume all energy by recharging multiple robots
        let mut robot1 = Robot::new(1, RobotType::Explorer, 0, 0, 50);
        let mut robot2 = Robot::new(2, RobotType::Explorer, 0, 0, 50);
        let _ = station.recharge_robot(&mut robot1);
        let _ = station.recharge_robot(&mut robot2);

        // Should not be able to recharge anymore (100 energy used, 50 each)
        assert!(!station.can_recharge());
    }

    #[test]
    fn station_processes_robot_discovery_successfully() {
        let mut station = Station::new(5, 5);

        let result = station.process_robot_discovery(
            1,                                // robot_id
            (10, 15),                         // position
            2,                                // terrain_type (Hill)
            Some((ResourceType::Energy, 50)), // resource
            100,                              // discovery_time
            0.8,                              // confidence
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictResolution::AcceptNew);

        // Check that knowledge was stored
        let knowledge = station.get_location_knowledge((10, 15));
        assert!(knowledge.is_some());

        let info = knowledge.unwrap();
        assert_eq!(info.discovered_by, 1);
        assert_eq!(info.terrain_type, 2);
        assert_eq!(info.resource, Some((ResourceType::Energy, 50)));
        assert_eq!(info.confidence, 0.8);
    }

    #[test]
    fn station_merges_compatible_information() {
        let mut station = Station::new(5, 5);

        // First discovery
        station
            .process_robot_discovery(1, (3, 4), 1, Some((ResourceType::Mineral, 40)), 50, 0.7)
            .unwrap();

        // Second discovery with different but significant amount (>20% different: 40 vs 60)
        let result = station.process_robot_discovery(
            2,
            (3, 4),
            1,
            Some((ResourceType::Mineral, 60)),
            60,
            0.8,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictResolution::Merge);

        let info = station.get_location_knowledge((3, 4)).unwrap();
        // Should be weighted average: (40*0.7 + 60*0.8) / (0.7 + 0.8) = 76/1.5 ≈ 50.67
        assert!(info.resource.is_some());
        let (_, amount) = info.resource.as_ref().unwrap();
        assert!(*amount >= 50 && *amount <= 52); // Allow for rounding
    }

    #[test]
    fn station_detects_resource_type_conflicts() {
        let mut station = Station::new(5, 5);

        // First discovery - Energy
        station
            .process_robot_discovery(1, (7, 8), 0, Some((ResourceType::Energy, 30)), 100, 0.6)
            .unwrap();

        // Second discovery - Mineral (conflict!)
        let result = station.process_robot_discovery(
            2,
            (7, 8),
            0,
            Some((ResourceType::Mineral, 35)),
            110,
            0.9,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictResolution::AcceptNew); // Higher confidence wins

        // Should have accepted the new (higher confidence) information
        let info = station.get_location_knowledge((7, 8)).unwrap();
        assert_eq!(info.resource, Some((ResourceType::Mineral, 35)));
        assert_eq!(info.discovered_by, 2);
    }

    #[test]
    fn station_handles_terrain_mismatch_conflicts() {
        let mut station = Station::new(5, 5);

        // First discovery
        station
            .process_robot_discovery(1, (2, 3), 0, None, 50, 0.8)
            .unwrap(); // Plain

        // Second discovery with different terrain
        let result = station.process_robot_discovery(2, (2, 3), 2, None, 60, 0.7); // Mountain

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictResolution::RequiresManualReview);

        // Should have a pending conflict
        assert_eq!(station.get_pending_conflicts().len(), 1);

        let conflict = &station.get_pending_conflicts()[0];
        assert_eq!(conflict.conflict_type, ConflictType::TerrainMismatch);
        assert_eq!(conflict.position, (2, 3));
    }

    #[test]
    fn station_resolves_manual_conflicts() {
        let mut station = Station::new(5, 5);

        // Create a terrain conflict
        station
            .process_robot_discovery(1, (1, 1), 0, None, 50, 0.8)
            .unwrap();
        station
            .process_robot_discovery(2, (1, 1), 2, None, 60, 0.7)
            .unwrap();

        assert_eq!(station.get_pending_conflicts().len(), 1);

        // Manually resolve by accepting new
        let result = station.resolve_conflict(0, ConflictResolution::AcceptNew);
        assert!(result.is_ok());

        // Conflict should be resolved
        assert_eq!(station.get_pending_conflicts().len(), 0);

        // Should have new terrain type
        let info = station.get_location_knowledge((1, 1)).unwrap();
        assert_eq!(info.terrain_type, 2);
    }

    #[test]
    fn station_provides_resource_estimates() {
        let mut station = Station::new(5, 5);

        // Add several energy discoveries
        station
            .process_robot_discovery(1, (1, 1), 0, Some((ResourceType::Energy, 40)), 50, 0.9)
            .unwrap();
        station
            .process_robot_discovery(2, (2, 2), 0, Some((ResourceType::Energy, 60)), 60, 0.7)
            .unwrap();
        station
            .process_robot_discovery(3, (3, 3), 0, Some((ResourceType::Mineral, 30)), 70, 0.8)
            .unwrap();

        let energy_estimates = station.get_resource_estimates(&ResourceType::Energy);
        assert_eq!(energy_estimates.len(), 2);

        // Should contain both energy locations
        let positions: Vec<(usize, usize)> = energy_estimates
            .iter()
            .map(|(x, y, _, _)| (*x, *y))
            .collect();
        assert!(positions.contains(&(1, 1)));
        assert!(positions.contains(&(2, 2)));

        // Check confidence values are included
        for (_, _, _, confidence) in &energy_estimates {
            assert!(*confidence > 0.0 && *confidence <= 1.0);
        }
    }

    #[test]
    fn station_provides_exploration_recommendations() {
        let mut station = Station::new(5, 5);

        // Add some discoveries with varying confidence
        station
            .process_robot_discovery(1, (0, 0), 0, None, 50, 0.9)
            .unwrap(); // High confidence
        station
            .process_robot_discovery(2, (1, 1), 0, None, 60, 0.5)
            .unwrap(); // Low confidence

        let recommendations = station.get_exploration_recommendations(3, 3);

        // Should recommend unknown areas and low confidence areas
        assert!(!recommendations.is_empty());

        // Low confidence area should be recommended
        assert!(recommendations.contains(&(1, 1)));

        // Unknown areas should be recommended
        assert!(recommendations.contains(&(2, 2)));
    }

    #[test]
    fn station_tracks_merge_statistics() {
        let mut station = Station::new(5, 5);

        let stats = station.get_knowledge_statistics();
        assert_eq!(stats.total_merges, 0);
        assert_eq!(stats.successful_merges, 0);

        // Process some discoveries
        station
            .process_robot_discovery(1, (0, 0), 0, None, 50, 0.8)
            .unwrap();
        station
            .process_robot_discovery(2, (1, 1), 0, None, 60, 0.7)
            .unwrap();

        // Create a conflict (terrain mismatch)
        station
            .process_robot_discovery(3, (0, 0), 2, None, 70, 0.6)
            .unwrap();

        let stats = station.get_knowledge_statistics();
        assert_eq!(stats.total_merges, 3);
        assert_eq!(stats.successful_merges, 2);
        assert_eq!(stats.manual_reviews_required, 1);
    }

    #[test]
    fn station_knowledge_handles_resource_amount_differences() {
        let mut station = Station::new(5, 5);

        // First discovery
        station
            .process_robot_discovery(1, (4, 4), 0, Some((ResourceType::Mineral, 100)), 50, 0.8)
            .unwrap();

        // Second discovery with significantly different amount (>20% difference)
        let result = station.process_robot_discovery(
            2,
            (4, 4),
            0,
            Some((ResourceType::Mineral, 50)),
            60,
            0.8,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictResolution::Merge);

        // Should have merged the amounts based on confidence weighting
        let info = station.get_location_knowledge((4, 4)).unwrap();
        if let Some((_, amount)) = &info.resource {
            // Weighted average: (100*0.8 + 50*0.8) / (0.8 + 0.8) = 75
            assert_eq!(*amount, 75);
        } else {
            panic!("Expected resource information");
        }
    }
}
