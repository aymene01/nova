use crate::domain::values::resource::ResourceType;
use serde::{Deserialize, Serialize};

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
