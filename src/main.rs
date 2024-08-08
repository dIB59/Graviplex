use bevy::prelude::*;

use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::fps::FpsPlugin;
use crate::input::UserInputPlugin;
use crate::movement::MovementPlugin;
use crate::platform_adaptive::PlatformAdaptiveDefaultPlugin;

mod camera;
mod debug;
mod fps;
mod input;
mod movement;
mod particle;
mod platform_adaptive;
mod world;

fn main() {
    App::new()
        .add_plugins(PlatformAdaptiveDefaultPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FpsPlugin)
        .add_plugins(UserInputPlugin)
        .add_systems(Update, exit_on_esc_system)
        .run();
}

fn exit_on_esc_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        info!("EXIT");
        app_exit_events.send(AppExit::Success);
    }
}
