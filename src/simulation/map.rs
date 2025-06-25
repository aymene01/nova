use crate::simulation::entities::{Map, ResourceType};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
            _ => TerrainType::Plain, // Default to plain for unknown values
        }
    }
}

/// Constants for map generation
pub struct MapConstants {
    /// Frequency for Perlin noise
    pub noise_frequency: f64,
    /// Threshold for resource generation
    pub energy_threshold: f64,
    /// Threshold for mineral generation
    pub mineral_threshold: f64,
    /// Threshold for scientific interest generation
    pub scientific_threshold: f64,
    /// Maximum resource amount
    pub max_resource_amount: u32,
}

impl Default for MapConstants {
    fn default() -> Self {
        Self {
            noise_frequency: 0.1,
            energy_threshold: 0.6,
            mineral_threshold: 0.7,
            scientific_threshold: 0.85,
            max_resource_amount: 100,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error("Failed to serialize map: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Failed to write map to file: {0}")]
    IOError(#[from] std::io::Error),
}

pub type MapResult<T> = Result<T, MapError>;

impl Map {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut map = Map {
            width,
            height,
            terrain: vec![vec![0; width]; height],
            resources: HashMap::new(),
            discovered: vec![vec![false; width]; height],
            noise: Perlin::new(seed as u32),
            seed,
        };

        map.generate_terrain();
        map.generate_resources();

        map
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> MapResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn generate_terrain(&mut self) {
        let constants = MapConstants::default();

        for y in 0..self.height {
            for x in 0..self.width {
                // Generate noise value for this point
                let nx = x as f64 * constants.noise_frequency;
                let ny = y as f64 * constants.noise_frequency;
                let noise_val = self.noise.get([nx, ny]);

                // Normalize noise from [-1, 1] to [0, 1]
                let normalized = (noise_val + 1.0) / 2.0;

                // Convert to terrain type (0-3)
                let terrain_type = if normalized < 0.4 {
                    TerrainType::Plain as u8
                } else if normalized < 0.7 {
                    TerrainType::Hill as u8
                } else if normalized < 0.85 {
                    TerrainType::Mountain as u8
                } else {
                    TerrainType::Canyon as u8
                };

                self.terrain[y][x] = terrain_type;
            }
        }
    }

    fn generate_resources(&mut self) {
        let constants = MapConstants::default();
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);

        let resource_noise = Perlin::new((self.seed.wrapping_add(42)) as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                // Use a different seed for resources
                let nx = x as f64 * constants.noise_frequency * 1.5;
                let ny = y as f64 * constants.noise_frequency * 1.5;
                let noise_val = (resource_noise.get([nx, ny]) + 1.0) / 2.0;

                // Generate different types of resources based on noise value
                if noise_val > constants.scientific_threshold {
                    // Scientific interest points (rare)
                    let amount = rng.random_range(10..=constants.max_resource_amount);
                    self.resources
                        .insert((x, y), (ResourceType::ScientificInterest, amount));
                } else if noise_val > constants.mineral_threshold {
                    // Minerals (less common)
                    let amount = rng.random_range(20..=constants.max_resource_amount);
                    self.resources
                        .insert((x, y), (ResourceType::Mineral, amount));
                } else if noise_val > constants.energy_threshold {
                    // Energy (more common)
                    let amount = rng.random_range(30..=constants.max_resource_amount);
                    self.resources
                        .insert((x, y), (ResourceType::Energy, amount));
                }
            }
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
