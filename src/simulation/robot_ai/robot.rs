use crate::simulation::entities::{Map, ResourceType, Station};
use crate::simulation::robot_ai::behavior::RobotBehavior;
use crate::simulation::robot_ai::executor::Executor;
use crate::simulation::robot_ai::pathfinding::Pathfinder;
use crate::simulation::robot_ai::types::{Direction, RobotState, RobotType, Task, TaskType};

pub struct Robot {
    pub id: usize,
    pub robot_type: RobotType,
    pub x: usize,
    pub y: usize,
    pub energy: u32,
    pub carrying: Option<(ResourceType, u32)>,
    pub state: RobotState,
    pub behavior: Box<dyn RobotBehavior>,
}

impl Robot {
    pub fn new(id: usize, robot_type: RobotType, x: usize, y: usize, energy: u32) -> Self {
        let behavior = crate::simulation::robot_ai::behavior::create_behavior(&robot_type);
        Self {
            id,
            robot_type,
            x,
            y,
            energy,
            carrying: None,
            state: RobotState::Idle,
            behavior,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn energy(&self) -> u32 {
        self.energy
    }

    pub fn max_energy(&self) -> u32 {
        self.behavior.get_max_energy()
    }

    pub fn energy_consumption_rate(&self) -> u32 {
        self.behavior.get_energy_consumption_rate()
    }

    pub fn robot_type(&self) -> RobotType {
        self.robot_type.clone()
    }

    pub fn state(&self) -> RobotState {
        self.state.clone()
    }

    pub fn set_state(&mut self, new_state: RobotState) {
        self.state = new_state;
    }

    pub fn is_low_energy(&self) -> bool {
        self.energy <= self.behavior.get_low_energy_threshold()
    }

    pub fn consume_energy(&mut self) -> Result<(), &'static str> {
        if self.energy >= self.energy_consumption_rate() {
            self.energy -= self.energy_consumption_rate();
            Ok(())
        } else {
            Err("Insufficient energy")
        }
    }

    pub fn recharge(&mut self, amount: u32) {
        self.energy = self.energy.saturating_add(amount);
    }

    pub fn move_in_direction(
        &mut self,
        direction: Direction,
        map: &Map,
    ) -> Result<(), &'static str> {
        let (dx, dy) = match direction {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        };

        let new_x = (self.x as i32 + dx) as usize;
        let new_y = (self.y as i32 + dy) as usize;

        // Check for negative values (underflow) and map bounds
        if (self.x as i32 + dx) < 0
            || (self.y as i32 + dy) < 0
            || new_x >= map.width
            || new_y >= map.height
        {
            return Err("Move out of bounds");
        }

        // Additional safety check for terrain traversability
        if map.terrain[new_y][new_x] != 0 {
            return Err("Move blocked by terrain");
        }

        self.x = new_x;
        self.y = new_y;
        Ok(())
    }

    pub fn collect_resource(&mut self, map: &mut Map) -> Result<(), &'static str> {
        if self.carrying.is_some() {
            return Err("Already carrying a resource");
        }

        if let Some(resource) = map.resources.remove(&(self.x, self.y)) {
            self.carrying = Some(resource);
            Ok(())
        } else {
            Err("No resource found at position")
        }
    }

    pub fn deliver_resource(&mut self, station: &mut Station) -> Result<(), &'static str> {
        if self.carrying.is_none() {
            return Err("Not carrying a resource");
        }

        station.receive_resource(
            self.carrying.as_ref().unwrap().0.clone(),
            self.carrying.as_ref().unwrap().1,
        );
        self.carrying = None;
        Ok(())
    }

    pub fn energy_needed_to_return_to_station(&self, station: &Station) -> u32 {
        let station_pos = (station.x, station.y);
        let distance = Pathfinder::manhattan_distance_to(self.position(), station_pos);
        let energy_per_move = self.energy_consumption_rate();
        distance * energy_per_move
    }

    pub fn can_perform_task_and_return(&self, station: &Station) -> bool {
        let energy_for_task = self.energy_consumption_rate();
        let energy_for_return = self.energy_needed_to_return_to_station(station);
        let total_energy_needed = energy_for_task + energy_for_return;
        self.energy >= total_energy_needed + 10
    }

    pub fn decide_next_action(&self, map: &Map, station: &Station) -> Option<Task> {
        let action = self.behavior.decide_next_action(self, map, station);
        if action.is_some() && !self.can_perform_task_and_return(station) {
            return Some(Task {
                task_type: TaskType::ReturnToStation,
                target_position: Some((station.x, station.y)),
                priority: 10,
            });
        }
        action
    }

    pub fn execute_action(
        &mut self,
        map: &mut Map,
        station: &mut Station,
        action: Option<Task>,
    ) -> Result<(), &'static str> {
        if let Some(task) = action {
            self.consume_energy()?;
            match task.task_type {
                TaskType::Explore(explore_task) => {
                    Executor::execute_explore_task(self, map, explore_task)
                }
                TaskType::Harvest(harvest_task) => {
                    Executor::execute_harvest_task(self, map, harvest_task)
                }
                TaskType::Analyze(analyze_task) => {
                    Executor::execute_analyze_task(self, map, analyze_task)
                }
                TaskType::ReturnToStation => {
                    Executor::execute_return_to_station_task(self, map, station)
                }
            }
        } else {
            // No action to execute, just consume minimal energy for being idle
            Ok(())
        }
    }

    pub fn mark_area_as_discovered(&self, map: &mut Map, center: (usize, usize), radius: usize) {
        let center_x = center.0 as i32;
        let center_y = center.1 as i32;

        for dx in -(radius as i32)..=(radius as i32) {
            for dy in -(radius as i32)..=(radius as i32) {
                let x = center_x + dx;
                let y = center_y + dy;

                if x >= 0 && y >= 0 && x < map.width as i32 && y < map.height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    map.discovered[y][x] = true;
                }
            }
        }
    }
}
