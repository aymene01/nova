use crate::simulation::entities::ResourceType;

#[derive(Debug, Clone, PartialEq)]
pub enum RobotType {
    Explorer,
    Harvester,
    Scientist,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum RobotState {
    Idle,
    Exploring,
    MovingToResource,
    Harvesting,
    ReturningToStation,
    Analyzing,
    Searching,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub task_type: TaskType,
    #[allow(dead_code)]
    pub target_position: Option<(usize, usize)>,
    #[allow(dead_code)]
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    Explore(ExploreTask),
    Harvest(HarvestTask),
    Analyze(AnalyzeTask),
    ReturnToStation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExploreTask {
    pub target_area: (usize, usize),
    pub radius: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HarvestTask {
    pub resource_type: ResourceType,
    pub target_position: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzeTask {
    pub target_position: (usize, usize),
    pub analysis_type: AnalysisType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisType {
    Chemical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[allow(dead_code)]
impl Direction {
    pub fn to_delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }

    pub fn all() -> Vec<Direction> {
        vec![
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
    }
}
