use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::{Camera, Condition, GlobalTransform, Window};

pub fn camera_to_world_coordinate(camera: &Camera, camera_transform: &GlobalTransform, window: &Window) -> Option<Vec2> {
    info!("ORIGINAL: {:?}" , window.cursor_position());

    let pos = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());

    info!("Transformed: {:?}" , pos);

    return pos;
}