use crate::simulation::entities::{Map, Station};
use crate::simulation::robot_ai::robot::Robot;
use crate::simulation::robot_ai::types::Task;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SimulationMessage {
    RobotMoved {
        robot_id: usize,
        new_position: (usize, usize),
    },
    RobotAction {
        robot_id: usize,
        action: Task,
    },
    ResourceCollected {
        robot_id: usize,
        position: (usize, usize),
    },
    ResourceDelivered {
        robot_id: usize,
    },
    RobotRecharged {
        robot_id: usize,
        energy_amount: u32,
    },
    ResourceDiscovered {
        robot_id: usize,
        position: (usize, usize),
        resource_type: crate::simulation::entities::ResourceType,
        amount: u32,
    },
    UpdateDisplay,
    Shutdown,
}

pub struct SharedState {
    pub map: Arc<Mutex<Map>>,
    pub station: Arc<Mutex<Station>>,
    pub robots: Arc<Mutex<Vec<Robot>>>,
    pub message_sender: Sender<SimulationMessage>,
}

impl SharedState {
    pub fn new(
        map: Map,
        station: Station,
        robots: Vec<Robot>,
    ) -> (Self, Receiver<SimulationMessage>) {
        let (tx, rx) = mpsc::channel(100);

        let shared_state = Self {
            map: Arc::new(Mutex::new(map)),
            station: Arc::new(Mutex::new(station)),
            robots: Arc::new(Mutex::new(robots)),
            message_sender: tx,
        };

        (shared_state, rx)
    }

    pub fn get_map(&self) -> Arc<Mutex<Map>> {
        self.map.clone()
    }

    pub fn get_station(&self) -> Arc<Mutex<Station>> {
        self.station.clone()
    }

    pub fn get_robots(&self) -> Arc<Mutex<Vec<Robot>>> {
        self.robots.clone()
    }

    pub fn get_message_sender(&self) -> Sender<SimulationMessage> {
        self.message_sender.clone()
    }
}

pub struct RobotThreadManager {
    shared_state: SharedState,
    robot_start_times: Vec<Instant>,
}

impl RobotThreadManager {
    pub fn new(shared_state: SharedState, robot_count: usize) -> Self {
        let mut robot_start_times = Vec::new();
        for i in 0..robot_count {
            robot_start_times.push(Instant::now() + Duration::from_millis(i as u64 * 1000));
        }

        Self {
            shared_state,
            robot_start_times,
        }
    }

    pub async fn start_robot_threads(&self) {
        let robots = self.shared_state.get_robots();
        let robots_guard = robots.lock().unwrap();

        for (i, robot) in robots_guard.iter().enumerate() {
            let robot_id = robot.id;
            let shared_state = self.shared_state.clone();
            let start_time = self.robot_start_times[i];

            tokio::spawn(async move {
                Self::robot_worker(robot_id, shared_state, start_time).await;
            });
        }
    }

    async fn robot_worker(robot_id: usize, shared_state: SharedState, start_time: Instant) {
        tokio::time::sleep_until(start_time.into()).await;

        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            let robots = shared_state.get_robots();
            let map = shared_state.get_map();
            let station = shared_state.get_station();
            let message_sender = shared_state.get_message_sender();

            let action = {
                let robots_guard = robots.lock().unwrap();
                let map_guard = map.lock().unwrap();
                let station_guard = station.lock().unwrap();

                if let Some(robot) = robots_guard.iter().find(|r| r.id == robot_id) {
                    robot.decide_next_action(&map_guard, &station_guard)
                } else {
                    None
                }
            };

            if let Some(task) = action {
                let _ = message_sender
                    .send(SimulationMessage::RobotAction {
                        robot_id,
                        action: task.clone(),
                    })
                    .await;

                let mut robots_guard = robots.lock().unwrap();
                let mut map_guard = map.lock().unwrap();
                let mut station_guard = station.lock().unwrap();

                if let Some(robot) = robots_guard.iter_mut().find(|r| r.id == robot_id) {
                    if let Err(e) = robot.execute_action(
                        &mut map_guard,
                        &mut station_guard,
                        Some(task),
                        Some(&message_sender),
                    ) {
                        eprintln!("Robot {} failed to execute action: {}", robot_id, e);
                    }
                }
            }
        }
    }
}

impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            station: self.station.clone(),
            robots: self.robots.clone(),
            message_sender: self.message_sender.clone(),
        }
    }
}
