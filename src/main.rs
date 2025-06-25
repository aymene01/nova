mod cli;
mod config;
mod simulation;
mod domain;
mod application;
mod infrastructure;

use crate::simulation::entities::{Map, Station};
use crate::simulation::robot_ai::robot::Robot;
use crate::simulation::robot_ai::types::RobotType;
use crate::simulation::threading::{RobotThreadManager, SharedState, SimulationMessage};

use config::Config;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
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

    let map = Map::new(config.map_width, config.map_height, config.seed);
    let mut station = Station::new(config.map_width / 2, config.map_height / 2);

    station.receive_resource(crate::simulation::entities::ResourceType::Energy, 10000);

    let mut robots = Vec::new();
    for i in 0..config.robots_count {
        let robot_type = match i % 3 {
            0 => RobotType::Explorer,
            1 => RobotType::Harvester,
            _ => RobotType::Scientist,
        };

        let station_pos = station.position();
        let robot = Robot::new(i, robot_type, station_pos.0, station_pos.1, 100);
        robots.push(robot);
    }

    let map_path = format!("map_seed_{}.json", config.seed);
    if let Err(e) = map.save_to_file(&map_path) {
        eprintln!("Failed to save map: {}", e);
    } else {
        println!("Map saved to {}", map_path);
    }

    println!("Simulation running... Press 'q' in TUI to quit");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    let (shared_state, mut message_receiver) = SharedState::new(map, station, robots);
    let thread_manager = RobotThreadManager::new(shared_state.clone(), config.robots_count);

    thread_manager.start_robot_threads().await;

    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

    let mut last_update = Instant::now();

    let result = loop {
        let mut should_shutdown = false;

        while let Ok(message) = message_receiver.try_recv() {
            match message {
                SimulationMessage::RobotAction {
                    robot_id: _,
                    action: _,
                } => {
                    // println!("Robot {} executing action: {:?}", robot_id, action);
                }
                SimulationMessage::ResourceCollected {
                    robot_id: _,
                    position: _,
                } => {
                    // println!("Robot {} collected resource at {:?}", robot_id, position);
                }
                SimulationMessage::ResourceDelivered { robot_id: _ } => {
                    // println!("Robot {} delivered resource to station", robot_id);
                }
                SimulationMessage::RobotRecharged {
                    robot_id: _,
                    energy_amount: _,
                } => {
                    // println!("Robot {} recharged with {} energy", robot_id, energy_amount);
                }
                SimulationMessage::ResourceDiscovered {
                    robot_id: _,
                    position: _,
                    resource_type: _,
                    amount: _,
                } => {
                    // println!(
                    //     "🔍 Robot {} discovered {:?} (amount: {}) at position {:?}",
                    //     robot_id, resource_type, amount, position
                    // );
                }
                SimulationMessage::Shutdown => {
                    should_shutdown = true;
                }
                _ => {}
            }
        }

        if should_shutdown {
            break Ok(());
        }

        if last_update.elapsed() >= Duration::from_millis(500) {
            if let Err(e) = draw_tui(&mut terminal, &shared_state) {
                break Err(e);
            }
            last_update = Instant::now();
        }

        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.code == KeyCode::Char('q') {
                    break Ok(());
                }
            }
        }
    };

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
    shared_state: &SharedState,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::simulation::visualization::MapVisualizer;

    let map = shared_state.get_map();
    let robots = shared_state.get_robots();
    let station = shared_state.get_station();

    let map_guard = map.lock().unwrap();
    let robots_guard = robots.lock().unwrap();
    let station_guard = station.lock().unwrap();

    terminal.draw(|f| {
        let app =
            crate::simulation::visualization::App::new(&map_guard, &robots_guard, &station_guard);
        MapVisualizer::ui(f, &app);
    })?;

    Ok(())
}
