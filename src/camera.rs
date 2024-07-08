use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Camera2dBundle, Commands};

pub struct CameraPlugin;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
