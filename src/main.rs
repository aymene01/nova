mod cli;
mod config;
mod simulation;

use crate::simulation::entities::{Direction, Map, Robot, RobotType, Station};
use config::Config;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Some(cfg) = cli::args::parse_args() {
        start_simulation(cfg).await;
    } else {
        println!("use cmd start to start")
    }
}

async fn start_simulation(config: Config) {
    println!("Starting simulation with:");
    println!("  Seed: {}", config.seed);
    println!("  Map: {}x{}", config.map_width, config.map_height);
    println!("  Robots: {}", config.robots_count);

    // Create map and station
    let map = Map::new(config.map_width, config.map_height, config.seed);
    let mut station = Station::new(config.map_width / 2, config.map_height / 2);

    // Give station initial energy for recharging robots
    station.receive_resource(crate::simulation::entities::ResourceType::Energy, 10000);

    // Create robots positioned around the station
    let mut robots = Vec::new();
    for i in 0..config.robots_count {
        let robot_type = match i % 3 {
            0 => RobotType::Explorer,
            1 => RobotType::Harvester,
            _ => RobotType::Scientist,
        };

        // Start robots in a large circle around the station
        let station_pos = station.position();
        let angle = (i as f64) * 2.0 * std::f64::consts::PI / (config.robots_count as f64);
        let radius = 8.0;

        let robot_x = (station_pos.0 as f64 + radius * angle.cos()).round() as usize;
        let robot_y = (station_pos.1 as f64 + radius * angle.sin()).round() as usize;

        // Ensure robot position is within map bounds with padding
        let robot_x = robot_x.min(config.map_width - 2).max(1);
        let robot_y = robot_y.min(config.map_height - 2).max(1);

        let robot = Robot::new(i, robot_type, robot_x, robot_y, 100);
        robots.push(robot);
    }

    // Save the initial map
    let map_path = format!("map_seed_{}.json", config.seed);
    if let Err(e) = map.save_to_file(&map_path) {
        eprintln!("Failed to save map: {}", e);
    } else {
        println!("Map saved to {}", map_path);
    }

    println!("Simulation running... Press 'q' in TUI to quit");
    tokio::time::sleep(Duration::from_millis(2000)).await; // Give user time to read

    // Setup persistent TUI
    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

    // Simple simulation loop with RANDOM MOVEMENT + PERSISTENT TUI
    let mut last_update = Instant::now();
    let mut rng = rand::rng();

    let result = loop {
        // Update robots every 500ms
        if last_update.elapsed() >= Duration::from_millis(500) {
            // Move each robot randomly
            for robot in &mut robots {
                // MUCH more forgiving recharge system - recharge all robots every tick
                if robot.energy() < 50 {
                    robot.recharge(20); // Direct recharge without station dependency
                }

                // Move robot (much lower energy threshold)
                if robot.energy() > 1 {
                    // Always move randomly - no complex pathfinding
                    let direction = match rng.random_range(0..4) {
                        0 => Direction::North,
                        1 => Direction::South,
                        2 => Direction::East,
                        _ => Direction::West,
                    };

                    // Try to move the robot
                    if robot.move_in_direction(direction).is_err() {
                        // If movement failed (hit boundary), try a different direction
                        let fallback_direction = match rng.random_range(0..4) {
                            0 => Direction::North,
                            1 => Direction::South,
                            2 => Direction::East,
                            _ => Direction::West,
                        };
                        let _ = robot.move_in_direction(fallback_direction);
                    }
                }
            }

            last_update = Instant::now();
        }

        // Draw TUI
        if let Err(e) = draw_tui(&mut terminal, &map, &robots) {
            break Err(e);
        }

        // Check for quit key (non-blocking)
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.code == KeyCode::Char('q') {
                    break Ok(());
                }
            }
        }
    };

    // Cleanup TUI
    disable_raw_mode().expect("Failed to disable raw mode");
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .expect("Failed to leave alternate screen");
    terminal.show_cursor().expect("Failed to show cursor");

    match result {
        Ok(()) => println!("Simulation stopped by user"),
        Err(e) => eprintln!("Simulation error: {}", e),
    }
}

fn draw_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    map: &Map,
    robots: &[Robot],
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::simulation::visualization::MapVisualizer;

    terminal.draw(|f| {
        // Use the existing UI function from MapVisualizer
        let app = crate::simulation::visualization::App::new(map, robots);
        MapVisualizer::ui(f, &app);
    })?;

    Ok(())
}
