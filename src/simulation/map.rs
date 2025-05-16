use crate::simulation::entities::{Map, ResourceType};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

/// Represents the different terrain types in the map
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

/// Errors that can occur during map operations
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum MapError {
    #[error("Position ({0}, {1}) is out of bounds")]
    OutOfBounds(usize, usize),

    #[error("No resource at position ({0}, {1})")]
    NoResourceAtPosition(usize, usize),

    #[error("Cannot collect more resources than available")]
    InsufficientResources,

    #[error("Failed to serialize map: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to write map to file: {0}")]
    IOError(#[from] io::Error),
}

/// Result type for map operations
pub type MapResult<T> = Result<T, MapError>;

#[allow(dead_code)]
impl Map {
    /// Creates a new map with the specified dimensions and seed
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

    /// Loads a map from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> MapResult<Self> {
        let file_content = fs::read_to_string(path)?;
        let mut map: Map = serde_json::from_str(&file_content)?;

        // Recreate the Perlin noise generator from the seed
        map.noise = Perlin::new(map.seed as u32);

        Ok(map)
    }

    /// Saves the map to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> MapResult<()> {
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Generates the terrain using Perlin noise
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

    /// Generates resources on the map
    fn generate_resources(&mut self) {
        let constants = MapConstants::default();
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);

        // Create a second noise instance for resource generation
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

    /// Checks if the given position is within the map bounds
    pub fn is_position_valid(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    /// Gets the terrain type at the given position
    pub fn get_terrain(&self, x: usize, y: usize) -> MapResult<TerrainType> {
        if !self.is_position_valid(x, y) {
            return Err(MapError::OutOfBounds(x, y));
        }

        Ok(TerrainType::from(self.terrain[y][x]))
    }

    /// Marks a position as discovered
    pub fn discover(&mut self, x: usize, y: usize) -> MapResult<()> {
        if !self.is_position_valid(x, y) {
            return Err(MapError::OutOfBounds(x, y));
        }

        self.discovered[y][x] = true;
        Ok(())
    }

    /// Checks if a position is discovered
    pub fn is_discovered(&self, x: usize, y: usize) -> MapResult<bool> {
        if !self.is_position_valid(x, y) {
            return Err(MapError::OutOfBounds(x, y));
        }

        Ok(self.discovered[y][x])
    }

    /// Gets the resource at the given position, if any
    pub fn get_resource(&self, x: usize, y: usize) -> MapResult<Option<(ResourceType, u32)>> {
        if !self.is_position_valid(x, y) {
            return Err(MapError::OutOfBounds(x, y));
        }

        Ok(self.resources.get(&(x, y)).cloned())
    }

    /// Collects resources from the given position
    /// Returns the type and amount of resources collected
    pub fn collect_resource(
        &mut self,
        x: usize,
        y: usize,
        amount: u32,
    ) -> MapResult<(ResourceType, u32)> {
        if !self.is_position_valid(x, y) {
            return Err(MapError::OutOfBounds(x, y));
        }

        // Get the resource if it exists
        if let Some((resource_type, available_amount)) = self.resources.get(&(x, y)).cloned() {
            if available_amount < amount {
                return Err(MapError::InsufficientResources);
            }

            // Calculate the new amount
            let new_amount = available_amount - amount;

            // If the resource is depleted, remove it
            if new_amount == 0 {
                self.resources.remove(&(x, y));
            } else {
                // Otherwise update the resource amount
                self.resources
                    .insert((x, y), (resource_type.clone(), new_amount));
            }

            Ok((resource_type, amount))
        } else {
            Err(MapError::NoResourceAtPosition(x, y))
        }
    }

    /// Gets the map's seed
    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    /// Gets the map dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Gets the movement cost for traversing the given terrain type
    pub fn movement_cost(&self, terrain_type: TerrainType) -> u32 {
        match terrain_type {
            TerrainType::Plain => 1,
            TerrainType::Hill => 2,
            TerrainType::Mountain => 3,
            TerrainType::Canyon => 4,
        }
    }

    /// Checks if the terrain at a position is traversable
    pub fn is_traversable(&self, x: usize, y: usize) -> MapResult<bool> {
        // Check if the position is valid without storing the terrain
        self.get_terrain(x, y)?;
        // In this simulation, all terrain is traversable but at different costs
        Ok(true)
    }
}
