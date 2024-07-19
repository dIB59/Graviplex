use bevy::math::vec3;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;
use rand::Rng;
use crate::camera::MainCamera;
use crate::movement::Velocity;
use crate::particle::Particle;
use crate::world::camera_to_world_coordinate;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_particle_cursor);
    }
}

fn spawn_particle_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = q_camera.single();
    let window = q_windows.single();

    let position = match camera_to_world_coordinate(camera, camera_transform, window) {
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
