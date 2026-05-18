#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Energy,
    Crystal,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub quantity: u32,
}

impl Resource {
    pub fn new(resource_type: ResourceType, quantity: u32) -> Self {
        Self {
            resource_type,
            quantity,
        }
    }
}