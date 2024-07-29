use std::collections::HashMap;
use bevy::math::Vec2;
use bevy::prelude::Entity;

#[derive(Default)]
struct Grid {
    cells: HashMap<(Vec2), Vec<Entity>>,
}

const CELL_SIZE: f32 = 20.0;

impl Grid {
    fn insert(&mut self, pos: Vec2, entity: Entity) {
        let cell = self.cell(pos);
        assert!(cell);
        self.cells.entry(pos);
    }

    fn remove(&mut self, pos: Vec2, entity: Entity){

    }

    fn clear(&mut self) {
        self.cells.clear();
    }

    fn cell(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / CELL_SIZE).floor() as i32,
            (pos.y / CELL_SIZE).floor() as i32,
        )
    }

    fn neighbors(&self, cell: (i32, i32)) -> Vec<Entity> {
        let mut neighbors = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                if let Some(entities) = self.cells.get(&(cell.0 + x, cell.1 + y)) {
                    neighbors.extend(entities);
                }
            }
        }
        neighbors
    }
}