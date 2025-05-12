mod cli;
mod simulation;
mod config;

use config::session::Config;

fn main() {
    if let Some(cfg) = cli::args::parse_args() {
        start_simulation(cfg);
    } else{
        println!("use cmd start to start")
    }

}

fn start_simulation(config: Config) {
    println!("Starting simulation with:");
    println!("  Seed: {}", config.seed);
    println!("  Map: {}x{}", config.map_width, config.map_height);
    println!("  Robots: {}", config.robots_count);
}