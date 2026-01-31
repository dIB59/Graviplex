//! Rendering subsystem for Graviplex engine.
//!
//! This module provides:
//! - [`Graphics`] - GPU device and queue management
//! - [`DrawContext`] - Immediate-mode drawing API
//! - [`Camera2D`] - 2D camera with pan/zoom support
//! - [`CameraFollow`] - Smooth camera following with offset limits
//! - [`RenderFrame`] - Single-encoder frame rendering abstraction
//! - Pipeline implementations for circles, lines, and custom shaders

pub mod camera;
pub mod circle_pipeline;
pub mod draw_context;
pub mod gpu_context;
pub mod line_pipeline;
pub mod pipeline;
pub mod render_frame;
pub mod render_state;
pub mod sprite_pipeline;
pub mod texture_atlas;
pub mod vertex_data;

// Primary exports
pub use camera::{Camera2D, CameraController, CameraFollow, CameraGpuData};
pub use draw_context::DrawContext;
pub use gpu_context::GpuContext as Graphics;
pub use render_frame::{PipelineFlush, RenderFrame};

// Pipeline exports
pub use circle_pipeline::CirclePipeline;
pub use line_pipeline::{LineInstance, LinePipeline};
pub use pipeline::RenderPipeline as ShaderPipeline;
pub use sprite_pipeline::SpritePipeline;
pub use texture_atlas::{AtlasBuilder, AtlasError, AtlasRegion, RegionId, TextureAtlas};

// Internal types (for advanced module)
pub use render_state::RenderState;
pub use vertex_data::{CircleInstance, Vertex};
pub use vertex_data::{SpriteInstance, SpriteInstanceGpu};
