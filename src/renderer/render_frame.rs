//! RenderFrame - Single-encoder frame rendering abstraction.
//!
//! This module provides a `RenderFrame` that collects all draw commands and submits
//! them to the GPU as a single command buffer, eliminating the overhead of multiple
//! encoder creations and submissions per frame.

use wgpu::{CommandEncoder, CommandEncoderDescriptor, Queue, TextureView};

use super::gpu_context::GpuContext;
use super::render_state::RenderState;
use super::Camera2D;

/// A frame rendering context that batches all GPU commands into a single submission.
///
/// `RenderFrame` holds a single `CommandEncoder` that is shared across all pipeline
/// flushes. This eliminates the overhead of creating multiple encoders and submitting
/// multiple command buffers per frame.
///
/// # Usage
///
/// ```ignore
/// // In App::render_frame():
/// let mut frame = RenderFrame::begin(&gpu, &view);
///
/// // All pipeline flushes use the shared encoder
/// circle_pipeline.flush_to(&mut frame);
/// line_pipeline.flush_to(&mut frame);
/// sprite_pipeline.flush_to(&mut frame);
///
/// // Single GPU submission at end of frame
/// frame.submit();
/// ```
pub struct RenderFrame<'a> {
    encoder: CommandEncoder,
    queue: &'a Queue,
    view: &'a TextureView,
    camera: &'a Camera2D,
}

impl<'a> RenderFrame<'a> {
    /// Begin a new render frame.
    pub fn begin(gpu: &'a GpuContext, view: &'a TextureView, camera: &'a Camera2D) -> Self {
        let encoder = gpu.raw_device().create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Frame Encoder"),
        });
        
        Self {
            encoder,
            queue: gpu.raw_queue(),
            view,
            camera,
        }
    }

    /// Get the GPU queue for buffer writes.
    #[inline]
    pub fn queue(&self) -> &Queue {
        self.queue
    }

    /// Get the render target view.
    #[inline]
    pub fn view(&self) -> &TextureView {
        self.view
    }

    /// Get the camera.
    #[inline]
    pub fn camera(&self) -> &Camera2D {
        self.camera
    }

    /// Create a RenderState (for pipelines that still need it).
    pub fn render_state(&self, gpu: &'a GpuContext) -> RenderState<'a> {
        RenderState {
            gpu,
            view: self.view,
            camera: self.camera,
        }
    }

    /// Execute a closure that needs the encoder and other frame data.
    /// 
    /// This provides simultaneous access to both the encoder and the view/camera,
    /// avoiding borrow checker issues when creating render passes.
    #[inline]
    pub fn with_encoder<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut CommandEncoder, &TextureView, &Camera2D, &Queue) -> R,
    {
        f(&mut self.encoder, self.view, self.camera, self.queue)
    }

    /// Submit all recorded commands to the GPU.
    ///
    /// This consumes the RenderFrame and submits the accumulated command buffer.
    pub fn submit(self) {
        self.queue.submit(std::iter::once(self.encoder.finish()));
    }
}

/// Extension trait for pipelines to support RenderFrame-based rendering.
pub trait PipelineFlush {
    /// Flush the pipeline's batched instances to the given RenderFrame.
    fn flush_to(&mut self, frame: &mut RenderFrame<'_>);
}
