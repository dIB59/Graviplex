use bevy::app::{App, FixedUpdate, Plugin, PreUpdate, Update};
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Component, info, KeyCode, Query, Res, Time, Transform, Window, With};
use bevy::time::Fixed;
use bevy::window::PrimaryWindow;

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
        app.add_systems(FixedUpdate, apply_velocity);
        app.add_systems(PreUpdate, handle_collisions);
        app.add_systems(Update, border_hit);
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time_step: Res<Time<Fixed>>
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
                    info!(speed);
                    let mass1 = 1.0; // Assuming mass of particle 1
                    let mass2 = 1.0; // Assuming mass of particle 2
                    let total_mass = mass1 + mass2;

                    let impulse = 2.0 * speed / total_mass;
                    let impulse_vec = impulse * Vec2::new(normal.x, normal.y);

                    velocity1.value -= impulse_vec * (mass2 / total_mass);
                    velocity2.value += impulse_vec * (mass1 / total_mass);
                }

                if speed > 10.0 {
                    velocity1.value = velocity1.value * 1./speed;
                    velocity2.value = velocity2.value * 1./speed;
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

        if new_position.x > window_width / 2.0 || new_position.x < -window_width / 2.0 {
            transform.translation.x = -transform.translation.x;
        }
        if new_position.y > window_height / 2.0 || new_position.y < -window_height / 2.0 {
            transform.translation.y = -transform.translation.y;
        }
    }
}



