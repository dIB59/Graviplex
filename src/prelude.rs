//! Graviplex prelude - import all essential types with a single `use graviplex::prelude::*`
//!
//! This module re-exports the most commonly used types for game development:
//! - [`App`] and [`AppBuilder`] - Application entry point and configuration
//! - [`GameLoop`] - Trait for implementing your game
//! - [`DrawContext`] - Immediate-mode drawing API
//! - [`Vec2`], [`Color`] - Math primitives
//! - [`Circle`], [`Rect`] - Geometry types
//! - [`Time`] - Frame timing information
//! - [`InputState`] - Keyboard/mouse input
//! - [`Camera2D`] - 2D camera with pan/zoom
//! - ECS types - [`World`], [`Entity`], components, and systems

// Core application
pub use crate::app::{App, AppBuilder, CameraConfig};
pub use crate::game_loop::GameLoop;

// Drawing
pub use crate::renderer::DrawContext;

// Math primitives
pub use crate::core::color::Color;
pub use crate::core::math::Vec2;

// Geometry
pub use crate::core::geometry::{Circle, Rect};

// Time
pub use crate::core::time::Time;

// Input
pub use crate::input::InputState;

// Camera
pub use crate::renderer::{Camera2D, CameraFollow};

// Stats
pub use crate::core::stats::{AppStats, FpsCounter, FpsDisplayConfig};

// Plugins
pub use crate::plugin::Plugin;
#[cfg(feature = "gui")]
pub use crate::plugin::FpsPlugin;

// Graphics (renamed from GpuContext)
pub use crate::renderer::Graphics;

// Texture atlas builder (for use with App::build().atlas())
#[cfg(feature = "textures")]
pub use crate::renderer::{AtlasBuilder, RegionId};

// ECS - Core types
pub use crate::ecs::{Entity, Resources, SystemContext, World};

// ECS - Built-in components
pub use crate::ecs::{
    Acceleration, BodyType, Despawn, Gravity, Health, Lifetime, Name, PlayerController, RigidBody,
    Sprite, SpriteAnimation, SpriteShape, Tags, Transform, Velocity, Visible,
};
#[cfg(feature = "textures")]
pub use crate::ecs::SpriteAnimationId;

// ECS - Tilemap support
pub use crate::ecs::{AutoTileConfig, NeighborFlags, Tile, TileAnimation, TileCollision, TileFlip, TileLayer, Tilemap};

// ECS - Traits for queries and components (re-exported from hecs)
pub use crate::ecs::{Component, Query, Ref, RefMut};

// ECS - Built-in systems
pub use crate::ecs::{
    acceleration_system, animation_system, despawn_system, drag_system, gravity_system,
    lifetime_system, movement_system, physics_system, player_input_system,
};
#[cfg(feature = "textures")]
pub use crate::ecs::animation_id_system;

// Collision detection
pub use crate::collision::{Collider, ColliderShape, CollisionEvent, CollisionLayer};
pub use crate::collision::{collision_system, point_query, raycast};

// Scene management
pub use crate::scene::{Scene, SceneManager, Transition};

// Events
pub use crate::events::{EventBus, Events};

// State machine
pub use crate::state_machine::{SimpleState, StateId, StateMachine};

// Audio (placeholder)
pub use crate::audio::{AudioClip, AudioListener, AudioManager, AudioSource};

/// Convenient access to all built-in systems.
pub mod systems {
    pub use crate::collision::{collision_system, point_query, raycast};
    pub use crate::ecs::{
        acceleration_system, animation_system, despawn_system, drag_system, gravity_system,
        lifetime_system, movement_system, physics_system, player_input_system, run_systems,
    };
    #[cfg(feature = "textures")]
    pub use crate::ecs::animation_id_system;
}
