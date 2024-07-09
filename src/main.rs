use bevy::prelude::*;
use bevy::sprite::Wireframe2dPlugin;

use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::movement::MovementPlugin;
use crate::particle::ParticlePlugin;
use crate::spaceship::SpaceShipPlugin;

mod camera;
mod debug;
mod movement;
mod spaceship;
mod particle;

fn main() {
    App::new()
        .add_plugins(CameraPlugin)
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_plugins(SpaceShipPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(ParticlePlugin)
        .run();
}