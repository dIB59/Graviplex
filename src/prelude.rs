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
pub use crate::renderer::Camera2D;

// Stats
pub use crate::core::stats::AppStats;

// Graphics (renamed from GpuContext)
pub use crate::renderer::Graphics;

// Texture atlas builder (for use with App::build().atlas())
#[cfg(feature = "textures")]
pub use crate::renderer::AtlasBuilder;

// ECS - Core types
pub use crate::ecs::{Entity, Resources, World};

// ECS - Built-in components
pub use crate::ecs::{Despawn, Lifetime, Sprite, SpriteShape, Transform, Velocity, Visible};

// ECS - Traits for queries and components (re-exported from hecs)
pub use crate::ecs::{Component, Query, Ref, RefMut};

// ECS - Built-in systems (namespaced)
pub use crate::ecs::{despawn_system, lifetime_system, movement_system};

/// Convenient access to all built-in systems.
pub mod systems {
    pub use crate::ecs::{despawn_system, lifetime_system, movement_system, run_systems};
}
