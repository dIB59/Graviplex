use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};

use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::fps::FpsPlugin;
use crate::input::UserInputPlugin;
use crate::movement::MovementPlugin;

mod camera;
mod debug;
mod movement;
mod particle;
mod fps;
mod input;
mod world;
mod window;
mod vulkan_render;
mod grid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CameraPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FpsPlugin)
        .add_plugins(UserInputPlugin)
        .run();
}