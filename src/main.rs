mod position;
mod resource;
mod cell;
mod map;
mod robot;
mod base;

use map::Map;
use base::Base;
use robot::{Robot, RobotMessage};

fn main() {
    let map = Map::new(80, 30);

    // Création de la base centrale
    let mut base = Base::new(map.base_position);

    // Création de plusieurs éclaireurs à la base
    let mut robots = vec![
        Robot::new_scout(1, base.position),
        Robot::new_scout(2, base.position),
        Robot::new_scout(3, base.position),
    ];

    println!("État initial de la carte :");
    map.print_debug();
    println!("--- Début de la simulation ---");

    // Simulation de 300 tours
    for turn in 1..=300 {
        // Déplacement et actions
        for robot in &mut robots {
            let (messages, wants_sync) = robot.act(&map, base.position);
            
            // Traitement immédiat des messages par la base
            if !messages.is_empty() {
                base.process_messages(messages, turn);
            }

            // Synchronisation événementielle (Delta Updates)
            if wants_sync {
                let current_version = base.get_current_version();
                let new_knowledge = base.get_knowledge_since(robot.knowledge_version);
                robot.apply_knowledge(new_knowledge, current_version);
            }
        }
    }

    println!("--- Fin de la simulation ---");
    println!("Total des obstacles découverts par la base : {}", base.known_obstacles.len());
    println!("Total des ressources découvertes par la base : {}", base.known_resources.len());
    println!("Stock d'énergie : {}", base.energy);
    println!("Stock de cristaux : {}", base.crystals);
}
