use crate::resource::Resource;

#[derive(Debug, Clone)]
pub enum CellType {
    Empty,
    Obstacle,
    Base,
    Resource(Resource),
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub cell_type: CellType,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            cell_type: CellType::Empty,
        }
    }

    pub fn obstacle() -> Self {
        Self {
            cell_type: CellType::Obstacle,
        }
    }

    pub fn base() -> Self {
        Self {
            cell_type: CellType::Base,
        }
    }

    pub fn resource(resource: Resource) -> Self {
        Self {
            cell_type: CellType::Resource(resource),
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(self.cell_type, CellType::Obstacle)
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.cell_type, CellType::Empty)
    }

    pub fn is_resource(&self) -> bool {
        matches!(self.cell_type, CellType::Resource(_))
    }
}