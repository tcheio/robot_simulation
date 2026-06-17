use std::collections::{HashSet, VecDeque};

use crate::position::Position;
use crate::resource::ResourceType;
use crate::message::RobotMessage;

const MAX_LOGS: usize = 50;

#[derive(Debug, Clone)]
pub struct KnownResource {
    pub position: Position,
    pub resource_type: ResourceType,
}

pub struct Base {
    pub position: Position,
    pub known_obstacles: HashSet<Position>,
    pub known_resources: Vec<KnownResource>,
    pub reserved_targets: HashSet<Position>,
    pub history: Vec<RobotMessage>,
    pub logs: VecDeque<String>,
    pub energy: u32,
    pub crystals: u32,
}

impl Base {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            known_obstacles: HashSet::new(),
            known_resources: Vec::new(),
            reserved_targets: HashSet::new(),
            history: Vec::new(),
            logs: VecDeque::new(),
            energy: 0,
            crystals: 0,
        }
    }

    fn log(&mut self, turn: u32, text: String) {
        self.logs.push_back(format!("[Tour {}] {}", turn, text));
        if self.logs.len() > MAX_LOGS {
            self.logs.pop_front();
        }
    }

    pub fn process_messages(&mut self, messages: Vec<RobotMessage>, turn: u32) {
        for message in messages {
            match message {
                RobotMessage::ObstacleDiscovered { position } => {
                    if self.known_obstacles.insert(position) {
                        self.log(turn, format!("Nouvel obstacle découvert en {:?}", position));
                        self.history.push(RobotMessage::ObstacleDiscovered { position });
                    }
                }
                RobotMessage::ResourceDiscovered { position, resource_type } => {
                    if !self.known_resources.iter().any(|r| r.position == position) {
                        self.log(turn, format!("Nouvelle ressource {:?} découverte en {:?}", resource_type, position));
                        self.known_resources.push(KnownResource { position, resource_type });
                        self.history.push(RobotMessage::ResourceDiscovered { position, resource_type });
                    }
                }
                RobotMessage::ResourceCollected { robot_id, resource_type, amount } => {
                    match resource_type {
                        ResourceType::Energy => self.energy += amount,
                        ResourceType::Crystal => self.crystals += amount,
                    }
                    self.log(turn, format!("Robot {} a déposé {} unité(s) de {:?} (énergie={}, cristaux={})",
                        robot_id, amount, resource_type, self.energy, self.crystals));
                }
                RobotMessage::ResourceDepleted { position } => {
                    self.known_resources.retain(|r| r.position != position);
                    self.log(turn, format!("Ressource en {:?} épuisée, retirée de la liste", position));
                    self.history.push(RobotMessage::ResourceDepleted { position });
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
}
