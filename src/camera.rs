use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Camera2dBundle, Commands, Component};

pub struct CameraPlugin;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(), MainCamera,
    ));
}
