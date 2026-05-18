use rand::Rng;
use noise::{NoiseFn, Perlin};

use crate::cell::{Cell, CellType};
use crate::position::Position;
use crate::resource::{Resource, ResourceType};

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
    pub base_position: Position,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![Cell::empty(); width]; height];

        let base_position = Position::new(width / 2, height / 2);

        let mut map = Self {
            width,
            height,
            cells,
            base_position,
        };

        map.generate_obstacles();
        map.place_base();
        map.place_resources(15, 10);

        map
    }

    fn generate_obstacles(&mut self) {
        let perlin = Perlin::new(42);
        let scale = 12.0;
        let threshold = 0.35;

        for y in 0..self.height {
            for x in 0..self.width {
                let nx = x as f64 / scale;
                let ny = y as f64 / scale;

                let noise_value = perlin.get([nx, ny]);

                if noise_value > threshold {
                    self.cells[y][x] = Cell::obstacle();
                }
            }
        }
    }

    fn place_base(&mut self) {
        let pos = self.base_position;

        self.cells[pos.y][pos.x] = Cell::base();

        for dy in -2..=2 {
            for dx in -2..=2 {
                let nx = pos.x as isize + dx;
                let ny = pos.y as isize + dy;

                if self.is_inside_isize(nx, ny) {
                    let clear_pos = Position::new(nx as usize, ny as usize);

                    if clear_pos != pos {
                        self.cells[clear_pos.y][clear_pos.x] = Cell::empty();
                    }
                }
            }
        }

        self.cells[pos.y][pos.x] = Cell::base();
    }

    fn place_resources(&mut self, energy_count: usize, crystal_count: usize) {
        self.place_resource_type(ResourceType::Energy, energy_count);
        self.place_resource_type(ResourceType::Crystal, crystal_count);
    }

    fn place_resource_type(&mut self, resource_type: ResourceType, count: usize) {
        let mut rng = rand::thread_rng();
        let mut placed = 0;
        let max_attempts = count * 100;
        let mut attempts = 0;

        while placed < count && attempts < max_attempts {
            attempts += 1;

            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            let pos = Position::new(x, y);

            if self.can_place_resource(pos) {
                let quantity = rng.gen_range(50..=200);
                let resource = Resource::new(resource_type, quantity);

                self.cells[y][x] = Cell::resource(resource);
                placed += 1;
            }
        }
    }

    fn can_place_resource(&self, pos: Position) -> bool {
        if !self.is_inside(pos) {
            return false;
        }

        if pos == self.base_position {
            return false;
        }

        self.cells[pos.y][pos.x].is_empty()
    }

    pub fn is_inside(&self, pos: Position) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    fn is_inside_isize(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && x < self.width as isize && y < self.height as isize
    }

    pub fn get_cell(&self, pos: Position) -> Option<&Cell> {
        if self.is_inside(pos) {
            Some(&self.cells[pos.y][pos.x])
        } else {
            None
        }
    }

    pub fn get_cell_mut(&mut self, pos: Position) -> Option<&mut Cell> {
        if self.is_inside(pos) {
            Some(&mut self.cells[pos.y][pos.x])
        } else {
            None
        }
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        self.get_cell(pos)
            .map(|cell| cell.is_walkable())
            .unwrap_or(false)
    }

    pub fn is_resource(&self, pos: Position) -> bool {
        self.get_cell(pos)
            .map(|cell| cell.is_resource())
            .unwrap_or(false)
    }

    pub fn get_neighbors(&self, pos: Position) -> Vec<Position> {
        let directions = [
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
        ];

        let mut neighbors = Vec::new();

        for (dx, dy) in directions {
            let nx = pos.x as isize + dx;
            let ny = pos.y as isize + dy;

            if self.is_inside_isize(nx, ny) {
                let next_pos = Position::new(nx as usize, ny as usize);

                if self.is_walkable(next_pos) {
                    neighbors.push(next_pos);
                }
            }
        }

        neighbors
    }

    pub fn print_debug(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let symbol = match &self.cells[y][x].cell_type {
                    CellType::Empty => '.',
                    CellType::Obstacle => 'O',
                    CellType::Base => '#',
                    CellType::Resource(resource) => match resource.resource_type {
                        ResourceType::Energy => 'E',
                        ResourceType::Crystal => 'C',
                    },
                };

                print!("{}", symbol);
            }

            println!();
        }
    }
}