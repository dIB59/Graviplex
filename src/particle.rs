use bevy::math::vec3;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use rand::Rng;
use crate::movement::Velocity;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_particle);
    }
}

#[derive(Component)]
pub struct Particle;

fn spawn_particle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let random_velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));

    commands.spawn((
        MaterialMesh2dBundle {
            transform: Transform {
                translation: vec3(0., 0., 0.),
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