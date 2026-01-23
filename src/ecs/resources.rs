//! Per-frame resources available to systems.
//!
//! Resources are shared data that systems need access to but don't belong
//! to any specific entity (like time, input state, camera).

use crate::core::time::Time;
use crate::input::InputState;
use crate::renderer::Camera2D;

/// Per-frame resources available during update.
///
/// Contains shared state that doesn't belong to any specific entity,
/// such as timing information, input state, and camera.
///
/// # Example
///
/// ```ignore
/// fn update(&mut self, world: &mut World, res: &Resources) {
///     let dt = res.time.delta();
///     let fps = res.time.fps();
///
///     // Check input
///     if res.input.is_key_pressed(KeyCode::Space) {
///         // Jump!
///     }
///
///     // Convert mouse to world coords
///     let mouse_world = res.camera.screen_to_world(res.input.mouse_pos());
/// }
/// ```
pub struct Resources<'a> {
    /// Frame timing information.
    pub time: &'a Time,
    /// Current input state (keyboard, mouse).
    pub input: &'a InputState,
    /// The 2D camera.
    pub camera: &'a Camera2D,
}

impl<'a> Resources<'a> {
    /// Creates a new Resources struct.
    pub fn new(time: &'a Time, input: &'a InputState, camera: &'a Camera2D) -> Self {
        Self {
            time,
            input,
            camera,
        }
    }

    /// Returns the delta time (seconds since last frame).
    #[inline]
    pub fn delta(&self) -> f32 {
        self.time.delta()
    }

    /// Returns the current frames per second.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.time.fps()
    }

    /// Returns the total elapsed time since app start.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.time.elapsed()
    }
}
