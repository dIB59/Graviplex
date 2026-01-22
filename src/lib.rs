//! Graviplex Engine - A 2D GPU-accelerated game engine
//!
//! # Quick Start
//!
//! ```no_run
//! use graviplex::prelude::*;
//!
//! struct MyGame;
//!
//! impl GameLoop for MyGame {
//!     fn init(&mut self, _gfx: &Graphics) {}
//!     fn update(&mut self, time: &Time, _gfx: &Graphics) {}
//!     fn render(&mut self, draw: &mut DrawContext) {
//!         draw.circle(Circle::new(Vec2::ZERO, 50.0, Color::RED));
//!     }
//! }
//!
//! fn main() {
//!     App::build(MyGame)
//!         .title("My Game")
//!         .size(1280, 720)
//!         .run()
//!         .unwrap();
//! }
//! ```
//!
//! # Module Organization
//!
//! - [`prelude`] - Import all essential types with `use graviplex::prelude::*`
//! - [`core`] - Math primitives, colors, geometry, time
//! - [`renderer`] - Drawing context, pipelines, camera
//! - [`input`] - Keyboard and mouse input handling
//! - [`gui`] - egui integration (feature-gated)

// Public modules
pub mod core;
pub mod input;
pub mod prelude;
pub mod renderer;

// Feature-gated modules
#[cfg(feature = "gui")]
pub mod gui;

// Private modules (implementation details)
mod app;
mod game_loop;

// =============================================================================
// PUBLIC API - Primary types users interact with
// =============================================================================

// Application
pub use app::{App, AppBuilder, CameraConfig};
pub use game_loop::GameLoop;

// Drawing
pub use renderer::DrawContext;

// Math & Geometry
pub use core::color::Color;
pub use core::geometry::{Circle, Rect};
pub use core::math::Vec2;

// Time & Stats
pub use core::stats::AppStats;
pub use core::time::Time;

// Input
pub use input::InputState;

// Camera
pub use renderer::{Camera2D, CameraController};

// Graphics device (for advanced users creating custom buffers/pipelines)
pub use renderer::Graphics;

// =============================================================================
// ADVANCED API - For users who need lower-level access
// =============================================================================

/// Advanced types for custom rendering pipelines and GPU buffer management.
///
/// Most users won't need these - they're for performance-critical scenarios
/// like rendering 1M+ particles with custom GPU buffers.
pub mod advanced {
    pub use crate::renderer::{
        CameraGpuData, CircleInstance, CirclePipeline, LineInstance, LinePipeline, RenderState,
        ShaderPipeline, Vertex,
    };

    #[cfg(feature = "physics")]
    pub use crate::physics::PhysicsInstance;
}

// =============================================================================
// FEATURE-GATED MODULES
// =============================================================================

/// Physics simulation types (requires `physics` feature).
#[cfg(feature = "physics")]
pub mod physics;

// =============================================================================
// RE-EXPORTED DEPENDENCIES (for advanced use cases)
// =============================================================================

/// Re-exported egui for GUI (requires `gui` feature).
#[cfg(feature = "gui")]
pub use egui;

/// Re-exported wgpu for advanced GPU programming.
pub use wgpu;

/// Convenience macro for including WGSL shaders.
pub use wgpu::include_wgsl;

/// Re-exported winit for window handling.
pub use winit;
