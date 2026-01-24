//! Built-in systems for common game functionality.
//!
//! Systems are plain functions that operate on the World. They're designed to be
//! called explicitly in your game's update loop, giving you full control over
//! execution order.
//!
//! # Example
//!
//! ```ignore
//! fn update(&mut self, world: &mut World, res: &Resources) {
//!     let dt = res.time.delta();
//!
//!     // Run systems in desired order
//!     systems::player_input(world, &res.input);
//!     systems::movement(world, dt);
//!     systems::lifetime(world, dt);
//!     systems::despawn(world);
//!
//!     // Your custom systems...
//!     my_collision_system(world);
//! }
//! ```

use super::components::{Despawn, Lifetime, PlayerController, Transform, Velocity};
use super::world::World;
use super::Entity;
use crate::core::math::Vec2;
use crate::input::InputState;
use winit::keyboard::KeyCode;

/// Updates velocity based on player input (WASD/Arrow keys).
///
/// For each entity with [`Velocity`] and [`PlayerController`], reads the
/// current input state and sets the velocity accordingly.
///
/// # Arguments
///
/// * `world` - The ECS world
/// * `input` - Current input state
///
/// # Example
///
/// ```ignore
/// // In your game's update method:
/// player_input_system(&mut world, &res.input);
/// movement_system(&mut world, res.time.delta());
/// ```
pub fn player_input_system(world: &mut World, input: &InputState) {
    let keys = input.pressed_keys();

    for (_, (velocity, controller)) in world.query::<(&mut Velocity, &PlayerController)>().iter() {
        let mut dir = Vec2::ZERO;

        // WASD
        if keys.contains(&KeyCode::KeyW) || keys.contains(&KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if keys.contains(&KeyCode::KeyS) || keys.contains(&KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if keys.contains(&KeyCode::KeyA) || keys.contains(&KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if keys.contains(&KeyCode::KeyD) || keys.contains(&KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        // Normalize diagonal movement if enabled
        if controller.normalize_diagonal && dir.length() > 1.0 {
            dir = dir.normalize();
        }

        velocity.0 = dir * controller.speed;
    }
}

/// Updates positions based on velocity.
///
/// For each entity with [`Transform`] and [`Velocity`], applies:
/// `transform.position += velocity * dt`
///
/// # Arguments
///
/// * `world` - The ECS world
/// * `dt` - Delta time in seconds
///
/// # Example
///
/// ```ignore
/// // In your game's update method:
/// movement_system(&mut world, time.delta());
/// ```
pub fn movement_system(world: &mut World, dt: f32) {
    for (_, (transform, velocity)) in world.query::<(&mut Transform, &Velocity)>().iter() {
        transform.position += velocity.0 * dt;
    }
}

/// Decrements lifetimes and marks expired entities for despawn.
///
/// For each entity with [`Lifetime`]:
/// 1. Decrements the lifetime by `dt`
/// 2. If lifetime <= 0, adds the [`Despawn`] marker component
///
/// Should be followed by [`despawn_system`] to actually remove entities.
///
/// # Arguments
///
/// * `world` - The ECS world
/// * `dt` - Delta time in seconds
///
/// # Example
///
/// ```ignore
/// // In your game's update method:
/// lifetime_system(&mut world, time.delta());
/// despawn_system(&mut world);
/// ```
pub fn lifetime_system(world: &mut World, dt: f32) {
    // First pass: decrement lifetimes and collect expired entities
    let mut expired = Vec::new();

    for (entity, lifetime) in world.query::<&mut Lifetime>().iter() {
        lifetime.tick(dt);
        if lifetime.is_expired() {
            expired.push(entity);
        }
    }

    // Second pass: mark expired entities for despawn
    for entity in expired {
        let _ = world.insert(entity, Despawn);
    }
}

/// Removes all entities marked with the [`Despawn`] component.
///
/// This should typically be called at the end of your update loop,
/// after all other systems have run.
///
/// # Example
///
/// ```ignore
/// // At the end of update:
/// despawn_system(&mut world);
/// ```
pub fn despawn_system(world: &mut World) {
    // Collect entities to despawn (can't despawn while iterating)
    let to_despawn: Vec<Entity> = world.query::<&Despawn>().iter().map(|(e, _)| e).collect();

    // Despawn them
    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

/// Applies acceleration to velocity.
///
/// For each entity with [`Velocity`] and `Acceleration`, applies:
/// `velocity += acceleration * dt`
///
/// Note: Acceleration component is not provided in the default components,
/// as most games use direct velocity modification. Add your own if needed:
///
/// ```ignore
/// #[derive(Clone, Copy)]
/// pub struct Acceleration(pub Vec2);
///
/// pub fn acceleration_system(world: &mut World, dt: f32) {
///     for (_, (mut velocity, accel)) in world.query::<(&mut Velocity, &Acceleration)>() {
///         velocity.0 += accel.0 * dt;
///     }
/// }
/// ```

// =============================================================================
// System Utilities
// =============================================================================

/// Helper to run multiple systems in sequence.
///
/// # Example
///
/// ```ignore
/// run_systems(&mut world, dt, &[
///     movement_system,
///     lifetime_system,
/// ]);
/// despawn_system(&mut world);
/// ```
pub fn run_systems(world: &mut World, dt: f32, systems: &[fn(&mut World, f32)]) {
    for system in systems {
        system(world, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;

    #[test]
    fn test_movement_system() {
        let mut world = World::new();

        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Velocity(Vec2::new(100.0, 50.0)),
        ));

        // Run for 1 second
        movement_system(&mut world, 1.0);

        for (_, transform) in world.query::<&Transform>().iter() {
            assert_eq!(transform.position.x, 100.0);
            assert_eq!(transform.position.y, 50.0);
        }
    }

    #[test]
    fn test_movement_system_partial_dt() {
        let mut world = World::new();

        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Velocity(Vec2::new(100.0, 0.0)),
        ));

        // Run for 0.1 seconds
        movement_system(&mut world, 0.1);

        for (_, transform) in world.query::<&Transform>().iter() {
            assert!((transform.position.x - 10.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_lifetime_marks_for_despawn() {
        let mut world = World::new();

        // Will expire
        let e1 = world.spawn((Transform::default(), Lifetime(0.5)));
        // Won't expire
        let e2 = world.spawn((Transform::default(), Lifetime(2.0)));

        lifetime_system(&mut world, 1.0);

        assert!(world.has::<Despawn>(e1));
        assert!(!world.has::<Despawn>(e2));
    }

    #[test]
    fn test_despawn_system() {
        let mut world = World::new();

        let e1 = world.spawn((Transform::default(), Despawn));
        let e2 = world.spawn((Transform::default(),));

        despawn_system(&mut world);

        assert!(!world.contains(e1));
        assert!(world.contains(e2));
    }

    #[test]
    fn test_lifetime_and_despawn_together() {
        let mut world = World::new();

        // Expires in 0.5s
        world.spawn((Transform::default(), Lifetime(0.5)));
        // Expires in 2.0s
        world.spawn((Transform::default(), Lifetime(2.0)));

        // Run for 1 second
        lifetime_system(&mut world, 1.0);
        despawn_system(&mut world);

        // Only one should remain
        assert_eq!(world.query::<&Transform>().count(), 1);
    }

    #[test]
    fn test_run_systems_helper() {
        let mut world = World::new();

        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Velocity(Vec2::new(100.0, 0.0)),
            Lifetime(0.5),
        ));

        run_systems(&mut world, 1.0, &[movement_system, lifetime_system]);
        despawn_system(&mut world);

        // Entity should be gone (lifetime expired)
        assert!(world.is_empty());
    }
}
