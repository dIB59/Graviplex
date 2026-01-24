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
//! # Schedule-Based Systems (New)
//!
//! For more control over system execution order, use the Schedule:
//!
//! ```ignore
//! use graviplex::ecs::{Schedule, CoreStage, IntoSystem};
//!
//! let mut schedule = Schedule::new();
//! schedule
//!     .add_system(CoreStage::Update, my_movement_system)
//!     .add_system(CoreStage::PostUpdate, my_physics_system)
//!     .add_system(CoreStage::Last, despawn_system);
//!
//! // In your game loop:
//! schedule.run(&mut world);
//! ```
//!
//! # Architecture
//!
//! - [`World`] - Container for all entities and components
//! - [`Entity`] - Lightweight handle to an entity
//! - [`components`] - Built-in component types (Transform, Sprite, etc.)
//! - [`systems`] - Built-in systems (movement, lifetime, etc.)
//! - [`Resources`] - Per-frame resources (Time, Input, Camera)
//! - [`SystemContext`] - Unified context for systems (optional)
//! - [`Schedule`] - System execution scheduler with stages
//! - [`Commands`] - Deferred entity operations
//! - [`Res`], [`ResMut`] - Resource accessors for systems

// Core modules
mod change_detection;
mod commands;
mod components;
mod context;
mod entity;
mod param;
mod resources;
mod schedule;
mod system;
pub mod systems;
pub mod tilemap;
mod world;

// Primary exports - Components
pub use components::{
    Acceleration, BodyType, Despawn, Gravity, Health, Lifetime, Name, PlayerController, RigidBody, Sprite,
    SpriteAnimation, SpriteShape, Tags, Transform, Velocity, Visible,
};
#[cfg(feature = "textures")]
pub use components::SpriteAnimationId;

// Primary exports - Legacy system context (for backwards compatibility)
pub use context::{run_context_systems, System, SystemContext};

// Primary exports - Core types
pub use entity::Entity;
pub use resources::Resources;
pub use world::{EntityBuilder, World};

// Primary exports - Built-in systems
pub use systems::{
    acceleration_system, animation_system, despawn_system, drag_system, gravity_system,
    lifetime_system, movement_system, physics_system, player_input_system, run_systems,
};
#[cfg(feature = "textures")]
pub use systems::animation_id_system;

// Primary exports - Tilemap
pub use tilemap::{AutoTileConfig, NeighborFlags, Tile, TileAnimation, TileCollision, TileFlip, TileLayer, Tilemap};

// New architecture exports - System parameters
pub use param::{Access, Local, Res, ResMut, SystemParam};

// New architecture exports - System trait and conversion
pub use system::{BoxedSystem, IntoSystem, SystemSet, SystemTrait};

// New architecture exports - Schedule and stages
pub use schedule::{CoreStage, Schedule, Stage, StartupSchedule};

// New architecture exports - Commands
pub use commands::{CommandQueue, Commands};

// New architecture exports - Change detection
pub use change_detection::{Added, Changed, ChangeTick, ChangeTrackers, Tick};

// Re-export hecs traits needed for queries and components
// This allows users to write `use graviplex::ecs::Query` instead of importing hecs
pub use hecs::{Component, Query, Ref, RefMut};

/// Re-export systems module for namespaced access: `ecs::systems::movement()`
pub mod prelude {
    // Components
    pub use super::components::{
        Acceleration, BodyType, Despawn, Gravity, Health, Lifetime, Name, PlayerController, RigidBody, Sprite,
        SpriteAnimation, SpriteShape, Tags, Transform, Velocity, Visible,
    };
    #[cfg(feature = "textures")]
    pub use super::components::SpriteAnimationId;
    
    // Legacy context
    pub use super::context::{run_context_systems, System, SystemContext};
    
    // Core types
    pub use super::entity::Entity;
    pub use super::resources::Resources;
    pub use super::systems;
    pub use super::tilemap::{AutoTileConfig, NeighborFlags, Tile, TileAnimation, TileCollision, TileFlip, TileLayer, Tilemap};
    pub use super::world::{EntityBuilder, World};
    
    // New architecture - system parameters
    pub use super::param::{Res, ResMut, Local};
    
    // New architecture - systems
    pub use super::system::{IntoSystem, SystemTrait};
    
    // New architecture - schedule
    pub use super::schedule::{CoreStage, Schedule, StartupSchedule};
    
    // New architecture - commands
    pub use super::commands::Commands;
    
    // New architecture - change detection
    pub use super::change_detection::{Changed, Added, Tick};
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
