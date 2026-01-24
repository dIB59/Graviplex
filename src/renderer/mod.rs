//! Rendering subsystem for Graviplex engine.
//!
//! This module provides:
//! - [`Graphics`] - GPU device and queue management
//! - [`DrawContext`] - Immediate-mode drawing API
//! - [`Camera2D`] - 2D camera with pan/zoom support
//! - [`CameraFollow`] - Smooth camera following with offset limits
//! - Pipeline implementations for circles, lines, and custom shaders

pub mod camera;
pub mod circle_pipeline;
pub mod draw_context;
pub mod gpu_context;
pub mod line_pipeline;
pub mod pipeline;
pub mod render_state;
#[cfg(feature = "textures")]
pub mod sprite_pipeline;
#[cfg(feature = "textures")]
pub mod texture_atlas;
pub mod vertex_data;

// Primary exports
pub use camera::{Camera2D, CameraController, CameraFollow, CameraGpuData};
pub use draw_context::DrawContext;
pub use gpu_context::GpuContext as Graphics;

// Pipeline exports
pub use circle_pipeline::CirclePipeline;
pub use line_pipeline::{LineInstance, LinePipeline};
pub use pipeline::RenderPipeline as ShaderPipeline;
#[cfg(feature = "textures")]
pub use sprite_pipeline::SpritePipeline;
#[cfg(feature = "textures")]
pub use texture_atlas::{AtlasBuilder, AtlasError, AtlasRegion, TextureAtlas};

// Internal types (for advanced module)
pub use render_state::RenderState;
pub use vertex_data::{CircleInstance, Vertex};
#[cfg(feature = "textures")]
pub use vertex_data::{SpriteInstance, SpriteInstanceGpu};
