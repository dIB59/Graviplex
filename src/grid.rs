use bevy::log;
use bevy::prelude::*;
use bevy::reflect::List;
use std::{borrow::BorrowMut, cell, collections::HashMap};

pub struct SpatialGridPlugin;

impl Plugin for SpatialGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_grid);
    }
}

fn setup_grid(mut commands: Commands) {
    commands.insert_resource(SpatialHashGrid::new(100));
}

#[derive(Debug, Clone, Resource)]
pub struct SpatialHashGrid {
    cell_size: i32,
    pub grid: HashMap<(i32, i32), Vec<Entity>>,
}

pub fn grid_hash_from_coor(cell_size: &i32, position: &Vec2) -> (i32, i32) {
    return (position.x as i32 / cell_size, position.y as i32 / cell_size);
}

impl SpatialHashGrid {
    pub fn new(cell_size: i32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        let key = grid_hash_from_coor(&self.cell_size, &position);
        let cell = self.grid.get_mut((&key));
        match cell {
            Some(cell) => cell.push(entity),
            None => {
                let mut vec = Vec::new();
                vec.push(entity);
                self.grid.insert(key, vec);
            }
        }
    }

    // pub fn insert_bulk(&mut self, entities: Vec<Entity>, positions: Vec<Vec2>) {
    //     for (entity, position) in entities.iter().zip(positions.iter()) {
    //         self.insert(*entity, *position);
    //     }
    // }

    // Remove an entity from the grid
    pub fn remove(&mut self, entity: Entity, position: Vec2) {
        let mut en = self
            .grid
            .get_mut(&grid_hash_from_coor(&self.cell_size, &position));

        match en {
            Some(en) => en.retain(|ent| ent.index() != entity.index()),
            None => log::info!("Entity does not exist within this cell"),
        }
    }

    pub fn get_entities_in_cell(&self, x: i32, y: i32) -> Option<&Vec<Entity>> {
        self.grid.get(&(x, y))
    }

    fn visualize(&self) {
        for (coords, entities) in &self.grid {
            println!("Cell {:?}: {:?}", coords, entities);
        }
    }

    fn get_neighbouring_cells(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        let mut neighbours = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                info!("{:?} {:?}", x + dx, y + dy);

                if dx == 0 && dy == 0 {
                    continue;
                }
                neighbours.push((x + dx, y + dy));
            }
        }
        info!("{:?}", neighbours);
        neighbours
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::World;

    #[test]
    fn test_get_neighbouring_cells() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();

        let mut grid = SpatialHashGrid::new(32);
        grid.insert(entity, Vec2::new(0.0, 0.0));
        info!("{:?}", grid.get_neighbouring_cells(0, 0));
        let neighbours = grid.get_neighbouring_cells(0, 0);
        assert_eq!(neighbours.len(), 8);
    }

    #[test]
    fn test_visualize() {
        let mut world = World::default();

        let mut grid = SpatialHashGrid::new(32);
        for i in 0..10 {
            let entity = world.spawn_empty().id();
            let position = Vec2::new(i as f32 * 32.0, i as f32 * 32.0);
            grid.insert(entity, position);
        }

        grid.visualize();
    }

    #[test]
    fn test_insert_and_query() {
        let mut world = World::default();
        let mut grid = SpatialHashGrid::new(32);

        // Create an entity
        let entity = world.spawn_empty().id();
        let position = Vec2::new(15.0, 25.0);
        let entity2 = world.spawn_empty().id();
        let position2 = Vec2::new(25.0, 25.0);

        // Insert the entity into the grid
        grid.insert(entity, position);
        grid.insert(entity2, position2);

        // Query the grid at the same position
        //
        let key = grid_hash_from_coor(&32, &position);
        let result = grid.get_entities_in_cell(key.0, key.1);

        println!("{:?}", grid.grid.values());
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);
        assert_eq!(result.unwrap()[0], entity);
    }

    #[test]
    fn test_remove() {
        let mut world = World::default();
        let mut grid = SpatialHashGrid::new(32);

        let entity = world.spawn_empty().id();
        let position = Vec2::new(115.0, 251.0);

        grid.insert(entity, position);
        grid.remove(entity, position);

        let key = grid_hash_from_coor(&32, &position);
        println!("{:?}", key);
        // Query the grid at the same position
        let result = grid.get_entities_in_cell(key.0, key.1);
        println!("{:?}", grid.grid.values());
        println!("{:?}", grid.grid.get_key_value(&key));
        assert!(result.expect("No entities in cell").is_empty());
    }

    #[test]
    fn test_get_entities_in_cell() {
        let mut world = World::default();
        let mut grid = SpatialHashGrid::new(32);

        let entity = world.spawn_empty().id();
        let position = Vec2::new(15.0, 25.0);

        grid.insert(entity, position);
        info!("{:?}", grid);

        let result = grid.get_entities_in_cell(0, 1);
        assert!(result.is_none());

        let result = grid.get_entities_in_cell(0, 0);
        assert!(result.is_some());

        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(result.unwrap()[0], entity);
    }
}
