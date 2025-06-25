#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Plain = 0,
    Hill = 1,
    Mountain = 2,
    Canyon = 3,
}

impl From<u8> for TerrainType {
    fn from(value: u8) -> Self {
        match value {
            0 => TerrainType::Plain,
            1 => TerrainType::Hill,
            2 => TerrainType::Mountain,
            3 => TerrainType::Canyon,
            _ => TerrainType::Plain,
        }
    }
}

impl TerrainType {
    pub fn is_traversable(&self) -> bool {
        matches!(self, TerrainType::Plain | TerrainType::Hill)
    }

    pub fn movement_cost(&self) -> u32 {
        match self {
            TerrainType::Plain => 1,
            TerrainType::Hill => 2,
            TerrainType::Mountain | TerrainType::Canyon => 0,
        }
    }
} 