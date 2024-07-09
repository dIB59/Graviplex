use bevy::prelude::*;
use bevy::sprite::Wireframe2dPlugin;

use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::movement::MovementPlugin;
use crate::paddle::PaddlePlugin;
use crate::particle::ParticlePlugin;

mod camera;
mod debug;
mod movement;
mod paddle;
mod particle;

fn main() {
    App::new()
        .add_plugins(CameraPlugin)
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_plugins(PaddlePlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(ParticlePlugin)
        .run();
}