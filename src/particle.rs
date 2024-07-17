use bevy::math::vec3;
use bevy::prelude::*;
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

fn spawn_particle(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let random_velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: vec3(0., 0., 0.),
                ..default()
            },
            sprite: Sprite {
                color: Color::srgb(0.7, 0.3, 0.7),
                custom_size: Some(Vec2::new( 2.0, 2.0)),
                ..default()
            },
            ..default()
        },
        Particle,
        Velocity {
            value: Vec2::new(0.1, 0.1), // Initial velocity, customize as needed
        },
    ));
}