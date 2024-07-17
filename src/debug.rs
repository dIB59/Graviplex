use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{AppExit, Entity, EventWriter, info, KeyCode, Query, Res, Transform};

use crate::movement::Velocity;

pub struct DebugPlugin;

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, print_position);
    }
}

fn print_position(query: Query<(Entity, &Transform, &Velocity)>) {
    // for (entity, position, _ve) in query.iter() {
    //     info!("Entity {:?} is at position {:?},", entity, position);
    // }
}
