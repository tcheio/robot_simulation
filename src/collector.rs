use std::collections::{HashMap, VecDeque};

use crate::base::KnownResource;
use crate::cell::CellType;
use crate::map::Map;
use crate::position::Position;
use crate::resource::ResourceType;
use crate::message::RobotMessage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectorState {
    Idle,
    MovingToResource,
    Collecting,
    ReturningToBase,
    Unloading,
}

pub struct CollectorRobot {
    pub id: usize,
    pub position: Position,
    pub state: CollectorState,
    pub carried_resource: Option<ResourceType>,
    pub target: Option<Position>,
    pub knowledge_version: usize,
    path: Vec<Position>,
}

impl CollectorRobot {
    pub fn new(id: usize, start_pos: Position) -> Self {
        Self {
            id,
            position: start_pos,
            state: CollectorState::Idle,
            carried_resource: None,
            target: None,
            knowledge_version: 0,
            path: Vec::new(),
        }
    }

    pub fn act(
        &mut self,
        map: &mut Map,
        base_position: Position,
        known_resources: &[KnownResource],
    ) -> (Vec<RobotMessage>, bool) {
        let mut messages = Vec::new();
        let mut wants_sync = false;

        match self.state {
            CollectorState::Idle => {
                if let Some(resource) = known_resources.first() {
                    let target = resource.position;
                    self.target = Some(target);
                    if self.position == target {
                        self.state = CollectorState::Collecting;
                    } else {
                        self.path = find_path(self.position, target, map).unwrap_or_default();
                        if self.path.is_empty() {
                            self.target = None;
                        } else {
                            self.state = CollectorState::MovingToResource;
                        }
                    }
                }
            }

            CollectorState::MovingToResource => {
                if let Some(target) = self.target {
                    if self.position == target {
                        self.state = CollectorState::Collecting;
                    } else if !self.path.is_empty() {
                        let next = self.path.remove(0);
                        self.position = next;
                        if self.position == target {
                            self.state = CollectorState::Collecting;
                        }
                    } else {
                        // Chemin épuisé sans atteindre la cible : recalcul
                        self.path = find_path(self.position, target, map).unwrap_or_default();
                        if self.path.is_empty() {
                            // Cible inatteignable, retour en Idle
                            self.state = CollectorState::Idle;
                            self.target = None;
                        }
                    }
                } else {
                    self.state = CollectorState::Idle;
                }
            }

            CollectorState::Collecting => {
                if let Some(target) = self.target {
                    let collected = Self::try_collect(map, target);
                    match collected {
                        Some((res_type, depleted)) => {
                            self.carried_resource = Some(res_type);
                            if depleted {
                                messages.push(RobotMessage::ResourceDepleted { position: target });
                            }
                            self.path = find_path(self.position, base_position, map).unwrap_or_default();
                            self.state = CollectorState::ReturningToBase;
                        }
                        None => {
                            // La ressource a déjà été collectée par un autre robot
                            self.state = CollectorState::Idle;
                            self.target = None;
                        }
                    }
                } else {
                    self.state = CollectorState::Idle;
                }
            }

            CollectorState::ReturningToBase => {
                if self.position == base_position {
                    self.state = CollectorState::Unloading;
                } else if !self.path.is_empty() {
                    let next = self.path.remove(0);
                    self.position = next;
                    if self.position == base_position {
                        self.state = CollectorState::Unloading;
                    }
                } else {
                    // Recalcul du chemin vers la base
                    self.path = find_path(self.position, base_position, map).unwrap_or_default();
                }
            }

            CollectorState::Unloading => {
                if let Some(res_type) = self.carried_resource.take() {
                    messages.push(RobotMessage::ResourceCollected {
                        robot_id: self.id,
                        resource_type: res_type,
                        amount: 1,
                    });
                }
                self.target = None;
                self.path.clear();
                self.state = CollectorState::Idle;
                wants_sync = true;
            }
        }

        (messages, wants_sync)
    }

    fn try_collect(map: &mut Map, target: Position) -> Option<(ResourceType, bool)> {
        // Phase 1 : lire et décrémenter la quantité (le borrow de `cell` se termine à la fin du bloc)
        let (res_type, depleted) = {
            let cell = map.get_cell_mut(target)?;
            match &mut cell.cell_type {
                CellType::Resource(resource) => {
                    let rt = resource.resource_type;
                    resource.quantity -= 1;
                    (rt, resource.quantity == 0)
                }
                _ => return None,
            }
        };

        // Phase 2 : si épuisée, vider la cellule (nouveau borrow indépendant)
        if depleted {
            if let Some(cell) = map.get_cell_mut(target) {
                cell.cell_type = CellType::Empty;
            }
        }

        Some((res_type, depleted))
    }

    pub fn apply_knowledge(&mut self, new_knowledge: &[RobotMessage], new_version: usize) {
        for msg in new_knowledge {
            if let RobotMessage::ResourceDepleted { position } = msg {
                // Si notre cible a été épuisée par un autre robot pendant le trajet, on annule
                if self.target == Some(*position)
                    && matches!(self.state, CollectorState::MovingToResource)
                {
                    self.state = CollectorState::Idle;
                    self.target = None;
                    self.path.clear();
                }
            }
        }
        self.knowledge_version = new_version;
    }
}

fn find_path(start: Position, goal: Position, map: &Map) -> Option<Vec<Position>> {
    if start == goal {
        return Some(vec![]);
    }

    let mut queue = VecDeque::new();
    // came_from[pos] = parent; start maps to itself as sentinel
    let mut came_from: HashMap<Position, Position> = HashMap::new();

    queue.push_back(start);
    came_from.insert(start, start);

    while let Some(current) = queue.pop_front() {
        if current == goal {
            // Reconstruction du chemin depuis goal jusqu'à start
            let mut path = Vec::new();
            let mut pos = current;
            while pos != start {
                path.push(pos);
                pos = came_from[&pos];
            }
            path.reverse();
            return Some(path);
        }

        for neighbor in map.get_neighbors(current) {
            if !came_from.contains_key(&neighbor) {
                came_from.insert(neighbor, current);
                queue.push_back(neighbor);
            }
        }
    }

    None
}
