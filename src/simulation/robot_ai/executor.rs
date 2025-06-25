use crate::simulation::entities::{Map, Station};
use crate::simulation::robot_ai::pathfinding::Pathfinder;
use crate::simulation::robot_ai::robot::Robot;
use crate::simulation::robot_ai::types::{AnalyzeTask, ExploreTask, HarvestTask, RobotState};
use crate::infrastructure::threading::simulation_runner::SimulationMessage;
use tokio::sync::mpsc::Sender;

pub struct Executor;

impl Executor {
    fn move_towards_target(
        robot: &mut Robot,
        map: &mut Map,
        target_pos: (usize, usize),
    ) -> Result<(), &'static str> {
        let pathfinder = Pathfinder::new();
        if let Some(direction) = pathfinder.get_next_move(robot.position(), target_pos, map) {
            let movement_cost = robot.move_in_direction(direction, map)?;
            robot.consume_energy_for_movement(movement_cost)?;
        } else if let Some(random_direction) =
            Pathfinder::get_safe_random_direction_from_position(map, robot.x, robot.y)
        {
            let movement_cost = robot.move_in_direction(random_direction, map)?;
            robot.consume_energy_for_movement(movement_cost)?;
        }
        Ok(())
    }

    pub fn execute_explore_task(
        robot: &mut Robot,
        map: &mut Map,
        task: ExploreTask,
        message_sender: Option<&Sender<SimulationMessage>>,
    ) -> Result<(), &'static str> {
        robot.set_state(RobotState::Exploring);
        let target_pos: (usize, usize) = task.target_area;
        let current_pos = robot.position();

        if current_pos == target_pos {
            robot.mark_area_as_discovered(map, target_pos, task.radius);

            let start_x = target_pos.0.saturating_sub(task.radius);
            let end_x = (target_pos.0 + task.radius + 1).min(map.width);
            let start_y = target_pos.1.saturating_sub(task.radius);
            let end_y = (target_pos.1 + task.radius + 1).min(map.height);

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let distance = ((x as i32 - target_pos.0 as i32).abs()
                        + (y as i32 - target_pos.1 as i32).abs())
                        as usize;
                    if distance <= task.radius {
                        if let Some((resource_type, amount)) = map.discover_resource_at((x, y)) {
                            if let Some(sender) = message_sender {
                                let _ = sender.try_send(SimulationMessage::ResourceDiscovered {
                                    robot_id: robot.id,
                                    position: (x, y),
                                    resource_type: resource_type.clone(),
                                    amount,
                                });
                            }
                        }
                    }
                }
            }

            return Ok(());
        }
        Executor::move_towards_target(robot, map, target_pos)?;
        if robot.x < map.width && robot.y < map.height {
            map.discovered[robot.y][robot.x] = true;
        }
        Ok(())
    }

    pub fn execute_harvest_task(
        robot: &mut Robot,
        map: &mut Map,
        harvest_task: HarvestTask,
    ) -> Result<(), &'static str> {
        let target_pos = harvest_task.target_position;
        let current_pos = robot.position();
        let has_resource_at_position = map.resources.contains_key(&current_pos);

        let should_harvest = current_pos == target_pos || has_resource_at_position;

        if should_harvest {
            robot.set_state(RobotState::Harvesting);

            if has_resource_at_position {
                robot.collect_resource(map)?;
            }
            return Ok(());
        }

        robot.set_state(RobotState::MovingToResource);
        Executor::move_towards_target(robot, map, target_pos)?;
        Ok(())
    }

    pub fn execute_analyze_task(
        robot: &mut Robot,
        map: &mut Map,
        analyze_task: AnalyzeTask,
    ) -> Result<(), &'static str> {
        robot.set_state(RobotState::Analyzing);
        let target_pos = analyze_task.target_position;
        let current_pos = robot.position();

        if current_pos == target_pos {
            let should_collect = map
                .resources
                .get(&target_pos)
                .map(|(resource_type, _)| {
                    *resource_type == crate::simulation::entities::ResourceType::ScientificInterest
                })
                .unwrap_or(false);
            if should_collect {
                if let Some((resource_type, collected_amount)) = map.resources.remove(&target_pos) {
                    robot.carrying = Some((resource_type, collected_amount));
                }
            }
            return Ok(());
        }
        Executor::move_towards_target(robot, map, target_pos)?;
        Ok(())
    }

    pub fn execute_return_to_station_task(
        robot: &mut Robot,
        map: &mut Map,
        station: &mut Station,
    ) -> Result<(), &'static str> {
        robot.set_state(RobotState::ReturningToStation);
        let station_pos = (station.x, station.y);
        let current_pos = robot.position();

        if current_pos == station_pos {
            if robot.carrying.is_some() {
                robot.deliver_resource(station)?;
            }
            if station.robot_at_station(current_pos) {
                if station.can_recharge() {
                    match station.recharge_robot(robot) {
                        Ok(_energy_given) => {
                            // println!("Robot recharged");
                        }
                        Err(_e) => {
                            // println!("Error recharging robot");
                        }
                    }
                } else {
                    // println!("Station is full, cannot recharge robot");
                }
            }
            robot.set_state(RobotState::Idle);
            return Ok(());
        }
        Executor::move_towards_target(robot, map, station_pos)?;
        Ok(())
    }
}
