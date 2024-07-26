use std::sync::Arc;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::log::info;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Circle, Component, default, GlobalTransform, Handle, Mesh, ResMut, Transform, UntypedHandle, Visibility};
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2d, Mesh2dHandle};

#[derive(Component)]
pub struct Particle {
    pub bundle: MaterialMesh2dBundle<ColorMaterial>,
}

impl Particle {

    pub fn new(
        position: Vec3,
        r: f32,
        color: Color,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        MaterialMesh2dBundle {
            transform: Transform {
                translation: position,
                ..default()
            },
            mesh: meshes.add(Mesh::from(Circle { radius: r })).into(),
            material: materials.add(ColorMaterial::from(color)),
            ..default()
        }
    }

    pub fn default(meshes: &mut ResMut<Assets<Mesh>>,
                   materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        MaterialMesh2dBundle {
            transform: default(),
            mesh: meshes.add(Mesh::from(Circle::default())).into(),
            material: materials.add(ColorMaterial::from(Color::default())),
            ..default()
        }
    }
}