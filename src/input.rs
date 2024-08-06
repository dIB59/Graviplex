use bevy::prelude::*;
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
    let window = q_windows.get_single();

    if window.is_err() {
        error!("Window does not exitst");
        return;
    }

    let mut rng = rand::thread_rng();
    let random_velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));

    match camera_to_world_coordinate(camera, camera_transform, window.expect("sometrihng")) {
        Some(vec2) => {
            let mut rng = rand::thread_rng();
            for _ in 0..1000 {
                let offset_x = rng.gen_range(-20.0..20.0);
                let offset_y = rng.gen_range(-20.0..20.0);
                let particle_position = Vec3::new(vec2.x + offset_x, vec2.y + offset_y, 0.0);

                commands.spawn((
                    Velocity { value: random_velocity }
                )).insert(
                    Particle::new(
                        particle_position,
                        4.0,
                        Color::WHITE,
                        &mut meshes,
                        &mut materials,
                    ));
            }
        }
        None => {
            warn!("Cursor click position was not found");
            commands.spawn((
                Velocity { value: random_velocity }
            )).insert(Particle::default(&mut meshes, &mut materials))
            ;
        }
    }
}
