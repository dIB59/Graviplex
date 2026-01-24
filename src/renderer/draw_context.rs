use crate::core::color::Color;
use crate::core::math::Vec2;
use crate::ecs::{Sprite, SpriteShape, Transform, Visible, World};
use crate::renderer::{
    Camera2D, CircleInstance, CirclePipeline, LinePipeline, RenderFrame, RenderState,
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
    // TEXTURE DRAWING
    // =========================================================================

    /// Draw a texture at the specified position.
    ///
    /// Requires the `textures` feature and a registered texture atlas.
    ///
    /// # Arguments
    /// * `texture_name` - Name of the texture region in the atlas
    /// * `position` - World position (center of the texture)
    /// * `size` - Size to render (or None to use the texture's original size)
    /// * `tint` - Color tint to apply (use Color::WHITE for no tint)
    ///
    /// # Example
    ///
    /// ```ignore
    /// draw.texture("player", Vec2::new(100.0, 50.0), None, Color::WHITE);
    /// draw.texture("enemy", Vec2::ZERO, Some(Vec2::new(64.0, 64.0)), Color::RED);
    /// ```
    #[cfg(feature = "textures")]
    pub fn texture(
        &mut self,
        texture_name: &str,
        position: impl Into<[f32; 2]>,
        size: Option<impl Into<[f32; 2]>>,
        tint: impl Into<[f32; 4]>,
    ) {
        let Some(sprite_pipeline) = &mut self.sprite_pipeline else {
            return;
        };
        let Some(atlas) = &self.texture_atlas else {
            return;
        };
        let Some(region) = atlas.get(texture_name) else {
            return;
        };

        let pos = position.into();
        let size = size.map(|s| s.into()).unwrap_or(region.size_f32());
        let tint = tint.into();

        sprite_pipeline.draw(pos, size, region.uv_rect(), tint, 0.0, 0);
    }

    /// Draw a texture with full control over parameters.
    ///
    /// # Arguments
    /// * `texture_name` - Name of the texture region in the atlas
    /// * `position` - World position (center of the texture)
    /// * `size` - Size to render
    /// * `tint` - Color tint
    /// * `rotation` - Rotation in radians
    /// * `z_order` - Draw order (lower values drawn first)
    #[cfg(feature = "textures")]
    pub fn texture_ex(
        &mut self,
        texture_name: &str,
        position: impl Into<[f32; 2]>,
        size: impl Into<[f32; 2]>,
        tint: impl Into<[f32; 4]>,
        rotation: f32,
        z_order: i32,
    ) {
        let Some(sprite_pipeline) = &mut self.sprite_pipeline else {
            return;
        };
        let Some(atlas) = &self.texture_atlas else {
            return;
        };
        let Some(region) = atlas.get(texture_name) else {
            return;
        };

        sprite_pipeline.draw(
            position.into(),
            size.into(),
            region.uv_rect(),
            tint.into(),
            rotation,
            z_order,
        );
    }

    /// Check if a texture exists in the atlas.
    #[cfg(feature = "textures")]
    pub fn has_texture(&self, texture_name: &str) -> bool {
        self.texture_atlas
            .as_ref()
            .map(|atlas| atlas.get(texture_name).is_some())
            .unwrap_or(false)
    }

    // =========================================================================
    // ECS RENDERING
    // =========================================================================

    /// Render all visible entities from the ECS world.
    ///
    /// This method queries the world for all entities with [`Transform`], [`Sprite`],
    /// and [`Visible`] components, then renders them in z-order.
    ///
    /// # Z-Ordering
    ///
    /// Entities are sorted by `z_order` (lower values rendered first = behind, higher = on top).
    /// Z-ordering is respected across all sprite types - a circle with `z_order=5` will correctly
    /// render behind a texture sprite with `z_order=10`, regardless of their types.
    ///
    /// Sprites with the same z-order are batched together for efficiency.
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
        // Discriminant for sprite type - used for efficient batching
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(u8)]
        enum SpriteType {
            Circle = 0,
            Rect = 1,
            Line = 2,
            Texture = 3,
            TextureId = 4,
        }

        fn sprite_type(shape: &SpriteShape) -> SpriteType {
            match shape {
                SpriteShape::Circle { .. } => SpriteType::Circle,
                SpriteShape::Rect { .. } => SpriteType::Rect,
                SpriteShape::Line { .. } => SpriteType::Line,
                SpriteShape::Texture { .. } => SpriteType::Texture,
                #[cfg(feature = "textures")]
                SpriteShape::TextureId { .. } => SpriteType::TextureId,
            }
        }

        // Collect entity handles with their sort keys (no cloning of sprite data!)
        // Sort by (z_order, sprite_type) to maximize batching while preserving z-order
        let mut renderables: Vec<(i32, SpriteType, crate::ecs::Entity)> = world
            .query::<(&Transform, &Sprite, &Visible)>()
            .iter()
            .map(|(entity, (_, sprite, _))| {
                (sprite.z_order, sprite_type(&sprite.shape), entity)
            })
            .collect();

        // Sort by z_order first (primary), then by sprite type (secondary) for batch efficiency
        renderables.sort_unstable_by(|a, b| {
            a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1))
        });

        // Track if we've warned about texture sprites (to avoid spam)
        #[cfg(not(feature = "textures"))]
        let mut texture_sprite_warning_shown = false;

        // Build state for texture rendering (needed for flushing)
        #[cfg(feature = "textures")]
        let state = RenderState {
            gpu: self.gpu,
            view: self.view,
            camera: self.camera,
        };

        // Batches for each primitive type
        let mut circle_batch: Vec<CircleInstance> = Vec::with_capacity(renderables.len());
        let mut line_batch: Vec<LineInstance> = Vec::with_capacity(renderables.len() / 4);
        
        // Track current sort key to know when to flush
        let mut current_key: Option<(i32, SpriteType)> = None;

        for &(z_order, stype, entity) in &renderables {
            // Get components by reference (no clone!)
            let Some(transform) = world.get::<Transform>(entity) else { continue };
            let Some(sprite) = world.get::<Sprite>(entity) else { continue };

            let key = (z_order, stype);
            
            // When sort key changes, flush the relevant batch
            // Because we sorted by (z_order, sprite_type), we only flush when BOTH change
            // or when z_order changes (which necessitates flushing all)
            if let Some(prev_key) = current_key {
                if prev_key.0 != z_order {
                    // Z-order changed - flush ALL batches to preserve ordering
                    if !circle_batch.is_empty() {
                        self.circles(&circle_batch);
                        circle_batch.clear();
                    }
                    if !line_batch.is_empty() {
                        self.lines(&line_batch);
                        line_batch.clear();
                    }
                    #[cfg(feature = "textures")]
                    if let (Some(sprite_pipeline), Some(atlas)) = (&mut self.sprite_pipeline, &self.texture_atlas) {
                        if sprite_pipeline.staging_count() > 0 {
                            sprite_pipeline.flush(&state, atlas);
                        }
                    }
                }
                // Note: if only sprite_type changed but z_order is same, we don't need to flush
                // because same z_order means the rendering order within that z doesn't matter
            }
            current_key = Some(key);

            match &sprite.shape {
                SpriteShape::Circle { radius } => {
                    let scaled_radius = radius * transform.scale.x;
                    circle_batch.push(CircleInstance {
                        position: [transform.position.x, transform.position.y],
                        radius: scaled_radius,
                        color: sprite.color.into(),
                    });
                }
                SpriteShape::Rect { size } => {
                    let half_w = size.x * transform.scale.x * 0.5;
                    let half_h = size.y * transform.scale.y * 0.5;
                    let avg_radius = (half_w + half_h) * 0.5;
                    circle_batch.push(CircleInstance {
                        position: [transform.position.x, transform.position.y],
                        radius: avg_radius,
                        color: sprite.color.into(),
                    });
                }
                SpriteShape::Line { end_offset } => {
                    let start = transform.position;
                    let offset = *end_offset;
                    let end = start + offset * transform.scale.x;
                    line_batch.push(LineInstance {
                        start: [start.x, start.y],
                        end: [end.x, end.y],
                        color: sprite.color.into(),
                    });
                }   
                SpriteShape::Texture { region_name, size } => {
                    #[cfg(feature = "textures")]
                    if let (Some(sprite_pipeline), Some(atlas)) = (&mut self.sprite_pipeline, &self.texture_atlas) {
                        if let Some(region) = atlas.get(region_name.as_str()) {
                            sprite_pipeline.draw(
                                [transform.position.x, transform.position.y],
                                [size.x * transform.scale.x, size.y * transform.scale.y],
                                region.uv_rect(),
                                sprite.color.into(),
                                transform.rotation,
                                z_order,
                            );
                        }
                    }
                    #[cfg(not(feature = "textures"))]
                    if !texture_sprite_warning_shown {
                        log::warn!(
                            "render_world() skips texture sprites - \
                             enable the 'textures' feature and register an atlas via App::build().atlas()"
                        );
                        texture_sprite_warning_shown = true;
                    }
                }
                #[cfg(feature = "textures")]
                SpriteShape::TextureId { region, size } => {
                    if let (Some(sprite_pipeline), Some(atlas)) = (&mut self.sprite_pipeline, &self.texture_atlas) {
                        let atlas_region = atlas.get_by_id(*region);
                        sprite_pipeline.draw(
                            [transform.position.x, transform.position.y],
                            [size.x * transform.scale.x, size.y * transform.scale.y],
                            atlas_region.uv_rect(),
                            sprite.color.into(),
                            transform.rotation,
                            z_order,
                        );
                    }
                }
            }
        }

        // Flush any remaining batches
        if !circle_batch.is_empty() {
            self.circles(&circle_batch);
        }
        if !line_batch.is_empty() {
            self.lines(&line_batch);
        }
        #[cfg(feature = "textures")]
        if let (Some(sprite_pipeline), Some(atlas)) = (&mut self.sprite_pipeline, &self.texture_atlas) {
            if sprite_pipeline.staging_count() > 0 {
                sprite_pipeline.flush(&state, atlas);
            }
        }
    }

    // =========================================================================
    // TILEMAP RENDERING
    // =========================================================================

    /// Render a tilemap at the given position.
    ///
    /// This method efficiently renders all visible tiles from the tilemap,
    /// culling tiles outside the camera view for performance.
    ///
    /// # Arguments
    ///
    /// * `tilemap` - The tilemap to render
    /// * `position` - World position of the tilemap's origin (top-left corner)
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn render(&mut self, world: &World, draw: &mut DrawContext) {
    ///     // Render tilemap entities
    ///     for (_, (transform, tilemap, _)) in world.query::<(&Transform, &Tilemap, &Visible)>().iter() {
    ///         draw.tilemap(tilemap, transform.position);
    ///     }
    ///     
    ///     // Then render other entities on top
    ///     draw.render_world(world);
    /// }
    /// ```
    #[cfg(feature = "textures")]
    pub fn tilemap(&mut self, tilemap: &crate::ecs::tilemap::Tilemap, position: Vec2) {
        use crate::ecs::tilemap::TileFlip;

        let Some(sprite_pipeline) = &mut self.sprite_pipeline else {
            log::warn!("tilemap() requires a sprite pipeline with texture atlas");
            return;
        };
        let Some(atlas) = &self.texture_atlas else {
            log::warn!("tilemap() requires a texture atlas to be registered");
            return;
        };

        let state = RenderState {
            gpu: self.gpu,
            view: self.view,
            camera: self.camera,
        };

        // Calculate visible tile range based on camera view
        let camera_bounds = self.camera.visible_bounds();
        let tile_size = tilemap.tile_size();

        // Convert camera bounds to tile coordinates (with padding)
        let min_tx = ((camera_bounds.0.x - position.x) / tile_size.x).floor().max(0.0) as u32;
        let min_ty = ((camera_bounds.0.y - position.y) / tile_size.y).floor().max(0.0) as u32;
        let max_tx = ((camera_bounds.1.x - position.x) / tile_size.x).ceil().min(tilemap.width as f32) as u32;
        let max_ty = ((camera_bounds.1.y - position.y) / tile_size.y).ceil().min(tilemap.height as f32) as u32;

        // Render layers in z-order
        for layer in tilemap.layers_sorted() {
            if !layer.visible || layer.opacity <= 0.0 {
                continue;
            }

            let layer_tint_alpha = layer.opacity;

            // Render visible tiles
            for ty in min_ty..max_ty {
                for tx in min_tx..max_tx {
                    if let Some(tile) = layer.get(tx, ty) {
                        if tile.is_empty() {
                            continue;
                        }

                        // Get the current region (may be animated)
                        let region_name = if let Some(anim) = &tile.animation {
                            anim.current_region().unwrap_or(&tile.region_name)
                        } else {
                            &tile.region_name
                        };

                        if let Some(region) = atlas.get(region_name) {
                            // Calculate world position (tile center)
                            let world_x = position.x + (tx as f32 + 0.5) * tile_size.x;
                            let world_y = position.y + (ty as f32 + 0.5) * tile_size.y;

                            // Apply flip to UV coordinates
                            let uv = region.uv_rect();
                            let uv_rect = match tile.flip {
                                TileFlip::None => uv,
                                TileFlip::Horizontal => [uv[2], uv[1], uv[0], uv[3]], // swap u
                                TileFlip::Vertical => [uv[0], uv[3], uv[2], uv[1]], // swap v
                                TileFlip::Both => [uv[2], uv[3], uv[0], uv[1]], // swap both
                            };

                            sprite_pipeline.draw(
                                [world_x, world_y],
                                [tile_size.x, tile_size.y],
                                uv_rect,
                                [1.0, 1.0, 1.0, layer_tint_alpha],
                                0.0, // Tiles don't rotate individually
                                layer.z_order,
                            );
                        }
                    }
                }
            }
        }

        // Flush after rendering all layers
        if sprite_pipeline.staging_count() > 0 {
            sprite_pipeline.flush(&state, atlas);
        }
    }

    /// Render all visible tilemap entities from the ECS world.
    ///
    /// This automatically queries for entities with `Transform`, `Tilemap`, and `Visible`
    /// components and renders them. Tilemaps are rendered before other entities.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn render(&mut self, world: &World, draw: &mut DrawContext) {
    ///     // Render all tilemaps first (background)
    ///     draw.render_tilemaps(world);
    ///     
    ///     // Then render sprites/entities on top
    ///     draw.render_world(world);
    /// }
    /// ```
    #[cfg(feature = "textures")]
    pub fn render_tilemaps(&mut self, world: &World) {
        use crate::ecs::tilemap::Tilemap;
        
        // Collect tilemap data first to avoid borrow issues
        let mut tilemap_data: Vec<(i32, Vec2, Tilemap)> = Vec::new();
        
        {
            let mut query = world.query::<(&Transform, &Tilemap, &Visible)>();
            for (_, (transform, tilemap, _)) in query.iter() {
                // Use the tilemap's minimum layer z-order for sorting tilemaps against each other
                let min_z = tilemap.layers().map(|l| l.z_order).min().unwrap_or(0);
                tilemap_data.push((min_z, transform.position, tilemap.clone()));
            }
        }

        // Sort tilemaps by their minimum z-order
        tilemap_data.sort_by_key(|(z, _, _)| *z);

        // Render each tilemap
        for (_, position, tilemap) in &tilemap_data {
            self.tilemap(tilemap, *position);
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
            #[cfg(feature = "textures")]
            SpriteShape::TextureId { .. } => {
                // TextureId sprites require SpritePipeline with atlas
                // Use draw_sprite() or render_world() instead
                log::warn!("TextureId sprites not supported in render_entity(), use SpritePipeline");
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

    /// Flush all batched draw calls to the GPU using a single command encoder.
    /// The engine calls this automatically at the end of the render pass.
    pub fn flush(&mut self) {
        // Use unified RenderFrame for single GPU submission
        let mut frame = RenderFrame::begin(self.gpu, self.view, self.camera);

        // Flush all pipelines to the shared encoder
        self.circle_pipeline.flush_to_frame(&mut frame);
        self.line_pipeline.flush_to_frame(&mut frame);

        #[cfg(feature = "textures")]
        if let (Some(sprite_pipeline), Some(atlas)) =
            (&mut self.sprite_pipeline, &self.texture_atlas)
        {
            sprite_pipeline.flush_to_frame(&mut frame, atlas);
        }

        // Single GPU submission
        frame.submit();
    }
}

#[cfg(test)]
mod tests {
    // Note: Most DrawContext logic requires a GpuContext and Pipelines which require a GPU.
    // Integration tests should be used for full rendering verification.
}
