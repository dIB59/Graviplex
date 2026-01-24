//! System context for unified system signatures.
//!
//! This module provides a `SystemContext` that combines both the ECS World
//! and per-frame resources (time, input, camera) into a single context.
//! This allows systems to have consistent signatures and access both
//! entity data and frame resources.
//!
//! # Example
//!
//! ```ignore
//! // System with unified signature
//! fn my_system(ctx: &mut SystemContext) {
//!     let dt = ctx.delta();
//!     let input = ctx.input();
//!     
//!     for (_, (transform, velocity)) in ctx.query::<(&mut Transform, &Velocity)>() {
//!         transform.position += velocity.0 * dt;
//!     }
//! }
//! ```

use crate::core::time::Time;
use crate::input::InputState;
use crate::renderer::Camera2D;

use super::Resources;
use super::World;

/// A unified context for systems that provides access to both the ECS World
/// and per-frame resources.
///
/// `SystemContext` wraps a mutable reference to the World along with immutable
/// references to frame resources (time, input, camera). This enables consistent
/// system signatures while maintaining Rust's borrowing rules.
///
/// # Usage
///
/// Systems can use `SystemContext` for a unified API:
///
/// ```ignore
/// fn player_movement(ctx: &mut SystemContext) {
///     let dt = ctx.delta();
///     let speed = 200.0;
///     
///     let mut dir = Vec2::ZERO;
///     if ctx.input().is_key_pressed(KeyCode::KeyW) {
///         dir.y += 1.0;
///     }
///     // ... etc
///     
///     for (_, (transform, player)) in ctx.query::<(&mut Transform, &Player)>() {
///         transform.position += dir * speed * dt;
///     }
/// }
/// ```
///
/// # Note
///
/// The existing system functions with specific signatures (like `movement_system(world, dt)`)
/// are still available and work well for simple cases. `SystemContext` is an alternative
/// for more complex systems that need multiple resources.
pub struct SystemContext<'a> {
    /// The ECS world containing all entities and components.
    pub world: &'a mut World,
    /// Per-frame resources (time, input, camera).
    resources: Resources<'a>,
}

impl<'a> SystemContext<'a> {
    /// Creates a new SystemContext.
    pub fn new(
        world: &'a mut World,
        time: &'a Time,
        input: &'a InputState,
        camera: &'a Camera2D,
    ) -> Self {
        Self {
            world,
            resources: Resources::new(time, input, camera),
        }
    }

    /// Creates a SystemContext from existing Resources.
    pub fn from_resources(world: &'a mut World, resources: Resources<'a>) -> Self {
        Self { world, resources }
    }

    // =========================================================================
    // Time accessors
    // =========================================================================

    /// Returns the delta time (seconds since last frame).
    #[inline]
    pub fn delta(&self) -> f32 {
        self.resources.time.delta()
    }

    /// Returns the current frames per second.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.resources.time.fps()
    }

    /// Returns the total elapsed time since app start.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.resources.time.elapsed()
    }

    /// Returns a reference to the Time resource.
    #[inline]
    pub fn time(&self) -> &Time {
        self.resources.time
    }

    // =========================================================================
    // Input accessors
    // =========================================================================

    /// Returns a reference to the InputState.
    #[inline]
    pub fn input(&self) -> &InputState {
        self.resources.input
    }

    // =========================================================================
    // Camera accessors
    // =========================================================================

    /// Returns a reference to the Camera.
    #[inline]
    pub fn camera(&self) -> &Camera2D {
        self.resources.camera
    }

    // =========================================================================
    // Resources accessor
    // =========================================================================

    /// Returns a reference to the Resources.
    #[inline]
    pub fn resources(&self) -> &Resources<'a> {
        &self.resources
    }
}

/// Type alias for system functions that use SystemContext.
pub type System = fn(&mut SystemContext);

/// Runs a list of systems in order.
///
/// # Example
///
/// ```ignore
/// let systems: &[System] = &[
///     player_movement,
///     enemy_ai,
///     collision_detection,
/// ];
///
/// run_context_systems(&mut ctx, systems);
/// ```
pub fn run_context_systems(ctx: &mut SystemContext, systems: &[System]) {
    for system in systems {
        system(ctx);
    }
}

#[cfg(test)]
mod tests {
    // Tests would require mocking Time/InputState/Camera which is complex.
    // Integration tests are more appropriate here.
}
