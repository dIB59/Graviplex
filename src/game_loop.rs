//! GameLoop trait for implementing games with the engine

use crate::core::time::Time;
use crate::renderer::gpu_context::GpuContext;
use crate::{Camera2D, DrawContext, InputState};

/// Trait that games must implement to use the engine.
///
/// The engine calls these methods at appropriate times during the game loop.
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
///
/// struct MyGame {
///     player_pos: Vec2,
/// }
///
/// impl GameLoop for MyGame {
///     fn init(&mut self, _gfx: &Graphics) {
///         // Initialize GPU resources here
///     }
///
///     fn update(&mut self, time: &Time, _gfx: &Graphics) {
///         // Update game logic
///         println!("FPS: {:.1}, dt: {:.4}", time.fps(), time.delta());
///     }
///
///     fn render(&mut self, draw: &mut DrawContext) {
///         draw.circle(Circle::new(self.player_pos, 50.0, Color::RED));
///     }
/// }
/// ```
pub trait GameLoop: 'static {
    /// Called once when the GPU context is initialized.
    /// Use this to create GPU resources (buffers, pipelines, etc.)
    fn init(&mut self, gfx: &GpuContext);

    /// Called every frame before rendering.
    ///
    /// # Arguments
    /// * `time` - Frame timing information (delta time, fps, elapsed time)
    /// * `gfx` - GPU context for creating/updating GPU resources
    fn update(&mut self, time: &Time, gfx: &GpuContext);

    /// Called every frame to render game content.
    /// The engine handles clearing the screen and presenting.
    fn render(&mut self, draw: &mut DrawContext);

    /// Called to handle per-frame input processing.
    /// Return true if input was consumed.
    fn handle_input(&mut self, input: &InputState, camera: &Camera2D) -> bool {
        let _ = (input, camera);
        false
    }

    /// Called to render game-specific GUI elements (requires `gui` feature).
    #[cfg(feature = "gui")]
    fn gui(&mut self, ctx: &egui::Context) {
        let _ = ctx;
    }
}
