mod cli;
mod config;
mod simulation;

use crate::simulation::entities::Map;
use crate::simulation::visualization::MapVisualizer;
use config::Config;

fn main() {
    if let Some(cfg) = cli::args::parse_args() {
        start_simulation(cfg);
    } else {
        println!("use cmd start to start")
    }
}

fn start_simulation(config: Config) {
    println!("Starting simulation with:");
    println!("  Seed: {}", config.seed);
    println!("  Map: {}x{}", config.map_width, config.map_height);
    println!("  Robots: {}", config.robots_count);

    // Create a new map
    let map = Map::new(config.map_width, config.map_height, config.seed);

    // Visualize the map
    MapVisualizer::visualize(&map);

    // Save the map for testing
    let map_path = format!("map_seed_{}.json", config.seed);
    if let Err(e) = map.save_to_file(&map_path) {
        eprintln!("Failed to save map: {}", e);
    } else {
        println!("Map saved to {}", map_path);
    }
}
