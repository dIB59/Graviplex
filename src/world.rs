use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::{Camera, GlobalTransform, Window};

pub fn camera_to_world_coordinate(camera: &Camera, camera_transform: &GlobalTransform, window: &Window) -> Option<Vec2> {
    info!("ORIGINAL: {:?}" , window.cursor_position());

    let pos = camera.viewport_to_world_2d(camera_transform, window.cursor_position()?);

    info!("Transformed: {:?}" , pos);

    pos
}