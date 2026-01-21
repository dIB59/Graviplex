//! GameLoop trait for implementing games with the engine

use crate::{Camera2D, GpuContext, InputState};

/// Trait that games must implement to use the engine.
///
/// The engine calls these methods at appropriate times during the game loop.
pub trait GameLoop: 'static {
    /// Called once when the GPU context is initialized.
    /// Use this to create GPU resources (buffers, pipelines, etc.)
    fn init(&mut self, gpu: &GpuContext);

    /// Called every frame before rendering.
    /// `dt` is the time since last frame in seconds.
    fn update(&mut self, dt: f32, gpu: &GpuContext);

    /// Called every frame to render game content.
    /// The engine handles clearing the screen and presenting.
    fn render(&mut self, gpu: &GpuContext, view: &wgpu::TextureView, camera: &Camera2D);

    /// Called to handle per-frame input processing.
    /// Return true if input was consumed.
    fn handle_input(&mut self, input: &InputState, camera: &Camera2D) -> bool {
        let _ = (input, camera);
        false
    }

    /// Called to render game-specific GUI elements.
    fn gui(&mut self, ctx: &egui::Context) {
        let _ = ctx;
    }
}
