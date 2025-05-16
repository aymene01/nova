use crate::simulation::entities::{Map, ResourceType};
use crate::simulation::map::TerrainType;
use colored::Colorize;

/// Utility for visualizing the map in the terminal
pub struct MapVisualizer;

impl MapVisualizer {
    /// Visualizes the map in the terminal
    pub fn visualize(map: &Map) {
        println!("Map {}x{} (seed: {})", map.width, map.height, map.seed);
        println!("Legend:");
        println!("  {} - Plain", ".".green());
        println!("  {} - Hill", "^".yellow());
        println!("  {} - Mountain", "▲".red());
        println!("  {} - Canyon", "#".purple());
        println!("  {} - Energy", "E".bright_green());
        println!("  {} - Mineral", "M".bright_blue());
        println!("  {} - Scientific Interest", "S".bright_cyan());
        println!();

        // Print column indices
        print!("   ");
        for x in 0..map.width {
            if x < 10 {
                print!(" {} ", x);
            } else {
                print!("{} ", x);
            }
        }
        println!();

        // Print top border
        print!("   ");
        for _ in 0..map.width {
            print!("---");
        }
        println!();

        // Print the map
        for y in 0..map.height {
            // Print row index
            if y < 10 {
                print!(" {} |", y);
            } else {
                print!("{} |", y);
            }

            for x in 0..map.width {
                let terrain_type = TerrainType::from(map.terrain[y][x]);
                let resource = map.resources.get(&(x, y));

                if let Some((resource_type, _)) = resource {
                    // Print resource symbol
                    match resource_type {
                        ResourceType::Energy => print!(" {} ", "E".bright_green()),
                        ResourceType::Mineral => print!(" {} ", "M".bright_blue()),
                        ResourceType::ScientificInterest => print!(" {} ", "S".bright_cyan()),
                    }
                } else {
                    // Print terrain symbol
                    match terrain_type {
                        TerrainType::Plain => print!(" {} ", ".".green()),
                        TerrainType::Hill => print!(" {} ", "^".yellow()),
                        TerrainType::Mountain => print!(" {} ", "▲".red()),
                        TerrainType::Canyon => print!(" {} ", "#".purple()),
                    }
                }
            }

            println!("|");
        }

        // Print bottom border
        print!("   ");
        for _ in 0..map.width {
            print!("---");
        }
        println!();

        // Print resource statistics
        let mut energy_count = 0;
        let mut mineral_count = 0;
        let mut scientific_count = 0;

        for ((_, _), (res_type, amount)) in &map.resources {
            match res_type {
                ResourceType::Energy => energy_count += amount,
                ResourceType::Mineral => mineral_count += amount,
                ResourceType::ScientificInterest => scientific_count += amount,
            }
        }

        println!("\nResource statistics:");
        println!("  Energy: {} units", energy_count);
        println!("  Minerals: {} units", mineral_count);
        println!("  Scientific Interest: {} units", scientific_count);
    }
}
