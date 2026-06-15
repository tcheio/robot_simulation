mod position;
mod resource;
mod cell;
mod map;
mod robot;
mod base;
mod collector;

use map::Map;
use base::Base;
use robot::Robot;
use collector::CollectorRobot;

fn main() {
    let mut map = Map::new(80, 30);

    let mut base = Base::new(map.base_position);

    let mut scouts = vec![
        Robot::new_scout(1, base.position),
        Robot::new_scout(2, base.position),
        Robot::new_scout(3, base.position),
    ];

    let mut collectors = vec![
        CollectorRobot::new(4, base.position),
        CollectorRobot::new(5, base.position),
        CollectorRobot::new(6, base.position),
    ];

    println!("État initial de la carte :");
    map.print_debug();
    println!("--- Début de la simulation ---");

    for turn in 1..=300 {
        // --- Éclaireurs (lecture seule de la carte) ---
        for scout in &mut scouts {
            let (messages, wants_sync) = scout.act(&map, base.position);

            if !messages.is_empty() {
                base.process_messages(messages, turn);
            }

            if wants_sync {
                let current_version = base.get_current_version();
                let new_knowledge = base.get_knowledge_since(scout.knowledge_version);
                scout.apply_knowledge(new_knowledge, current_version);
            }
        }

        // --- Collecteurs (mutation de la carte pour récolter) ---
        for collector in &mut collectors {
            let known_resources = base.known_resources.clone();
            let (messages, wants_sync) = collector.act(&mut map, base.position, &known_resources);

            if !messages.is_empty() {
                base.process_messages(messages, turn);
            }

            if wants_sync {
                let current_version = base.get_current_version();
                let new_knowledge = base.get_knowledge_since(collector.knowledge_version);
                collector.apply_knowledge(new_knowledge, current_version);
            }
        }
    }

    println!("--- Fin de la simulation ---");
    println!("Obstacles découverts par la base : {}", base.known_obstacles.len());
    println!("Ressources encore connues         : {}", base.known_resources.len());
    println!("Stock d'énergie                   : {}", base.energy);
    println!("Stock de cristaux                 : {}", base.crystals);
}
