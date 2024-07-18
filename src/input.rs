use bevy::app::{App, Update};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::log::warn;
use bevy::math::{Vec2, vec3};
use bevy::prelude::{Circle, ColorMaterial, Commands, default, Mesh, MouseButton, Plugin, Query, Res, ResMut, Transform, Window, With};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::info;
use bevy::window::PrimaryWindow;
use rand::Rng;
use crate::movement::Velocity;
use crate::particle::Particle;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_particle_cursor);
    }
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
    }

}

fn spawn_particle_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let position = match q_windows.single().cursor_position() {
        Some(vec2) => vec2,
        None => {
            warn!("Cursor click position was not found");
            return;
        }
    };


    let mut rng = rand::thread_rng();
    let random_velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));

    commands.spawn((
        MaterialMesh2dBundle {
            transform: Transform {
                translation: vec3(position.x, position.y, 0.),
                ..default()
            },
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 10.0 })),
            material: materials.add(Color::WHITE),
            ..default()
        },
        Velocity {
            value: random_velocity,
        },
        Particle,
    ));
}