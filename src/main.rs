use bevy::prelude::*;

use crate::camera::CameraPlugin;
use crate::custom_render::CustomRenderPlugin;
use crate::debug::DebugPlugin;
use crate::fps::FpsPlugin;
use crate::input::UserInputPlugin;
use crate::movement::MovementPlugin;

mod camera;
mod custom_render;
mod debug;
mod fps;
mod input;
mod movement;
mod particle;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CustomRenderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FpsPlugin)
        .add_plugins(UserInputPlugin)
        .run();
}
