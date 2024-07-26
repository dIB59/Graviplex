use std::slice::Windows;
use bevy::app::{App, Plugin, Update};
use bevy::asset::ron::Value::Option;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Camera, Component, GlobalTransform, info, KeyCode, Mut, Query, Res, Time, Transform, Window, With};
use bevy::time::Fixed;
use bevy::utils::info;
use bevy::window::PrimaryWindow;
use crate::paddle::Paddle;
use crate::particle::{Particle};
use crate::world::{camera_to_world_coordinate};

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
        app.add_systems(Update, handle_collisions);
        app.add_systems(Update, border_hit);
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

fn handle_collisions(mut query: Query<(&mut Transform, &mut Velocity)>) {
    let mut particles = query.iter_mut().collect::<Vec<_>>();
    for i in 0..particles.len() {

        let (left, right) = particles.split_at_mut(i + 1);
        let (transform1, velocity1) = &mut left[i];
        // border_hit(transform1);
        for j in 0..right.len() {
            let (transform2, velocity2) = &mut right[j];

            let pos1 = transform1.translation;
            let pos2 = transform2.translation;

            let distance = pos1.distance(pos2);
            let sum_of_radii = 10.;

            if distance < sum_of_radii {
                // Resolve the collision
                let normal = (pos2 - pos1).normalize();
                let relative_velocity = velocity2.value - velocity1.value;
               let speed = relative_velocity.dot(Vec2::new(normal.x, normal.y));

                if speed > 0.0 {
                    let impulse = 2.0 * speed / (10.);
                    let impulse_vec = impulse * Vec2::new(normal.x, normal.y);

                    velocity1.value -= impulse_vec * 5.;
                    velocity2.value += impulse_vec * 5.;

                    // // Reposition the balls to avoid overlap
                    // let penetration = sum_of_radii - distance;
                    // let correction = normal * penetration / 2.0;

                    // transform1.translation -= correction;
                    // transform2.translation += correction;
                }
            }
        }
    }
}

fn border_hit(mut particles: Query<(&mut Transform, &mut Velocity)>, windows: Query<&Window, With<PrimaryWindow>>) {

    let window = windows.single();

    let window_width = window.width();
    let window_height = window.height();

    for (mut transform, _velocity) in particles.iter_mut() {
        let new_position = transform.translation;
        info!("{:?}", new_position);

        if new_position.x > window_width / 2.0 || new_position.x < -window_width / 2.0 {
            transform.translation.x = -transform.translation.x;
        }
        if new_position.y > window_height / 2.0 || new_position.y < -window_height / 2.0 {
            transform.translation.y = -transform.translation.y;
        }
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
        paddle_transform.translation.x + direction * time_step.delta_seconds() * 300f32;
    paddle_transform.translation.x = new_x;
}


