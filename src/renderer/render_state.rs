use crate::renderer::{Camera2D, GpuContext};
use wgpu::TextureView;

/// A bundle of state needed for a single render pass.
///
/// This simplifies passing around common rendering resources.
pub struct RenderState<'a> {
    pub gpu: &'a GpuContext,
    pub view: &'a TextureView,
    pub camera: &'a Camera2D,
}
