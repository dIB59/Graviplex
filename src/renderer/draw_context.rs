use crate::core::color::Color;
use crate::core::math::Vec2;
use crate::renderer::{
    Camera2D, CircleInstance, CirclePipeline, LinePipeline, RenderState,
};
use crate::Circle;
use gpu_context::GpuContext;
use wgpu::*;

use super::gpu_context;
use super::line_pipeline::LineInstance;

// =============================================================================
// CIRCLE DRAWING PARAMS - Flexible input types
// =============================================================================

/// Parameters for drawing a circle. Supports multiple input types via `Into`.
#[derive(Clone, Copy, Debug)]
pub struct CircleParams {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
}

impl From<Circle> for CircleParams {
    fn from(c: Circle) -> Self {
        Self {
            position: c.center.into(),
            radius: c.radius,
            color: c.color.into(),
        }
    }
}

impl From<&Circle> for CircleParams {
    fn from(c: &Circle) -> Self {
        Self {
            position: c.center.into(),
            radius: c.radius,
            color: c.color.into(),
        }
    }
}

impl From<(Vec2, f32, Color)> for CircleParams {
    fn from((pos, radius, color): (Vec2, f32, Color)) -> Self {
        Self {
            position: pos.into(),
            radius,
            color: color.into(),
        }
    }
}

impl From<([f32; 2], f32, [f32; 4])> for CircleParams {
    fn from((position, radius, color): ([f32; 2], f32, [f32; 4])) -> Self {
        Self {
            position,
            radius,
            color,
        }
    }
}

impl From<CircleInstance> for CircleParams {
    fn from(c: CircleInstance) -> Self {
        Self {
            position: c.position,
            radius: c.radius,
            color: c.color,
        }
    }
}

// =============================================================================
// LINE DRAWING PARAMS - Flexible input types
// =============================================================================

/// Parameters for drawing a line. Supports multiple input types via `Into`.
#[derive(Clone, Copy, Debug)]
pub struct LineParams {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
}

impl From<(Vec2, Vec2, Color)> for LineParams {
    fn from((start, end, color): (Vec2, Vec2, Color)) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
            color: color.into(),
        }
    }
}

impl From<([f32; 2], [f32; 2], [f32; 4])> for LineParams {
    fn from((start, end, color): ([f32; 2], [f32; 2], [f32; 4])) -> Self {
        Self { start, end, color }
    }
}

impl From<LineInstance> for LineParams {
    fn from(l: LineInstance) -> Self {
        Self {
            start: l.start,
            end: l.end,
            color: l.color,
        }
    }
}

// =============================================================================
// DRAW CONTEXT - Immediate-mode drawing API
// =============================================================================

/// Immediate-mode drawing context for 2D graphics.
///
/// `DrawContext` provides a simple, Raylib-inspired API for drawing shapes.
/// All draw calls are batched and flushed automatically at frame end.
///
/// # Example
///
/// ```ignore
/// fn render(&mut self, draw: &mut DrawContext) {
///     // Using rich types
///     draw.circle(Circle::new(Vec2::ZERO, 50.0, Color::RED));
///     
///     // Using tuples
///     draw.circle((Vec2::new(100.0, 0.0), 30.0, Color::BLUE));
///     
///     // Using raw arrays
///     draw.circle(([200.0, 0.0], 25.0, [0.0, 1.0, 0.0, 1.0]));
///     
///     // Lines work the same way
///     draw.line((Vec2::ZERO, Vec2::new(100.0, 100.0), Color::WHITE));
/// }
/// ```
pub struct DrawContext<'a> {
    pub(crate) gpu: &'a GpuContext,
    pub(crate) view: &'a TextureView,
    pub(crate) camera: &'a Camera2D,
    pub(crate) circle_pipeline: &'a mut CirclePipeline,
    pub(crate) line_pipeline: &'a mut LinePipeline,
}

impl<'a> DrawContext<'a> {
    /// Get access to the GPU context for advanced operations.
    pub fn graphics(&self) -> &GpuContext {
        self.gpu
    }

    /// Get the current camera.
    pub fn camera(&self) -> &Camera2D {
        self.camera
    }

    /// Create a simplified RenderState for custom pipeline calls.
    pub fn state(&self) -> RenderState<'a> {
        RenderState {
            gpu: self.gpu,
            view: self.view,
            camera: self.camera,
        }
    }

    // =========================================================================
    // CIRCLE DRAWING
    // =========================================================================

    /// Draw a circle.
    ///
    /// Accepts multiple input types:
    /// - `Circle` - Rich domain type
    /// - `(Vec2, f32, Color)` - Tuple with engine types
    /// - `([f32; 2], f32, [f32; 4])` - Raw arrays
    /// - `CircleInstance` - GPU instance type
    ///
    /// # Example
    ///
    /// ```ignore
    /// draw.circle(Circle::new(Vec2::ZERO, 50.0, Color::RED));
    /// draw.circle((Vec2::new(100.0, 0.0), 30.0, Color::BLUE));
    /// draw.circle(([200.0, 0.0], 25.0, [0.0, 1.0, 0.0, 1.0]));
    /// ```
    pub fn circle(&mut self, params: impl Into<CircleParams>) {
        let p = params.into();
        self.circle_pipeline.draw_circle(p.position, p.radius, p.color);
    }

    /// Draw multiple circles efficiently from a slice of `CircleInstance`.
    ///
    /// For large batches (1000+), this is more efficient than individual `circle()` calls.
    pub fn circles(&mut self, instances: &[CircleInstance]) {
        self.circle_pipeline.draw_circles(instances);
    }

    /// Draw circles directly from a GPU buffer.
    ///
    /// This is the highest-performance path for rendering 100K+ particles.
    /// Use this when you have a pre-populated GPU buffer from compute shaders.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
    ///     label: Some("Particles"),
    ///     contents: bytemuck::cast_slice(&instances),
    ///     usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
    /// });
    /// draw.circles_from_buffer(&buffer, 1_000_000);
    /// ```
    pub fn circles_from_buffer(&self, buffer: &Buffer, count: u32) {
        self.circle_pipeline
            .render_with_external_buffer(&self.state(), count, buffer);
    }

    // =========================================================================
    // LINE DRAWING
    // =========================================================================

    /// Draw a line.
    ///
    /// Accepts multiple input types:
    /// - `(Vec2, Vec2, Color)` - Tuple with engine types
    /// - `([f32; 2], [f32; 2], [f32; 4])` - Raw arrays
    /// - `LineInstance` - GPU instance type
    ///
    /// # Example
    ///
    /// ```ignore
    /// draw.line((Vec2::ZERO, Vec2::new(100.0, 100.0), Color::WHITE));
    /// draw.line(([-50.0, 0.0], [50.0, 0.0], [1.0, 0.0, 0.0, 1.0]));
    /// ```
    pub fn line(&mut self, params: impl Into<LineParams>) {
        let p = params.into();
        self.line_pipeline.draw_line(p.start, p.end, p.color);
    }

    /// Draw multiple lines efficiently from a slice of `LineInstance`.
    pub fn lines(&mut self, instances: &[LineInstance]) {
        self.line_pipeline.draw_lines(instances);
    }

    /// Draw lines directly from a GPU buffer.
    pub fn lines_from_buffer(&self, buffer: &Buffer, count: u32) {
        self.line_pipeline
            .render_with_external_buffer(&self.state(), count, buffer);
    }

    // =========================================================================
    // DEPRECATED - Old API (kept for backwards compatibility)
    // =========================================================================

    /// Draw a circle using the rich Circle domain model.
    #[deprecated(since = "0.3.0", note = "Use `circle()` instead")]
    pub fn draw_circle(&mut self, circle: Circle) {
        self.circle(circle);
    }

    /// Draw a circle with raw components.
    #[deprecated(since = "0.3.0", note = "Use `circle()` instead")]
    pub fn draw_circle_raw(&mut self, position: [f32; 2], radius: f32, color: [f32; 4]) {
        self.circle((position, radius, color));
    }

    /// Draw a circle with raw components (alias).
    #[deprecated(since = "0.3.0", note = "Use `circle()` instead")]
    pub fn draw_circle_with_components(
        &mut self,
        position: [f32; 2],
        radius: f32,
        color: [f32; 4],
    ) {
        self.circle((position, radius, color));
    }

    /// Draw multiple circles.
    #[deprecated(since = "0.3.0", note = "Use `circles()` instead")]
    pub fn draw_circles(&mut self, instances: &[CircleInstance]) {
        self.circles(instances);
    }

    /// Draw circles from a GPU buffer.
    #[deprecated(since = "0.3.0", note = "Use `circles_from_buffer()` instead")]
    pub fn draw_circles_raw(&self, buffer: &Buffer, count: u32) {
        self.circles_from_buffer(buffer, count);
    }

    /// Draw a line with raw components.
    #[deprecated(since = "0.3.0", note = "Use `line()` instead")]
    pub fn draw_line(&mut self, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
        self.line((start, end, color));
    }

    /// Draw multiple lines.
    #[deprecated(since = "0.3.0", note = "Use `lines()` instead")]
    pub fn draw_lines(&mut self, instances: &[LineInstance]) {
        self.lines(instances);
    }

    /// Draw lines from a GPU buffer.
    #[deprecated(since = "0.3.0", note = "Use `lines_from_buffer()` instead")]
    pub fn draw_lines_raw(&self, buffer: &Buffer, count: u32) {
        self.lines_from_buffer(buffer, count);
    }

    // =========================================================================
    // INTERNAL
    // =========================================================================

    /// Flush all batched draw calls to the GPU.
    /// The engine calls this automatically at the end of the render pass.
    pub fn flush(&mut self) {
        let state = self.state();
        self.circle_pipeline.flush(&state);
        self.line_pipeline.flush(&state);
    }
}

#[cfg(test)]
mod tests {
    // Note: Most DrawContext logic requires a GpuContext and Pipelines which require a GPU.
    // Integration tests should be used for full rendering verification.
}
