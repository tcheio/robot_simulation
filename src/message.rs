use crate::position::Position;
use crate::resource::ResourceType;

#[derive(Debug, Clone)]
pub enum RobotMessage {
    ResourceDiscovered {
        position: Position,
        resource_type: ResourceType,
    },
    ObstacleDiscovered {
        position: Position,
    },
    ResourceCollected {
        robot_id: usize,
        resource_type: ResourceType,
        amount: u32,
    },
    ResourceDepleted {
        position: Position,
    },
}
