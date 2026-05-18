mod position;
mod resource;
mod cell;
mod map;

use map::Map;

fn main() {
    let map = Map::new(80, 30);

    map.print_debug();
}