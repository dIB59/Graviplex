use crate::core::color::Color;
use crate::core::math::Vec2;
use crate::ecs::{Sprite, SpriteShape, Transform, Visible, World};
use crate::renderer::{
    Camera2D, CircleInstance, CirclePipeline, LinePipeline, RenderState,
};
use crate::Circle;
use gpu_context::GpuContext;
use wgpu::*;

use super::gpu_context;
use super::line_pipeline::LineInstance;

#[cfg(feature = "textures")]
use super::{SpritePipeline, TextureAtlas};

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
    #[cfg(feature = "textures")]
    pub(crate) sprite_pipeline: Option<&'a mut SpritePipeline>,
    #[cfg(feature = "textures")]
    pub(crate) texture_atlas: Option<&'a TextureAtlas>,
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
    /// use wgpu::util::DeviceExt;
    /// let buffer = gfx.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    ///     label: Some("Particles"),
    ///     contents: bytemuck::cast_slice(&instances),
    ///     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
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
    // ECS RENDERING
    // =========================================================================

    /// Render all visible entities from the ECS world.
    ///
    /// This method queries the world for all entities with [`Transform`], [`Sprite`],
    /// and [`Visible`] components, then batches and renders them efficiently.
    ///
    /// Entities are sorted by z_order (lower values rendered first, higher on top).
    ///
    /// # Texture Sprite Support
    ///
    /// When a texture atlas is registered via [`App::build().atlas()`], texture sprites
    /// are rendered automatically. Without a registered atlas, texture sprites are skipped
    /// with a warning.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn render(&mut self, world: &World, draw: &mut DrawContext) {
    ///     // Render ALL visible entities automatically
    ///     // (circles, rects, lines, AND texture sprites if atlas is registered)
    ///     draw.render_world(world);
    ///
    ///     // You can still draw additional shapes manually
    ///     draw.circle(Circle::new(Vec2::ZERO, 10.0, Color::WHITE));
    /// }
    /// ```
    pub fn render_world(&mut self, world: &World) {
        // Collect all visible entities with their components
        let mut renderables: Vec<(i32, Transform, Sprite)> = world
            .query::<(&Transform, &Sprite, &Visible)>()
            .iter()
            .map(|(_, (transform, sprite, _))| (sprite.z_order, *transform, *sprite))
            .collect();

        // Sort by z_order (lower values first = rendered first = behind)
        renderables.sort_by_key(|(z, _, _)| *z);

        // Track if we've warned about texture sprites (to avoid spam)
        #[cfg(not(feature = "textures"))]
        let mut texture_sprite_warning_shown = false;

        // Batch by shape type for efficient rendering
        let mut circle_batch: Vec<CircleInstance> = Vec::with_capacity(renderables.len());
        let mut line_batch: Vec<LineInstance> = Vec::with_capacity(renderables.len() / 4);

        for (_, transform, sprite) in &renderables {
            match sprite.shape {
                SpriteShape::Circle { radius } => {
                    // Apply transform scale to radius
                    let scaled_radius = radius * transform.scale.x;
                    circle_batch.push(CircleInstance {
                        position: [transform.position.x, transform.position.y],
                        radius: scaled_radius,
                        color: sprite.color.into(),
                    });
                }
                SpriteShape::Rect { size } => {
                    // Draw rectangle as 4 lines (outline)
                    // TODO: Add filled rectangle support via rect_pipeline
                    let half_w = size.x * transform.scale.x * 0.5;
                    let half_h = size.y * transform.scale.y * 0.5;
                    let pos = transform.position;
                    let color: [f32; 4] = sprite.color.into();

                    // For now, approximate as circle with average dimension
                    let avg_radius = (half_w + half_h) * 0.5;
                    circle_batch.push(CircleInstance {
                        position: [pos.x, pos.y],
                        radius: avg_radius,
                        color,
                    });
                }
                SpriteShape::Line { end_offset } => {
                    let start = transform.position;
                    let end = start + end_offset * transform.scale.x;
                    line_batch.push(LineInstance {
                        start: [start.x, start.y],
                        end: [end.x, end.y],
                        color: sprite.color.into(),
                    });
                }   
                SpriteShape::Texture { .. } => {
                    // Texture sprites handled separately below
                    #[cfg(not(feature = "textures"))]
                    if !texture_sprite_warning_shown {
                        log::warn!(
                            "render_world() skips texture sprites - \
                             enable the 'textures' feature and register an atlas via App::build().atlas()"
                        );
                        texture_sprite_warning_shown = true;
                    }
                }
            }
        }

        // Flush primitive batches
        if !circle_batch.is_empty() {
            self.circles(&circle_batch);
        }
        if !line_batch.is_empty() {
            self.lines(&line_batch);
        }

        // Render texture sprites if atlas is registered
        #[cfg(feature = "textures")]
        {
            // Build state before taking mutable borrow of sprite_pipeline
            let state = RenderState {
                gpu: self.gpu,
                view: self.view,
                camera: self.camera,
            };
            
            if let (Some(sprite_pipeline), Some(atlas)) = (&mut self.sprite_pipeline, &self.texture_atlas) {
                for (_, transform, sprite) in &renderables {
                    if let SpriteShape::Texture { region_name, size } = sprite.shape {
                        if let Some(region) = atlas.get(region_name) {
                            sprite_pipeline.draw(
                                [transform.position.x, transform.position.y],
                                [size.x * transform.scale.x, size.y * transform.scale.y],
                                region.uv_rect(),
                                sprite.color.into(),
                                transform.rotation,
                                sprite.z_order,
                            );
                        }
                    }
                }
                sprite_pipeline.flush(&state, atlas);
            }
        }
    }

    /// Render specific entities from the world (for custom rendering logic).
    ///
    /// This allows you to render only a subset of entities, useful for
    /// layered rendering or special effects.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Render only player entities
    /// for (_, (transform, sprite, _)) in world.query::<(&Transform, &Sprite, &Player)>() {
    ///     // Custom rendering logic
    /// }
    /// ```
    pub fn render_entity(&mut self, transform: &Transform, sprite: &Sprite) {
        match sprite.shape {
            SpriteShape::Circle { radius } => {
                let scaled_radius = radius * transform.scale.x;
                self.circle_pipeline.draw_circle(
                    [transform.position.x, transform.position.y],
                    scaled_radius,
                    sprite.color.into(),
                );
            }
            SpriteShape::Rect { size } => {
                // Approximate as circle for now
                let half_w = size.x * transform.scale.x * 0.5;
                let half_h = size.y * transform.scale.y * 0.5;
                let avg_radius = (half_w + half_h) * 0.5;
                self.circle_pipeline.draw_circle(
                    [transform.position.x, transform.position.y],
                    avg_radius,
                    sprite.color.into(),
                );
            }
            SpriteShape::Line { end_offset } => {
                let start = transform.position;
                let end = start + end_offset * transform.scale.x;
                self.line_pipeline.draw_line(
                    [start.x, start.y],
                    [end.x, end.y],
                    sprite.color.into(),
                );
            }
            SpriteShape::Texture { .. } => {
                // Texture sprites require SpritePipeline with atlas
                // Use draw_sprite() or render_world_with_atlas() instead
                log::warn!("Texture sprites not supported in render_entity(), use SpritePipeline");
            }
        }
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
