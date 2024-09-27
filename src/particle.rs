use bevy::{
    asset::Assets,
    color::Color,
    math::{Vec2, Vec3},
    prelude::{Circle, Commands, Component, default, Entity, Mesh, ResMut, Transform},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::movement::Velocity;

#[derive(Component)]
pub struct Particle;

pub trait Spawnable<T> {
    fn spawn(
        commands: &mut Commands,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        velocity: Vec2,
    ) -> T;
}

impl Spawnable<Entity> for Particle {
    fn spawn(
        commands: &mut Commands,
        particle_position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        random_velocity: Vec2,
    ) -> Entity {
        commands
            .spawn(
                MaterialMesh2dBundle {
                    transform: Transform::from_translation(particle_position),
                    mesh: Mesh2dHandle(meshes.add(Circle::new(5.0))),
                    material: materials.add(Color::WHITE),
                    ..default()
                }
            )
            .insert(Particle)
            .insert(Velocity::from(random_velocity))
            .id()
    }
}
