use bevy::app::{App, Plugin, Startup};
use bevy::color::Color;
use bevy::math::{Vec2, vec3};
use bevy::prelude::{Commands, Component, Transform};
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::utils::default;

pub struct PaddlePlugin;

impl Plugin for PaddlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_paddle);
    }
}

const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);



#[derive(Component)]
pub struct Paddle;

fn spawn_paddle(mut commands: Commands) {
    
    commands.spawn((
        SpriteBundle{
            transform: Transform {
                translation: vec3(0., 1.0, 0.),
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                custom_size: Some(PADDLE_SIZE),
                ..default()
            },
            ..default()
        },
        Paddle,
    ));
}
