use bevy::log::{error, info};
use bevy::math::Vec2;
use bevy::prelude::{Camera, GlobalTransform, Window};

pub fn camera_to_world_coordinate(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
) -> Option<Vec2> {
    info!("ORIGINAL: {:?}", window.cursor_position());

    let cursor_pos = window.cursor_position();

    match cursor_pos {
        Some(pos) => {
            let world_pos = camera.viewport_to_world_2d(camera_transform, pos);
            info!("Transformed world position: {:?}", world_pos);
            world_pos
        }
        None => {
            error!("Cursor position is not available in the window.");
            cursor_pos
        }
    }
}
