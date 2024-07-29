use bevy::prelude::*;
use bevy::sprite::Wireframe2dPlugin;

use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::fps::FpsPlugin;
use crate::input::UserInputPlugin;
use crate::movement::MovementPlugin;
use crate::paddle::PaddlePlugin;

mod camera;
mod debug;
mod movement;
mod paddle;
mod particle;
mod fps;
mod input;
mod world;
mod window;
mod grid;

fn main() {
    App::new()
        .add_plugins(CameraPlugin)
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_plugins(PaddlePlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FpsPlugin)
        .add_plugins(UserInputPlugin)
        .run();
}