use bevy::math::vec3;
use bevy::prelude::*;
use bevy::sprite::Wireframe2dConfig;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_particle);
    }
}

#[derive(Component)]
pub struct Particle;

fn spawn_particle(mut commands: Commands) {
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
    ));
}