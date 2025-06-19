use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time;

use crate::simulation::entities::{Map, Station};
use crate::simulation::robot_ai::robot::Robot;
use crate::simulation::robot_ai::types::{RobotState, RobotType};

/// Commands that can be sent to the simulation engine
#[allow(dead_code)]
pub enum SimulationCommand {
    AddRobot(Robot),
    RemoveRobot(usize), // robot ID
    Pause,
    Resume,
    Shutdown,
    GetStatus,
    GetDetailedMetrics,
    GetRobots,
    SetTickRate(u64), // milliseconds per tick
}

/// Status information about the simulation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SimulationStatus {
    pub robots_count: usize,
    pub active_robots: usize,
    pub total_energy_collected: u32,
    pub total_minerals_collected: u32,
    pub total_discoveries: u32,
    pub simulation_ticks: u64,
    pub is_running: bool,
}

/// Detailed metrics for monitoring and performance analysis
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SimulationMetrics {
    pub status: SimulationStatus,
    pub performance: PerformanceMetrics,
    pub robot_distribution: RobotDistribution,
    pub resource_stats: ResourceStatistics,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PerformanceMetrics {
    pub avg_tick_duration_ms: f64,
    pub ticks_per_second: f64,
    pub total_runtime_seconds: f64,
    pub memory_usage_estimate: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RobotDistribution {
    pub explorers: usize,
    pub harvesters: usize,
    pub scientists: usize,
    pub idle_robots: usize,
    pub working_robots: usize,
    pub returning_robots: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResourceStatistics {
    pub energy_per_tick: f64,
    pub minerals_per_tick: f64,
    pub discoveries_per_tick: f64,
    pub station_energy: u32,
    pub station_minerals: u32,
    pub station_discoveries: u32,
}

/// Concurrent simulation engine that manages robot processing in parallel
#[allow(dead_code)]
pub struct SimulationEngine {
    map: Arc<RwLock<Map>>,
    station: Arc<Mutex<Station>>,
    robots: Arc<Mutex<Vec<Robot>>>,
    command_rx: mpsc::Receiver<SimulationCommand>,
    status_tx: mpsc::Sender<SimulationStatus>,
    robots_tx: mpsc::Sender<Vec<usize>>,
    is_running: Arc<Mutex<bool>>,
    tick_count: Arc<Mutex<u64>>,
    start_time: std::time::Instant,
    last_metrics: Arc<Mutex<Option<SimulationMetrics>>>,
}

impl SimulationEngine {
    /// Create a new simulation engine
    #[allow(dead_code)]
    pub fn new(
        map: Map,
        station: Station,
        robots: Vec<Robot>,
    ) -> (
        Self,
        mpsc::Sender<SimulationCommand>,
        mpsc::Receiver<SimulationStatus>,
        mpsc::Receiver<Vec<usize>>,
    ) {
        let (command_tx, command_rx) = mpsc::channel(100);
        let (status_tx, status_rx) = mpsc::channel(10);
        let (robots_tx, robots_rx) = mpsc::channel(10);

        let engine = Self {
            map: Arc::new(RwLock::new(map)),
            station: Arc::new(Mutex::new(station)),
            robots: Arc::new(Mutex::new(robots)),
            command_rx,
            status_tx,
            robots_tx,
            is_running: Arc::new(Mutex::new(false)),
            tick_count: Arc::new(Mutex::new(0)),
            start_time: std::time::Instant::now(),
            last_metrics: Arc::new(Mutex::new(None)),
        };

        (engine, command_tx, status_rx, robots_rx)
    }

    /// Start the simulation engine
    #[allow(dead_code)]
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Starting simulation engine");
        *self.is_running.lock().await = true;

        let mut tick_interval = time::interval(Duration::from_millis(100)); // 10 FPS

        loop {
            tokio::select! {
                // Handle commands
                Some(command) = self.command_rx.recv() => {
                    match command {
                        SimulationCommand::Shutdown => {
                            log::info!("Shutting down simulation engine");
                            break;
                        }
                        SimulationCommand::Pause => {
                            *self.is_running.lock().await = false;
                            log::info!("Simulation paused");
                        }
                        SimulationCommand::Resume => {
                            *self.is_running.lock().await = true;
                            log::info!("Simulation resumed");
                        }
                        SimulationCommand::AddRobot(robot) => {
                            self.robots.lock().await.push(robot);
                            log::info!("Robot added to simulation");
                        }
                        SimulationCommand::RemoveRobot(id) => {
                            let mut robots = self.robots.lock().await;
                            robots.retain(|r| r.id != id);
                            log::info!("Robot {} removed from simulation", id);
                        }
                        SimulationCommand::GetStatus => {
                            let status = self.get_status().await;
                            let _ = self.status_tx.send(status).await;
                        }
                        SimulationCommand::GetDetailedMetrics => {
                            let metrics = self.get_detailed_metrics().await;
                            let _ = self.status_tx.send(metrics.status).await;
                        }
                        SimulationCommand::GetRobots => {
                            let robots_guard = self.robots.lock().await;
                            let robot_ids: Vec<usize> = robots_guard.iter().map(|r| r.id).collect();
                            let _ = self.robots_tx.send(robot_ids).await;
                        }
                        SimulationCommand::SetTickRate(rate) => {
                            tick_interval = time::interval(Duration::from_millis(rate));
                        }
                    }
                }

                // Simulation tick
                _ = tick_interval.tick() => {
                    if *self.is_running.lock().await {
                        self.process_simulation_tick().await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process one simulation tick - update all robots concurrently
    async fn process_simulation_tick(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get robot IDs to process
        let robot_ids: Vec<usize> = {
            let robots_guard = self.robots.lock().await;
            robots_guard.iter().map(|r| r.id).collect()
        };

        if robot_ids.is_empty() {
            return Ok(());
        }

        // Process robots by ID to avoid borrowing issues
        let batch_size = (robot_ids.len() / 4).max(1);
        let mut handles = Vec::new();

        for chunk in robot_ids.chunks(batch_size) {
            let map = Arc::clone(&self.map);
            let station = Arc::clone(&self.station);
            let robots_store = Arc::clone(&self.robots);
            let chunk_ids = chunk.to_vec();

            let handle = tokio::spawn(async move {
                Self::process_robot_batch_by_ids(&chunk_ids, map, station, robots_store).await
            });

            handles.push(handle);
        }

        // Wait for all batches to complete
        for handle in handles {
            handle.await??;
        }

        // Increment tick count
        *self.tick_count.lock().await += 1;

        Ok(())
    }

    /// Process a batch of robots by their IDs
    async fn process_robot_batch_by_ids(
        robot_ids: &[usize],
        map: Arc<RwLock<Map>>,
        station: Arc<Mutex<Station>>,
        robots_store: Arc<Mutex<Vec<Robot>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for &robot_id in robot_ids {
            // Get the robot from the store
            let mut robots_guard = robots_store.lock().await;
            if let Some(robot) = robots_guard.iter_mut().find(|r| r.id == robot_id) {
                // Get write lock for map and lock station
                let mut map_guard = map.write().await;
                let mut station_guard = station.lock().await;

                // Get the next action for the robot
                let action = robot.decide_next_action(&map_guard, &station_guard);

                // Execute the action
                if let Err(e) = robot.execute_action(&mut map_guard, &mut station_guard, action) {
                    log::warn!("Robot {} action failed: {}", robot_id, e);
                }
            }
        }

        Ok(())
    }

    /// Get current simulation status
    async fn get_status(&self) -> SimulationStatus {
        let robots = self.robots.lock().await;
        let station = self.station.lock().await;
        let tick_count = *self.tick_count.lock().await;
        let is_running = *self.is_running.lock().await;

        let active_robots = robots
            .iter()
            .filter(|r| !matches!(r.state(), RobotState::Idle))
            .count();

        SimulationStatus {
            robots_count: robots.len(),
            active_robots,
            total_energy_collected: station
                .get_resource_amount(&crate::simulation::entities::ResourceType::Energy),
            total_minerals_collected: station
                .get_resource_amount(&crate::simulation::entities::ResourceType::Mineral),
            total_discoveries: station.discoveries,
            simulation_ticks: tick_count,
            is_running,
        }
    }

    /// Get detailed metrics for monitoring and performance analysis
    async fn get_detailed_metrics(&self) -> SimulationMetrics {
        let status = self.get_status().await;
        let performance = self.calculate_performance_metrics().await;
        let robot_distribution = self.calculate_robot_distribution().await;
        let resource_stats = self.calculate_resource_statistics().await;

        SimulationMetrics {
            status,
            performance,
            robot_distribution,
            resource_stats,
        }
    }

    /// Calculate performance metrics
    async fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        let tick_count = *self.tick_count.lock().await;
        let runtime = self.start_time.elapsed();
        let runtime_seconds = runtime.as_secs_f64();

        let ticks_per_second = if runtime_seconds > 0.0 {
            tick_count as f64 / runtime_seconds
        } else {
            0.0
        };

        let avg_tick_duration_ms = if tick_count > 0 {
            runtime.as_millis() as f64 / tick_count as f64
        } else {
            0.0
        };

        // Rough memory estimate based on robot count and map size
        let robots = self.robots.lock().await;
        let memory_estimate = robots.len() * 256 + 1024 * 1024; // rough estimate

        PerformanceMetrics {
            avg_tick_duration_ms,
            ticks_per_second,
            total_runtime_seconds: runtime_seconds,
            memory_usage_estimate: memory_estimate,
        }
    }

    /// Calculate robot distribution
    async fn calculate_robot_distribution(&self) -> RobotDistribution {
        let robots = self.robots.lock().await;

        let mut explorers = 0;
        let mut harvesters = 0;
        let mut scientists = 0;
        let mut idle_robots = 0;
        let mut working_robots = 0;
        let mut returning_robots = 0;

        for robot in robots.iter() {
            match robot.robot_type() {
                RobotType::Explorer => explorers += 1,
                RobotType::Harvester => harvesters += 1,
                RobotType::Scientist => scientists += 1,
            }

            match robot.state() {
                RobotState::Idle => idle_robots += 1,
                RobotState::Exploring | RobotState::MovingToResource | RobotState::Harvesting => {
                    working_robots += 1
                }
                RobotState::ReturningToStation => returning_robots += 1,
                _ => (),
            }
        }

        RobotDistribution {
            explorers,
            harvesters,
            scientists,
            idle_robots,
            working_robots,
            returning_robots,
        }
    }

    /// Calculate resource statistics
    async fn calculate_resource_statistics(&self) -> ResourceStatistics {
        let station = self.station.lock().await;
        let tick_count = *self.tick_count.lock().await;

        let station_energy =
            station.get_resource_amount(&crate::simulation::entities::ResourceType::Energy);
        let station_minerals =
            station.get_resource_amount(&crate::simulation::entities::ResourceType::Mineral);
        let station_discoveries = station.discoveries;

        // Calculate rates based on current totals and ticks
        let energy_per_tick = if tick_count > 0 {
            station_energy as f64 / tick_count as f64
        } else {
            0.0
        };

        let minerals_per_tick = if tick_count > 0 {
            station_minerals as f64 / tick_count as f64
        } else {
            0.0
        };

        let discoveries_per_tick = if tick_count > 0 {
            station_discoveries as f64 / tick_count as f64
        } else {
            0.0
        };

        ResourceStatistics {
            energy_per_tick,
            minerals_per_tick,
            discoveries_per_tick,
            station_energy,
            station_minerals,
            station_discoveries,
        }
    }
}

/// Helper to create a basic simulation setup
#[allow(dead_code)]
pub async fn create_basic_simulation(
    map_width: usize,
    map_height: usize,
    seed: u64,
    num_robots: usize,
) -> (
    SimulationEngine,
    mpsc::Sender<SimulationCommand>,
    mpsc::Receiver<SimulationStatus>,
    mpsc::Receiver<Vec<usize>>,
) {
    use crate::simulation::entities::Map;

    let map = Map::new(map_width, map_height, seed);
    let station = Station::new(map_width / 2, map_height / 2);

    let mut robots = Vec::new();
    for i in 0..num_robots {
        let robot_type = match i % 3 {
            0 => RobotType::Explorer,
            1 => RobotType::Harvester,
            _ => RobotType::Scientist,
        };

        // Start robots in a circle around the station
        let station_pos = station.position();
        let angle = (i as f64) * 2.0 * std::f64::consts::PI / (num_robots as f64);
        let radius = 2.0; // Start robots 2 tiles away from station

        let robot_x = (station_pos.0 as f64 + radius * angle.cos()).round() as usize;
        let robot_y = (station_pos.1 as f64 + radius * angle.sin()).round() as usize;

        // Ensure robot position is within map bounds
        let robot_x = robot_x.min(map_width - 1);
        let robot_y = robot_y.min(map_height - 1);

        let robot = Robot::new(i, robot_type, robot_x, robot_y, 100);
        robots.push(robot);
    }

    SimulationEngine::new(map, station, robots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn simulation_engine_starts_and_stops() {
        let (mut engine, command_tx, _status_rx, _robots_rx) =
            create_basic_simulation(10, 10, 42, 3).await;

        // Start engine in background
        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Send shutdown command
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();

        // Engine should stop gracefully
        let result = timeout(Duration::from_secs(1), engine_handle).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn simulation_processes_status_requests() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(10, 10, 42, 3).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Request status
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();

        // Should receive status
        let status = timeout(Duration::from_millis(100), status_rx.recv()).await;
        assert!(status.is_ok());

        let status = status.unwrap().unwrap();
        assert_eq!(status.robots_count, 3);

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_can_pause_and_resume() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(10, 10, 42, 3).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Pause simulation
        command_tx.send(SimulationCommand::Pause).await.unwrap();

        // Check status
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();
        assert!(!status.is_running);

        // Resume simulation
        command_tx.send(SimulationCommand::Resume).await.unwrap();

        // Check status again
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();
        assert!(status.is_running);

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_processes_robots_concurrently() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(20, 20, 123, 12).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Let simulation run for a bit
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check that robots are being processed
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();

        assert_eq!(status.robots_count, 12);
        assert!(status.simulation_ticks > 0); // Should have processed some ticks

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_handles_robot_addition_and_removal() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(10, 10, 42, 2).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Check initial robot count
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();
        assert_eq!(status.robots_count, 2);

        // Add a robot
        let new_robot = Robot::new(99, RobotType::Scientist, 5, 5, 100);
        command_tx
            .send(SimulationCommand::AddRobot(new_robot))
            .await
            .unwrap();

        // Check robot count increased
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();
        assert_eq!(status.robots_count, 3);

        // Remove a robot
        command_tx
            .send(SimulationCommand::RemoveRobot(99))
            .await
            .unwrap();

        // Check robot count decreased
        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();
        assert_eq!(status.robots_count, 2);

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_handles_concurrent_load() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(30, 30, 456, 20).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Send multiple status requests concurrently
        let mut handles = Vec::new();
        for _ in 0..10 {
            let tx = command_tx.clone();
            let handle = tokio::spawn(async move {
                tx.send(SimulationCommand::GetStatus).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Should receive all status responses
        for _ in 0..10 {
            let status = timeout(Duration::from_millis(100), status_rx.recv()).await;
            assert!(status.is_ok());
        }

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_maintains_performance_under_load() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(50, 50, 789, 50).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        let start_time = std::time::Instant::now();

        // Let simulation run for a reasonable time
        tokio::time::sleep(Duration::from_millis(500)).await;

        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();

        let elapsed = start_time.elapsed();

        // Should have processed a reasonable number of ticks
        assert!(status.simulation_ticks > 0);
        assert_eq!(status.robots_count, 50);

        // Performance check: should process at least 1 tick per 100ms
        let expected_min_ticks = elapsed.as_millis() / 100;
        assert!(status.simulation_ticks >= expected_min_ticks as u64);

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_provides_detailed_metrics() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(15, 15, 999, 6).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Let simulation run to generate some data
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Request detailed metrics
        command_tx
            .send(SimulationCommand::GetDetailedMetrics)
            .await
            .unwrap();
        let status = status_rx.recv().await.unwrap();

        // Should have basic status
        assert_eq!(status.robots_count, 6);
        assert!(status.simulation_ticks > 0);

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_handles_dynamic_tick_rate() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(10, 10, 42, 3).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Change tick rate to slower
        command_tx
            .send(SimulationCommand::SetTickRate(200))
            .await
            .unwrap();

        let start_time = std::time::Instant::now();
        tokio::time::sleep(Duration::from_millis(300)).await;

        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status = status_rx.recv().await.unwrap();

        let elapsed = start_time.elapsed().as_millis();
        // With 200ms tick rate, should have fewer ticks than with 100ms rate
        let expected_max_ticks = elapsed / 200 + 1; // +1 for timing tolerance
        assert!(status.simulation_ticks <= expected_max_ticks as u64);

        // Change back to faster rate
        command_tx
            .send(SimulationCommand::SetTickRate(50))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        command_tx.send(SimulationCommand::GetStatus).await.unwrap();
        let status2 = status_rx.recv().await.unwrap();

        // Should have more ticks now
        assert!(status2.simulation_ticks > status.simulation_ticks);

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }

    #[tokio::test]
    async fn simulation_tracks_performance_metrics() {
        let (mut engine, command_tx, mut status_rx, _robots_rx) =
            create_basic_simulation(20, 20, 456, 10).await;

        let engine_handle = tokio::spawn(async move { engine.run().await });

        // Let simulation run for performance tracking
        tokio::time::sleep(Duration::from_millis(400)).await;

        // Request detailed metrics through command channel
        command_tx
            .send(SimulationCommand::GetDetailedMetrics)
            .await
            .unwrap();
        let status = status_rx.recv().await.unwrap();

        // Should have basic status data (detailed metrics not exposed through status channel yet)
        assert_eq!(status.robots_count, 10);
        assert!(status.simulation_ticks > 0);
        // Note: These fields are u32, so they're always >= 0 by definition

        // Shutdown
        command_tx.send(SimulationCommand::Shutdown).await.unwrap();
        let _ = engine_handle.await;
    }
}
