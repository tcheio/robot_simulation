use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::base::Base;
use crate::collector::{CollectorRobot, CollectorState};
use crate::map::Map;
use crate::message::RobotMessage;
use crate::position::Position;
use crate::robot::Robot;

pub const TICK_DURATION: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotKind {
    Scout,
    Collector,
}

#[derive(Debug, Clone, Copy)]
pub struct RobotView {
    pub kind: RobotKind,
    pub position: Position,
}

pub type SharedMap = Arc<RwLock<Map>>;
pub type SharedBase = Arc<Mutex<Base>>;
pub type SharedRobotViews = Arc<Mutex<HashMap<usize, RobotView>>>;

pub struct Simulation {
    pub map: SharedMap,
    pub base: SharedBase,
    pub robot_views: SharedRobotViews,
    running: Arc<AtomicBool>,
    rx: Receiver<RobotMessage>,
    handles: Vec<JoinHandle<()>>,
    tick: u32,
}

impl Simulation {
    pub fn new(map: Map, base_position: Position, num_scouts: usize, num_collectors: usize) -> Self {
        let map: SharedMap = Arc::new(RwLock::new(map));
        let base: SharedBase = Arc::new(Mutex::new(Base::new(base_position)));
        let robot_views: SharedRobotViews = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::channel();

        let mut handles = Vec::new();
        let mut next_id = 1;

        for _ in 0..num_scouts {
            let id = next_id;
            next_id += 1;
            handles.push(spawn_scout(
                id,
                base_position,
                Arc::clone(&map),
                Arc::clone(&base),
                Arc::clone(&robot_views),
                Arc::clone(&running),
                tx.clone(),
            ));
        }

        for _ in 0..num_collectors {
            let id = next_id;
            next_id += 1;
            handles.push(spawn_collector(
                id,
                base_position,
                Arc::clone(&map),
                Arc::clone(&base),
                Arc::clone(&robot_views),
                Arc::clone(&running),
                tx.clone(),
            ));
        }

        Self {
            map,
            base,
            robot_views,
            running,
            rx,
            handles,
            tick: 0,
        }
    }

    /// Draine les messages reçus des threads robots et met à jour la base.
    pub fn process_messages(&mut self) {
        self.tick += 1;

        let mut incoming = Vec::new();
        while let Ok(message) = self.rx.try_recv() {
            incoming.push(message);
        }

        if !incoming.is_empty() {
            let mut base = self.base.lock().unwrap();
            base.process_messages(incoming, self.tick);
        }
    }

    /// Arrête tous les threads robots et attend leur fin.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}

fn spawn_scout(
    id: usize,
    base_position: Position,
    map: SharedMap,
    base: SharedBase,
    robot_views: SharedRobotViews,
    running: Arc<AtomicBool>,
    tx: Sender<RobotMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut scout = Robot::new_scout(id, base_position);

        while running.load(Ordering::Relaxed) {
            let (messages, wants_sync) = {
                let map_guard = map.read().unwrap();
                scout.act(&map_guard, base_position)
            };

            for message in messages {
                let _ = tx.send(message);
            }

            if wants_sync {
                let (current_version, new_knowledge) = {
                    let base_guard = base.lock().unwrap();
                    let version = base_guard.get_current_version();
                    let knowledge = base_guard.get_knowledge_since(scout.knowledge_version).to_vec();
                    (version, knowledge)
                };
                scout.apply_knowledge(&new_knowledge, current_version);
            }

            {
                let mut views = robot_views.lock().unwrap();
                views.insert(
                    id,
                    RobotView {
                        kind: RobotKind::Scout,
                        position: scout.position,
                    },
                );
            }

            thread::sleep(TICK_DURATION);
        }
    })
}

fn spawn_collector(
    id: usize,
    base_position: Position,
    map: SharedMap,
    base: SharedBase,
    robot_views: SharedRobotViews,
    running: Arc<AtomicBool>,
    tx: Sender<RobotMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut collector = CollectorRobot::new(id, base_position);
        // Position que ce collecteur a réservée auprès de la base, pour éviter
        // que deux collecteurs ne visent inutilement la même ressource.
        let mut reserved: Option<Position> = None;

        while running.load(Ordering::Relaxed) {
            let known_resources = {
                let mut base_guard = base.lock().unwrap();

                if collector.state == CollectorState::Idle && reserved.is_none() {
                    let candidate = base_guard
                        .known_resources
                        .iter()
                        .find(|r| !base_guard.reserved_targets.contains(&r.position))
                        .cloned();

                    if let Some(resource) = &candidate {
                        base_guard.reserved_targets.insert(resource.position);
                        reserved = Some(resource.position);
                    }

                    candidate.into_iter().collect::<Vec<_>>()
                } else {
                    base_guard.known_resources.clone()
                }
            };

            let (messages, wants_sync) = {
                let mut map_guard = map.write().unwrap();
                collector.act(&mut map_guard, base_position, &known_resources)
            };

            for message in messages {
                let _ = tx.send(message);
            }

            if wants_sync {
                let (current_version, new_knowledge) = {
                    let base_guard = base.lock().unwrap();
                    let version = base_guard.get_current_version();
                    let knowledge = base_guard.get_knowledge_since(collector.knowledge_version).to_vec();
                    (version, knowledge)
                };
                collector.apply_knowledge(&new_knowledge, current_version);
            }

            // Le collecteur a abandonné ou déposé sa cible : on libère la réservation.
            if collector.target.is_none() {
                if let Some(pos) = reserved.take() {
                    base.lock().unwrap().reserved_targets.remove(&pos);
                }
            }

            {
                let mut views = robot_views.lock().unwrap();
                views.insert(
                    id,
                    RobotView {
                        kind: RobotKind::Collector,
                        position: collector.position,
                    },
                );
            }

            thread::sleep(TICK_DURATION);
        }

        if let Some(pos) = reserved.take() {
            base.lock().unwrap().reserved_targets.remove(&pos);
        }
    })
}
