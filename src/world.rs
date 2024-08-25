use bevy::log::{error, info};
use bevy::math::Vec2;
use bevy::prelude::{Camera, GlobalTransform};

pub fn camera_to_world_coordinate(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    coordinate: Option<Vec2>,
) -> Option<Vec2> {
    info!("ORIGINAL: {:?}", coordinate);

    match coordinate {
        Some(pos) => {
            let world_pos = camera.viewport_to_world_2d(camera_transform, pos);
            info!("Transformed world position: {:?}", world_pos);
            world_pos
        }
        None => {
            error!("Cursor position is not available in the window.");
            coordinate
        }
    }
}
