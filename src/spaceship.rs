use bevy::app::{App, Plugin, Startup};
use bevy::math::Vec2;
use bevy::prelude::{Commands, SpatialBundle};

use crate::movement::Velocity;

pub struct SpaceShipPlugin;

impl Plugin for SpaceShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spaceship);
    }
}

fn spawn_spaceship(mut commands: Commands) {
    commands.spawn((
        SpatialBundle::default(),
        Velocity {
            value: Vec2::new(1.0, -1.0)
        }
    ));
}
