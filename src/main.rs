use bevy::prelude::*;

use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::fps::FpsPlugin;
use crate::input::UserInputPlugin;
use crate::movement::MovementPlugin;
use crate::vulkan_render::VulkanRenderPlugin;

mod camera;
mod debug;
mod movement;
mod particle;
mod fps;
mod input;
mod world;
mod vulkan_render;

fn main() {
    App::new()
        .add_plugins(VulkanRenderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FpsPlugin)
        .add_plugins(UserInputPlugin)
        .run();
}