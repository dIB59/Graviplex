//! GameLoop trait for implementing games with the engine

use crate::core::time::Time;
use crate::ecs::{Resources, World};
use crate::renderer::gpu_context::GpuContext;
use crate::{Camera2D, DrawContext, InputState};

/// Trait that games must implement to use the engine.
///
/// The engine calls these methods at appropriate times during the game loop.
/// The ECS [`World`] is passed to all methods, allowing entity management
/// throughout the game lifecycle.
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
///
/// struct MyGame;
///
/// impl GameLoop for MyGame {
///     fn init(&mut self, world: &mut World, _gfx: &Graphics) {
///         // Spawn initial entities
///         world.spawn((
///             Transform::from_position(Vec2::ZERO),
///             Sprite::circle(50.0, Color::RED),
///             Visible,
///         ));
///     }
///
///     fn update(&mut self, world: &mut World, res: &Resources) {
///         // Run systems
///         systems::movement(world, res.delta());
///         systems::lifetime(world, res.delta());
///         systems::despawn(world);
///     }
///
///     fn render(&mut self, world: &World, draw: &mut DrawContext) {
///         // Render all visible entities
///         draw.render_world(world);
///     }
/// }
/// ```
pub trait GameLoop: 'static {
    /// Called once when the GPU context is initialized.
    ///
    /// Use this to spawn initial entities and create GPU resources.
    ///
    /// # Arguments
    /// * `world` - The ECS world for spawning entities
    /// * `gfx` - GPU context for creating buffers, pipelines, etc.
    fn init(&mut self, world: &mut World, gfx: &GpuContext);

    /// Called every frame before rendering.
    ///
    /// Run your game systems here to update entity state.
    ///
    /// # Arguments
    /// * `world` - The ECS world
    /// * `res` - Per-frame resources (time, input, camera)
    fn update(&mut self, world: &mut World, res: &Resources);

    /// Called every frame to render game content.
    ///
    /// Use [`DrawContext::render_world`] for automatic batched rendering
    /// of all visible entities, or query the world manually for custom rendering.
    ///
    /// # Arguments
    /// * `world` - The ECS world (read-only)
    /// * `draw` - Drawing context for rendering shapes
    fn render(&mut self, world: &World, draw: &mut DrawContext);

    /// Called to handle per-frame input processing.
    ///
    /// Return `true` if input was consumed (prevents further processing).
    ///
    /// # Arguments
    /// * `world` - The ECS world
    /// * `input` - Current input state
    /// * `camera` - The 2D camera (for coordinate conversion)
    fn handle_input(&mut self, world: &mut World, input: &InputState, camera: &Camera2D) -> bool {
        let _ = (world, input, camera);
        false
    }

    /// Called to render game-specific GUI elements (requires `gui` feature).
    #[cfg(feature = "gui")]
    fn gui(&mut self, ctx: &egui::Context) {
        let _ = ctx;
    }

    // =========================================================================
    // Deprecated methods for backwards compatibility
    // =========================================================================

    /// Deprecated: Use the new init signature with World.
    #[deprecated(since = "0.4.0", note = "Use init(&mut self, world: &mut World, gfx: &GpuContext) instead")]
    fn init_legacy(&mut self, _gfx: &GpuContext) {}

    /// Deprecated: Use the new update signature with World and Resources.
    #[deprecated(since = "0.4.0", note = "Use update(&mut self, world: &mut World, res: &Resources) instead")]
    fn update_legacy(&mut self, _time: &Time, _gfx: &GpuContext) {}

    /// Deprecated: Use the new render signature with World.
    #[deprecated(since = "0.4.0", note = "Use render(&mut self, world: &World, draw: &mut DrawContext) instead")]
    fn render_legacy(&mut self, _draw: &mut DrawContext) {}
}
