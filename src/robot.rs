use rand::Rng;
use std::collections::HashSet;

use crate::position::Position;
use crate::resource::ResourceType;
use crate::map::Map;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RobotType {
    Scout,
    Collector,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RobotState {
    Exploring,
    ReturningToBase,
}

#[derive(Debug, Clone)]
pub enum RobotMessage {
    ResourceDiscovered {
        position: Position,
        resource_type: ResourceType,
    },
    ObstacleDiscovered {
        position: Position,
    },
}

pub struct Robot {
    pub id: usize,
    pub robot_type: RobotType,
    pub state: RobotState,
    pub position: Position,
    pub known_obstacles: HashSet<Position>,
    pub known_resources: HashSet<Position>,
    pub pending_messages: Vec<RobotMessage>,
    pub knowledge_version: usize,
}

impl Robot {
    pub fn new_scout(id: usize, start_pos: Position) -> Self {
        Self {
            id,
            robot_type: RobotType::Scout,
            state: RobotState::Exploring,
            position: start_pos,
            known_obstacles: HashSet::new(),
            known_resources: HashSet::new(),
            pending_messages: Vec::new(),
            knowledge_version: 0,
        }
    }

    pub fn act(&mut self, map: &Map, base_position: Position) -> (Vec<RobotMessage>, bool) {
        let mut messages_for_base = Vec::new();
        let mut wants_sync = false;

        if self.robot_type == RobotType::Scout {
            match self.state {
                RobotState::Exploring => {
                    self.discover_surroundings(map);
                    self.move_randomly(map);
                }
                RobotState::ReturningToBase => {
                    self.move_towards(base_position, map);
                }
            }

            // Vérification physique : peu importe son état, s'il est SUR la base, il agit
            if self.position == base_position {
                if !self.pending_messages.is_empty() {
                    messages_for_base = std::mem::take(&mut self.pending_messages);
                }
                // S'il rentrait, il repart en exploration
                if self.state == RobotState::ReturningToBase {
                    self.state = RobotState::Exploring;
                }
                // Il profite d'être physiquement à la base pour se synchroniser
                wants_sync = true;
            }
        }

        (messages_for_base, wants_sync)
    }

    pub fn apply_knowledge(&mut self, new_knowledge: &[RobotMessage], new_version: usize) {
        for msg in new_knowledge {
            match msg {
                RobotMessage::ObstacleDiscovered { position } => {
                    self.known_obstacles.insert(*position);
                }
                RobotMessage::ResourceDiscovered { position, .. } => {
                    self.known_resources.insert(*position);
                }
            }
        }
        self.knowledge_version = new_version;
    }

    fn discover_surroundings(&mut self, map: &Map) {
        let mut found_new_resource = false;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = self.position.x as isize + dx;
                let ny = self.position.y as isize + dy;
                
                if nx >= 0 && ny >= 0 && nx < map.width as isize && ny < map.height as isize {
                    let pos = Position::new(nx as usize, ny as usize);
                    if let Some(cell) = map.get_cell(pos) {
                        // Découverte d'un obstacle
                        if !cell.is_walkable() && !self.known_obstacles.contains(&pos) {
                            self.known_obstacles.insert(pos);
                            self.pending_messages.push(RobotMessage::ObstacleDiscovered { position: pos });
                        } 
                        // Découverte d'une ressource
                        else if cell.is_resource() && !self.known_resources.contains(&pos) {
                            self.known_resources.insert(pos);
                            if let crate::cell::CellType::Resource(res) = &cell.cell_type {
                                self.pending_messages.push(RobotMessage::ResourceDiscovered {
                                    position: pos,
                                    resource_type: res.resource_type,
                                });
                                found_new_resource = true;
                            }
                        }
                    }
                }
            }
        }

        // S'il a trouvé une nouvelle ressource, il décide de rentrer à la base
        if found_new_resource {
            self.state = RobotState::ReturningToBase;
        }
    }

    fn move_towards(&mut self, target: Position, map: &Map) {
        // Déplacement basique vers la cible sans pathfinding complexe (temporaire jusqu'au step 4)
        let dx = if target.x > self.position.x { 1 } else if target.x < self.position.x { -1 } else { 0 };
        let dy = if target.y > self.position.y { 1 } else if target.y < self.position.y { -1 } else { 0 };
        
        let mut best_pos = self.position;
        
        // Essaie sur l'axe X
        if dx != 0 {
            let nx = (self.position.x as isize + dx) as usize;
            let pos = Position::new(nx, self.position.y);
            if map.is_walkable(pos) && !self.known_obstacles.contains(&pos) {
                best_pos = pos;
            }
        }
        
        // Si bloqué ou pas de dx, essaie sur l'axe Y
        if best_pos == self.position && dy != 0 {
            let ny = (self.position.y as isize + dy) as usize;
            let pos = Position::new(self.position.x, ny);
            if map.is_walkable(pos) && !self.known_obstacles.contains(&pos) {
                best_pos = pos;
            }
        }
        
        if best_pos == self.position {
             // Si bloqué, petit mouvement aléatoire pour se décoincer
             self.move_randomly(map);
        } else {
             self.position = best_pos;
        }
    }

    fn move_randomly(&mut self, map: &Map) {
        let mut rng = rand::thread_rng();
        let directions = [
            (0, -1), // Up
            (0, 1),  // Down
            (-1, 0), // Left
            (1, 0),  // Right
        ];
        
        let mut valid_moves = Vec::new();
        
        for (dx, dy) in directions.iter() {
            let nx = self.position.x as isize + dx;
            let ny = self.position.y as isize + dy;
            
            if nx >= 0 && ny >= 0 && nx < map.width as isize && ny < map.height as isize {
                let pos = Position::new(nx as usize, ny as usize);
                
                // Le robot évite les obstacles connus
                if !self.known_obstacles.contains(&pos) {
                    // Vérification sur la map réelle pour éviter de traverser un obstacle ignoré ou non découvert
                    if map.is_walkable(pos) {
                        valid_moves.push(pos);
                    }
                }
            }
        }
        
        if !valid_moves.is_empty() {
            let choice = rng.gen_range(0..valid_moves.len());
            self.position = valid_moves[choice];
        }
    }
}
