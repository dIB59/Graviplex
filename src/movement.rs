use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Component, KeyCode, Query, Res, Time, Transform, With};
use bevy::time::Fixed;
use crate::paddle::Paddle;

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec2,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_position);
        app.add_systems(Update, move_paddle);
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x;
        transform.translation.y += velocity.value.y;
    }
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
        paddle_transform.translation.x + direction * 0.005 * time_step.wrap_period().as_secs_f32();
    paddle_transform.translation.x = new_x;
}


