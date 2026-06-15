use std::collections::HashSet;

use crate::position::Position;
use crate::resource::ResourceType;
use crate::robot::RobotMessage;

pub struct Base {
    pub position: Position,
    pub known_obstacles: HashSet<Position>,
    pub known_resources: HashSet<Position>,
    pub history: Vec<RobotMessage>,
    pub energy: u32,
    pub crystals: u32,
}

impl Base {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            known_obstacles: HashSet::new(),
            known_resources: HashSet::new(),
            history: Vec::new(),
            energy: 0,
            crystals: 0,
        }
    }

    pub fn process_messages(&mut self, messages: Vec<RobotMessage>, turn: u32) {
        for message in messages {
            match message {
                RobotMessage::ObstacleDiscovered { position } => {
                    if self.known_obstacles.insert(position) {
                        println!("[Tour {}] Base : Nouvel obstacle découvert en {:?}", turn, position);
                        self.history.push(RobotMessage::ObstacleDiscovered { position });
                    }
                }
                RobotMessage::ResourceDiscovered { position, resource_type } => {
                    if self.known_resources.insert(position) {
                        println!("[Tour {}] Base : Nouvelle ressource {:?} découverte en {:?}", turn, resource_type, position);
                        self.history.push(RobotMessage::ResourceDiscovered { position, resource_type });
                    }
                }
            }
        }
    }

    pub fn get_knowledge_since(&self, version: usize) -> &[RobotMessage] {
        if version < self.history.len() {
            &self.history[version..]
        } else {
            &[]
        }
    }

    pub fn get_current_version(&self) -> usize {
        self.history.len()
    }

    pub fn deposit_resource(&mut self, resource_type: ResourceType, amount: u32) {
        match resource_type {
            ResourceType::Energy => self.energy += amount,
            ResourceType::Crystal => self.crystals += amount,
        }
    }
}
