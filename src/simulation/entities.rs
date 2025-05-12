
use std::collections::HashMap;
use noise::Perlin;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    noise: Perlin,
    seed: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RobotType {
    Explorer,
    Harvester,
    Scientist,
}

#[allow(dead_code)]
pub struct Robot {
    pub id: usize,
    pub robot_type: RobotType,
    pub x: usize,
    pub y: usize,
    pub energy: u32,
    pub carrying: Option<(ResourceType, u32)>,
}

#[allow(dead_code)]
pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
}