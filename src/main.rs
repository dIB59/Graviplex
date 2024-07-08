mod camera;
mod debug;
mod movement;
mod spaceship;

use bevy::prelude::*;
use bevy::sprite::Wireframe2dPlugin;
use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::movement::MovementPlugin;
use crate::spaceship::SpaceShipPlugin;


fn main() {
    App::new()
        .add_plugins(CameraPlugin)
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_plugins(SpaceShipPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .run();
}