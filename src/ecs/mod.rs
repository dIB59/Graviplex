//! Graviplex ECS - Entity Component System
//!
//! This module provides a thin abstraction over the ECS backend (currently `hecs`).
//! All game code should use these Graviplex types, never the underlying library directly.
//! This enables swapping to a custom ECS implementation later without breaking user code.
//!
//! # Quick Start
//!
//! ```ignore
//! use graviplex::prelude::*;
//!
//! struct MyGame;
//!
//! impl GameLoop for MyGame {
//!     fn init(&mut self, world: &mut World, _gfx: &Graphics) {
//!         // Spawn entities with components
//!         world.spawn((
//!             Transform::from_position(Vec2::ZERO),
//!             Velocity(Vec2::new(100.0, 0.0)),
//!             Sprite::circle(50.0, Color::RED),
//!             Visible,
//!         ));
//!     }
//!
//!     fn update(&mut self, world: &mut World, res: &Resources) {
//!         // Run built-in systems
//!         systems::movement(world, res.time.delta());
//!     }
//!
//!     fn render(&mut self, world: &World, draw: &mut DrawContext) {
//!         // Automatic batched rendering of all visible entities
//!         draw.render_world(world);
//!     }
//! }
//! ```
//!
//! # Architecture
//!
//! - [`World`] - Container for all entities and components
//! - [`Entity`] - Lightweight handle to an entity
//! - [`components`] - Built-in component types (Transform, Sprite, etc.)
//! - [`systems`] - Built-in systems (movement, lifetime, etc.)
//! - [`Resources`] - Per-frame resources (Time, Input, Camera)

mod components;
mod entity;
mod resources;
pub mod systems;
pub mod tilemap;
mod world;

// Primary exports
pub use components::{
    Acceleration, BodyType, Despawn, Gravity, Health, Lifetime, Name, PlayerController, RigidBody, Sprite,
    SpriteAnimation, SpriteShape, Tags, Transform, Velocity, Visible,
};
#[cfg(feature = "textures")]
pub use components::SpriteAnimationId;
pub use entity::Entity;
pub use resources::Resources;
pub use systems::{
    acceleration_system, animation_system, despawn_system, drag_system, gravity_system,
    lifetime_system, movement_system, physics_system, player_input_system, run_systems,
};
#[cfg(feature = "textures")]
pub use systems::animation_id_system;
pub use tilemap::{AutoTileConfig, NeighborFlags, Tile, TileAnimation, TileCollision, TileFlip, TileLayer, Tilemap};
pub use world::{EntityBuilder, World};

// Re-export hecs traits needed for queries and components
// This allows users to write `use graviplex::ecs::Query` instead of importing hecs
pub use hecs::{Component, Query, Ref, RefMut};

/// Re-export systems module for namespaced access: `ecs::systems::movement()`
pub mod prelude {
    pub use super::components::{
        Acceleration, BodyType, Despawn, Gravity, Health, Lifetime, Name, PlayerController, RigidBody, Sprite,
        SpriteAnimation, SpriteShape, Tags, Transform, Velocity, Visible,
    };
    #[cfg(feature = "textures")]
    pub use super::components::SpriteAnimationId;
    pub use super::entity::Entity;
    pub use super::resources::Resources;
    pub use super::systems;
    pub use super::tilemap::{AutoTileConfig, NeighborFlags, Tile, TileAnimation, TileCollision, TileFlip, TileLayer, Tilemap};
    pub use super::world::{EntityBuilder, World};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::core::math::Vec2;

    #[test]
    fn test_spawn_and_query() {
        let mut world = World::new();

        let entity = world.spawn((
            Transform::from_position(Vec2::new(10.0, 20.0)),
            Velocity(Vec2::new(1.0, 2.0)),
        ));

        assert!(world.contains(entity));

        // Query single component
        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.position.x, 10.0);
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();

        let entity = world.spawn((Transform::default(),));
        assert!(world.contains(entity));

        world.despawn(entity).unwrap();
        assert!(!world.contains(entity));
    }

    #[test]
    fn test_movement_system() {
        let mut world = World::new();

        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Velocity(Vec2::new(100.0, 50.0)),
        ));

        // Run movement for 1 second
        movement_system(&mut world, 1.0);

        // Check position updated
        for (_, transform) in world.query::<&Transform>().iter() {
            assert_eq!(transform.position.x, 100.0);
            assert_eq!(transform.position.y, 50.0);
        }
    }

    #[test]
    fn test_lifetime_and_despawn_systems() {
        let mut world = World::new();

        // Entity that should expire
        world.spawn((Transform::default(), Lifetime(0.5)));

        // Entity that should survive
        world.spawn((Transform::default(), Lifetime(2.0)));

        // Run for 1 second
        lifetime_system(&mut world, 1.0);
        despawn_system(&mut world);

        // Only one entity should remain
        assert_eq!(world.query::<&Transform>().iter().count(), 1);
    }

    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite::circle(50.0, Color::RED);
        assert!(matches!(sprite.shape, SpriteShape::Circle { radius: 50.0 }));
        assert_eq!(sprite.z_order, 0);

        let sprite = Sprite::rect(100.0, 50.0, Color::BLUE).with_z_order(10);
        assert!(matches!(sprite.shape, SpriteShape::Rect { .. }));
        assert_eq!(sprite.z_order, 10);
    }
}
