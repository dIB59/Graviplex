use bevy::app::{App, FixedUpdate, Plugin};
use bevy::log;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Component, Query, Res, ResMut, SystemSet, Time, Transform, Window, With};
use bevy::prelude::Entity;
use bevy::prelude::Mut;
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use bevy::time::Fixed;
use bevy::utils::info;
use bevy::window::PrimaryWindow;

use crate::grid::{grid_hash_from_coor, SpatialHashGrid};
use crate::particle::Particle;

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
        app.add_systems(FixedUpdate, apply_velocity);
        app.add_systems(FixedUpdate, handle_cell_collisions);
        app.add_systems(FixedUpdate, border_hit);
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time_step: Res<Time<Fixed>>) {
    let dt = time_step.delta_seconds() * 100f32;
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x * dt;
        transform.translation.y += velocity.value.y * dt;
    }
}

type GridParticles<'a> = Vec<(Entity, Mut<'a, Transform>, Mut<'a, Velocity>)>;

fn handle_cell_collisions(
    mut particles: Query<(Entity, &mut Transform, &mut Velocity)>,
    mut grid: ResMut<SpatialHashGrid>,
    time_step: Res<Time<Fixed>>,
) {
    let dt = time_step.delta_seconds() * 100f32;
    //iterate over all cells

    //get all particles in query
    let particles_in_query = particles.iter().collect::<Vec<_>>();
    //update grid with all particles
    let particle_positions = particles_in_query.iter().map(|(_, transform, _)| Vec2::new(transform.translation.x, transform.translation.y)).collect();
    grid.update(particles_in_query.iter()
                    .map(|(entity, _, _)| *entity).collect(), particle_positions);
    

    for (entity) in grid.cells.keys() {
        let mut particles_in_cell = grid.get_entities_in_cell(entity.0, entity.1);
        if particles_in_cell.is_none() {
            continue;
        }
        let available_particles_in_cell = match particles_in_cell {
            Some(particles_in_cell) => particles_in_cell,
            None => {
                log::warn!("Particles in cell not found");
                continue;
            }
        };

        //use window method to get the other particles in the cell
        available_particles_in_cell.windows(2).for_each(|particle| {
            let [mut particle_1, mut particle_2]: [(_, Mut<'_, Transform>, Mut<'_, Velocity>); 2] = particles
                .get_many_mut([particle[0], particle[1]])
                .expect("One or both particles do not exist");

            collide_two_particles(particle_1, particle_2);
        });
    }
}

fn collide_two_particles(
    mut particle: (Entity, Mut<Transform>, Mut<Velocity>),
    mut other_particle: (Entity, Mut<Transform>, Mut<Velocity>),
) {
    let (entity1, transform1, mut velocity1) = particle;
    let (entity2, transform2, mut velocity2) = other_particle;

    let distance = transform1.translation.distance(transform2.translation);

    if distance > 1.0 {
        return;
    }

    log::info!("Collision detected");

    let collision_vector = transform1.translation - transform2.translation;
    let collision_vector = collision_vector.normalize();
    let collision_vector = collision_vector * 0.5;

    velocity1.value -= Vec2::new(collision_vector.x, collision_vector.y);
    velocity2.value += Vec2::new(collision_vector.x, collision_vector.y);
}


fn border_hit(
    mut particles: Query<(&mut Transform, &mut Velocity)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = match q_windows.get_single() {
        Ok(window) => window,
        Err(_) => {
            log::warn!("Primary window not found");
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
