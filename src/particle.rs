use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::sprite::Wireframe2dConfig;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_particle);
        app.add_systems(Update, toggle_wireframe);
    }
}

fn spawn_particle(mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shapes = [
        Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
    ];
    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        let id = shape.id(); 
        info!("Shape created with id: {:?} ", id);
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn(MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            ..default()
        });
    }
    
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    info!("I am running");
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}
