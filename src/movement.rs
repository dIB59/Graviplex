use bevy::app::{App, FixedUpdate, Plugin, PostUpdate, PreUpdate, Update};
use bevy::log::warn;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Component, Query, Res, Time, Transform, Window, With};
use bevy::time::Fixed;
use bevy::window::PrimaryWindow;

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec2,
}

impl From<Vec2> for Velocity {
    fn from(value: Vec2) -> Self {
        Self { value }
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self { value: Vec2::ZERO }
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_velocity);
        app.add_systems(Update, handle_collisions);
        app.add_systems(Update, border_hit);
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time_step: Res<Time<Fixed>>) {
    let dt = time_step.delta_seconds() * 100f32;
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x * dt;
        transform.translation.y += velocity.value.y * dt;
    }
}

fn handle_grid_collisions(
    grid: Res<SpatialHashGrid>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity)>,
) {
    let mut particles = query.iter_mut().collect::<Vec<_>>();

    grid.grid.iter().for_each(|e| log::info!("{:?}", e.1))
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

            if distance < sum_of_radii + 1. {
                let normal = (pos2 - pos1).normalize();
                let relative_velocity = velocity2.value - velocity1.value;
                let speed = relative_velocity.dot(Vec2::new(normal.x, normal.y));

                if speed > 0.0 {
                    let mass1 = 1.0; // Assuming mass of particle 1
                    let mass2 = 1.0; // Assuming mass of particle 2
                    let total_mass = mass1 + mass2;

                    let impulse = 2.0 * speed / total_mass;
                    let impulse_vec = impulse * Vec2::new(normal.x, normal.y);

                    velocity1.value -= impulse_vec * (mass2 / total_mass);
                    velocity2.value += impulse_vec * (mass1 / total_mass);
                }

                if speed > 2.0 {
                    velocity1.value = velocity1.value * 1. / speed;
                    velocity2.value = velocity2.value * 1. / speed;
                }
            }
        }
    }
}

fn border_hit(
    mut particles: Query<(&mut Transform, &mut Velocity)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = match q_windows.get_single() {
        Ok(window) => window,
        Err(_) => {
            warn!("Primary window not found");
            return;
        }
    };
    let window_width = window.width();
    let window_height = window.height();

    for (transform, mut velocity) in particles.iter_mut() {
        let new_position = transform.translation;

        if new_position.x > window_width / 2.0 || new_position.x < -window_width / 2.0 {
            velocity.value.x = -velocity.value.x;
        }
        if new_position.y > window_height / 2.0 || new_position.y < -window_height / 2.0 {
            velocity.value.y = -velocity.value.y;
        }
    }
}
