use crate::renderer::{
    Camera2D, CircleInstance, CirclePipeline, GpuContext, LineInstance, LinePipeline,
};
use wgpu::*;

/// A Raylib-like drawing context that handles batching and performance optimization.
pub struct DrawContext<'a> {
    pub gpu: &'a GpuContext,
    pub view: &'a TextureView,
    pub camera: &'a Camera2D,
    pub circle_pipeline: &'a mut CirclePipeline,
    pub line_pipeline: &'a mut LinePipeline,
}

impl<'a> DrawContext<'a> {
    /// Draw a single circle. This will be batched.
    pub fn draw_circle(&mut self, position: [f32; 2], radius: f32, color: [f32; 4]) {
        self.circle_pipeline.draw_circle(position, radius, color);
    }

    /// Draw multiple circles efficiently.
    pub fn draw_circles(&mut self, instances: &[CircleInstance]) {
        self.circle_pipeline.draw_circles(instances);
    }

    /// High performance path for drawing 1M+ particles directly from a GPU buffer.
    /// This bypasses CPU-side batching.
    pub fn draw_circles_raw(&self, buffer: &Buffer, count: u32) {
        self.circle_pipeline.render_with_external_buffer(
            &self.gpu.device,
            &self.gpu.queue,
            self.view,
            self.camera,
            count,
            buffer,
        );
    }

    /// Draw a single line. This will be batched.
    pub fn draw_line(&mut self, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
        self.line_pipeline.draw_line(start, end, color);
    }

    /// Draw multiple lines efficiently.
    pub fn draw_lines(&mut self, instances: &[LineInstance]) {
        self.line_pipeline.draw_lines(instances);
    }

    /// High performance path for drawing lines directly from a GPU buffer.
    pub fn draw_lines_raw(&self, buffer: &Buffer, count: u32) {
        self.line_pipeline.render_with_external_buffer(
            &self.gpu.device,
            &self.gpu.queue,
            self.view,
            self.camera,
            count,
            buffer,
        );
    }

    /// Flush all batched draw calls to the GPU.
    /// The engine calls this automatically at the end of the render pass.
    pub fn flush(&mut self) {
        self.circle_pipeline
            .flush(&self.gpu.device, &self.gpu.queue, self.view, self.camera);
        self.line_pipeline
            .flush(&self.gpu.device, &self.gpu.queue, self.view, self.camera);
    }
}

#[cfg(test)]
mod tests {
    // Note: Most DrawContext logic requires a GpuContext and Pipelines which require a GPU.
    // Integration tests should be used for full rendering verification.
}
