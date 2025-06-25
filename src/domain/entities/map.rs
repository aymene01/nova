use crate::domain::values::resource::ResourceType;
use crate::domain::values::terrain::TerrainType;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct MapConstants {
    pub noise_frequency: f64,
    pub energy_threshold: f64,
    pub mineral_threshold: f64,
    pub scientific_threshold: f64,
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

impl Map {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut map = Map {
            width,
            height,
            terrain: vec![vec![0; width]; height],
            resources: HashMap::new(),
            discovered: vec![vec![false; width]; height],
            discovered_resources: HashMap::new(),
            noise: Perlin::new(seed as u32),
            seed,
        };

        map.generate_terrain();
        map.generate_resources();

        map
    }

    pub fn discover_resource_at(
        &mut self,
        position: (usize, usize),
    ) -> Option<(ResourceType, u32)> {
        if let Some(resource) = self.resources.get(&position) {
            let resource_info = resource.clone();
            self.discovered_resources
                .insert(position, resource_info.clone());
            Some(resource_info)
        } else {
            None
        }
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
                let nx = x as f64 * constants.noise_frequency;
                let ny = y as f64 * constants.noise_frequency;
                let noise_val = self.noise.get([nx, ny]);

                let normalized = (noise_val + 1.0) / 2.0;

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
                let nx = x as f64 * constants.noise_frequency * 1.5;
                let ny = y as f64 * constants.noise_frequency * 1.5;
                let noise_val = (resource_noise.get([nx, ny]) + 1.0) / 2.0;

                if noise_val > constants.scientific_threshold {
                    let amount = rng.random_range(10..=constants.max_resource_amount);
                    self.resources
                        .insert((x, y), (ResourceType::ScientificInterest, amount));
                } else if noise_val > constants.mineral_threshold {
                    let amount = rng.random_range(20..=constants.max_resource_amount);
                    self.resources
                        .insert((x, y), (ResourceType::Mineral, amount));
                } else if noise_val > constants.energy_threshold {
                    let amount = rng.random_range(30..=constants.max_resource_amount);
                    self.resources
                        .insert((x, y), (ResourceType::Energy, amount));
                }
            }
        }
    }
} 