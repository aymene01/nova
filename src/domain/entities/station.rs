use crate::application::robot_ai::robot::Robot;
use crate::domain::values::resource::ResourceType;
use std::collections::HashMap;

pub const STATION_RECHARGE_RATE: u32 = 50;

pub struct Station {
    pub resources: HashMap<ResourceType, u32>,
    pub discoveries: u32,
    pub x: usize,
    pub y: usize,
}

impl Station {
    pub fn new(x: usize, y: usize) -> Self {
        let mut resources = HashMap::new();
        resources.insert(ResourceType::Energy, 100);
        Self {
            resources,
            discoveries: 0,
            x,
            y,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn receive_resource(&mut self, resource_type: ResourceType, amount: u32) {
        let resource_type_clone = resource_type.clone();
        *self.resources.entry(resource_type_clone).or_insert(0) += amount;
        if resource_type == ResourceType::ScientificInterest {
            self.discoveries += 1;
        }
    }

    pub fn get_resource_amount(&self, resource_type: &ResourceType) -> u32 {
        *self.resources.get(resource_type).unwrap_or(&0)
    }

    pub fn robot_at_station(&self, robot_position: (usize, usize)) -> bool {
        robot_position == self.position()
    }

    pub fn recharge_robot(&mut self, robot: &mut Robot) -> Result<u32, &'static str> {
        let energy_available = self.get_resource_amount(&ResourceType::Energy);
        if energy_available == 0 {
            return Err("No energy available for recharging");
        }

        let energy_needed = robot.max_energy() - robot.energy();
        if energy_needed == 0 {
            return Err("Robot is already at full energy");
        }

        let energy_to_transfer = std::cmp::min(energy_needed, STATION_RECHARGE_RATE);
        let actual_transfer = std::cmp::min(energy_to_transfer, energy_available);

        robot.recharge(actual_transfer);
        self.resources
            .insert(ResourceType::Energy, energy_available - actual_transfer);

        Ok(actual_transfer)
    }

    pub fn can_recharge(&self) -> bool {
        self.get_resource_amount(&ResourceType::Energy) > 0
    }
}
