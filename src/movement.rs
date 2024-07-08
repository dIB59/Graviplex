use bevy::app::{App, Plugin, Update};
use bevy::math::Vec2;
use bevy::prelude::{Component, Query, Transform};

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec2,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_position);
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x;
        transform.translation.y += velocity.value.y;
    }
}


