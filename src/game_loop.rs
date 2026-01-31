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

    /// Returns the position the camera should follow/center on.
    ///
    /// Override this method to make the camera follow an entity (like a player).
    /// Return `None` to use the default camera controller (arrow keys/mouse).
    ///
    /// For smoothing and offset configuration, see [`camera_follow_config`].
    ///
    /// # Arguments
    /// * `world` - The ECS world (read-only)
    ///
    /// # Example
    /// ```ignore
    /// fn camera_target(&self, world: &World) -> Option<[f32; 2]> {
    ///     // Follow the player entity
    ///     for (_, (transform, _)) in world.query::<(&Transform, &Player)>().iter() {
    ///         return Some([transform.position.x, transform.position.y]);
    ///     }
    ///     None
    /// }
    /// ```
    fn camera_target(&self, world: &World) -> Option<[f32; 2]> {
        let _ = world;
        None
    }

    /// Returns the camera follow configuration.
    ///
    /// Override this to customize how the camera follows the target:
    /// - `smoothing`: How fast camera catches up (8.0 = balanced, 5.0 = floaty, 15.0 = snappy)
    /// - `max_offset`: Maximum distance camera can lag behind target (0 = no limit)
    /// - `deadzone`: Camera won't move if target within this distance
    ///
    /// # Example
    /// ```ignore
    /// fn camera_follow_config(&self) -> CameraFollow {
    ///     CameraFollow::new()
    ///         .with_smoothing(6.0)      // Smooth follow
    ///         .with_max_offset(150.0)   // Max 150 pixels behind
    ///         .with_deadzone(5.0)       // Ignore tiny movements
    /// }
    /// ```
    fn camera_follow_config(&self) -> crate::renderer::CameraFollow {
        crate::renderer::CameraFollow::new()
    }

    /// Called to render game-specific GUI elements (requires `gui` feature).
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
