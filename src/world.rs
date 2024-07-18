use bevy::math::Vec2;
use bevy::prelude::{Camera, GlobalTransform, Window};

pub fn camera_to_world_coordinate(camera: &Camera, camera_transform: &GlobalTransform, window: &Window) -> Option<Vec2> {
    window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
}