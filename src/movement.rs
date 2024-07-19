use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, KeyCode, Query, Res, Time, Transform, With};
use bevy::time::Fixed;
use crate::paddle::Paddle;
use crate::particle::Particle;

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec2,
}

impl Velocity {
    pub fn default() -> Velocity {
        Velocity {
            value: Vec2::ZERO
        }
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_position);
        app.add_systems(Update, move_paddle);
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>, time_step: Res<Time<Fixed>>
) {
    let dt = time_step.delta_seconds() * 100f32;
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x * dt;
        transform.translation.y += velocity.value.y * dt;
    }
}

fn handle_collisions(mut particle: Query<&mut Transform, With<Particle>>)
{
}

fn move_paddle(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time_step: Res<Time<Fixed>>
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if input.pressed(KeyCode::ArrowLeft){
        direction -= 1.0;
    }

    if input.pressed(KeyCode::ArrowRight){
        direction += 1.0;
    }

    let new_x =
        paddle_transform.translation.x + direction * time_step.delta_seconds() * 300f32;
    paddle_transform.translation.x = new_x;
}


