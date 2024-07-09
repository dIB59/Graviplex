use bevy::color::palettes::basic::BLACK;
use bevy::color::palettes::css::{DARK_CYAN, WHITE};
use bevy::prelude::*;
use bevy::sprite::Wireframe2dConfig;
use bevy_prototype_lyon::prelude::{Fill, GeometryBuilder, ShapeBundle, Stroke};
use bevy_prototype_lyon::shapes;

use crate::movement::Velocity;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_system);
        app.add_systems(Update, toggle_wireframe);
        app.add_systems(Update, draw_particle);
    }
}

fn setup_system(mut commands: Commands) {

    let circle = shapes::Circle{
        radius: 100.0,
        center: Default::default(),
    } ;

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&circle),
            ..default()
        },
        Fill::color(WHITE),
        Stroke::new(WHITE, 10.0),
    ));
}

fn draw_particle(query: Query<(&Velocity, &mut Transform)>, mut commands: Commands) {
    let circle = shapes::Circle{
        radius: 100.0,
        center: Vec2::ZERO,
    } ;

    commands.spawn((
        GeometryBuilder::build_as(&circle),
        Stroke::new(Color::WHITE, 1.0)
    ));
    
    for (_velocity, transform) in query.iter() {
        Transform::from_xyz(transform.translation.x,transform.translation.y, 0.0);
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&circle),
                ..default()
            },
            Fill::color(DARK_CYAN),
            Stroke::new(BLACK, 10.0),
        ));
    }
}


fn toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}
