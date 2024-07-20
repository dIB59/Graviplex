use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Entity, info, Query, Res, ResMut, Resource, Time, Timer, TimerMode, Transform};

use crate::movement::Velocity;

pub struct DebugPlugin;

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrintTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(Update, print_position);
    }
}

#[derive(Resource)]
struct PrintTimer(Timer);

fn print_position(time: Res<Time>, mut timer: ResMut<PrintTimer>, query: Query<(Entity, &Transform, &Velocity)>) {
    if timer.0.tick(time.delta()).just_finished() {
        for (entity, transform, _velocity) in query.iter() {
            info!("Entity {:?} is at position {:?}", entity, transform);
        }
    }
}