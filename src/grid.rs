use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::HashMap;

use bevy::{prelude::*, window::PrimaryWindow};

const CELL_SIZE: f32 = 100.0;

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq)]
struct GridCell(pub i32, pub i32);

#[derive(Default, Resource, Clone, Debug)]
struct SpatialGrid {
    cells: HashMap<GridCell, Vec<Entity>>,
}

impl SpatialGrid {
    fn get_cells(&self) -> Vec<&GridCell> {
        let some: Vec<&GridCell> = self.cells.iter().map(|(e, _)| e).collect();
        return some;
    }
}

pub struct SpatialGridPlugin;

impl Plugin for SpatialGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialGrid::default())
            .add_systems(Update, update_spatial_grid)
            .add_systems(Startup, setup_based_on_window_size);
    }
}

fn setup_based_on_window_size(
    mut commands: Commands,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut grid: ResMut<SpatialGrid>,
) {
    let window = match q_windows.get_single() {
        Ok(window) => window,
        Err(_) => {
            warn!("Primary window not found");
            return;
        }
    };

    let window_width = window.width();
    let window_height = window.height();

    let grid_width = (window_width / CELL_SIZE).ceil() as i32;
    let grid_height = (window_height / CELL_SIZE).ceil() as i32;

    for x in 0..grid_width {
        for y in 0..grid_height {
            info!("SPAWNING CELL: ({:?}, {:?})", x, y);
            let grid_cell = GridCell(x, y);
            commands.spawn(grid_cell.clone());
            grid.cells.insert(grid_cell, Vec::new());
        }
    }

    //print cells in range

    for cell in grid.cells.borrow() {
        info!("Cell:  {:?}", cell.0);
        info!("Value: {:?}", cell.1)
    }
}

fn get_cell_coords(position: Vec3) -> (i32, i32) {
    (
        (position.x / CELL_SIZE).floor() as i32,
        (position.y / CELL_SIZE).floor() as i32,
    )
}

fn insert_entity_into_cell_based_on_position(
    position: Vec3,
    spatial_grid: &mut SpatialGrid,
    entity: Entity,
) {
    let cell_coords = get_cell_coords(position);
    spatial_grid
        .cells
        .entry(GridCell(cell_coords.0, cell_coords.1))
        .or_default()
        .push(entity);
}

fn update_spatial_grid(
    mut spatial_grid: ResMut<SpatialGrid>,
    mut query: Query<(Entity, &Transform, &mut GridCell)>,
) {
    // info!("FOLLOWING CELLS CREATED: {:?}", spatial_grid.cells);

    spatial_grid.cells.clear();

    for (entity, transform, mut cell) in query.iter_mut() {
        let new_cell_coords = get_cell_coords(transform.translation);

        if (cell.0, cell.1) != new_cell_coords {
            cell.0 = new_cell_coords.0;
            cell.1 = new_cell_coords.1;
        }

        spatial_grid
            .cells
            .entry(GridCell(cell.0, cell.1))
            .or_default()
            .push(entity);
    }
}
