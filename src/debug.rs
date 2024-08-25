#![allow(dead_code, unused_variables)]

use std::borrow::Borrow;

use bevy::app::{App, Plugin, PreUpdate, Update};
use bevy::log::info;
use bevy::prelude::{Entity, Query, Res, ResMut, Resource, Time, Timer, TimerMode, Transform};

use crate::movement::Velocity;

pub struct DebugPlugin;

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrintTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        // .add_systems(Update, print_enities_within_grid_cells)
        // .add_systems(Update, print_position);
    }
}

#[derive(Resource)]
struct PrintTimer(Timer);

fn print_position(
    query: Query<(Entity, &Transform, &Velocity)>,
    time: Res<Time>,
    mut timer: ResMut<PrintTimer>,
    // grid: Res<SpatialGrid>,
) {
    // if timer.0.tick(time.delta()).just_finished() {
    //     for (entity, transform, _velocity) in query.iter() {
    //         info!(
    //             "Entity {:?} is at position {:?}",
    //             entity.index(),
    //             transform.translation
    //         );
    //     }
    //     for cells in grid.cells.borrow().iter() {
    //         info!(
    //             "Cell: {:?}, Value: {:?}",
    //             cells.0,
    //             cells.1.iter().map(|e| e).collect::<Vec<&Entity>>()
    //         );
    //     }
    // }
}

// fn print_enities_within_grid_cells(
//     time: Res<Time>,
//     mut timer: ResMut<PrintTimer>,
//     query: Query<(Entity, &Transform, &Velocity)>,
//     grid: Res<SpatialGrid>,
// ) {
//     if timer.0.tick(time.delta()).just_finished() {
//         let cells = grid.cells.borrow();
//         for cell in cells {
//             info!(
//                 "Cell: {:?}, Value: {:?}",
//                 cell.0,
//                 cell.1.iter().map(|e| e).collect::<Vec<&Entity>>()
//             );
//         }
//     }
// }
