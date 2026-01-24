//! Sprite rendering pipeline with texture atlas support.
//!
//! This pipeline renders textured quads using instanced rendering for efficiency.
//! Sprites are batched and sorted by z-order before rendering.
//!
//! # Buffer Capacity
//!
//! The instance buffer has a fixed size of 128MB, which allows for approximately
//! 2.5 million sprites per batch. Larger batches are automatically split into
//! multiple draw calls.

use crate::ecs::{Sprite, SpriteShape, Transform, Visible, World};
use crate::renderer::camera::CameraGpuData;
use crate::renderer::vertex_data::{SpriteInstance, SpriteInstanceGpu, Vertex};
use crate::renderer::{Camera2D, RenderState};
use wgpu::*;

#[cfg(feature = "textures")]
use super::texture_atlas::TextureAtlas;

/// Size of the instance buffer in bytes (128MB).
const INSTANCE_BUFFER_SIZE: u64 = 1 << 27;

/// Maximum number of sprites per draw call.
/// Calculated as buffer size / size of SpriteInstanceGpu (52 bytes).
const MAX_SPRITES_PER_BATCH: usize = (INSTANCE_BUFFER_SIZE as usize) / std::mem::size_of::<SpriteInstanceGpu>();

/// Pipeline for rendering textured sprites with atlas support.
///
/// Sprites are batched and sorted by z-order before rendering.
/// Uses instanced rendering for efficiency.
#[cfg(feature = "textures")]
pub struct SpritePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    camera_gpu_data: CameraGpuData,
    staging_instances: Vec<SpriteInstance>,
}

#[cfg(feature = "textures")]
impl SpritePipeline {
    /// Create a new sprite pipeline.
    ///
    /// The `atlas` is used to get the bind group layout for texture binding.
    pub fn new(
        device: &Device,
        format: TextureFormat,
        camera: &Camera2D,
        atlas: &TextureAtlas,
    ) -> Self {
        let camera_gpu_data = CameraGpuData::new(device, camera);
        let shader = device.create_shader_module(include_wgsl!("../shaders/sprite_shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&camera_gpu_data.bind_group_layout, &atlas.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), SpriteInstance::desc()],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        });

        // Quad vertices: centered at origin, size 1x1
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: 1 << 16,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Sprite Instance Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: INSTANCE_BUFFER_SIZE,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
            camera_gpu_data,
            staging_instances: Vec::with_capacity(10000),
        }
    }

    /// Add a sprite instance to the current batch.
    pub fn draw_sprite(&mut self, instance: SpriteInstance) {
        self.staging_instances.push(instance);
    }

    /// Add a sprite with explicit parameters.
    pub fn draw(
        &mut self,
        position: [f32; 2],
        size: [f32; 2],
        uv_rect: [f32; 4],
        tint: [f32; 4],
        rotation: f32,
        z_order: i32,
    ) {
        self.staging_instances.push(SpriteInstance {
            position,
            size,
            uv_rect,
            tint,
            rotation,
            z_order,
        });
    }

    /// Clear the current batch without drawing.
    pub fn clear_batch(&mut self) {
        self.staging_instances.clear();
    }

    /// Get the number of sprites in the current batch.
    pub fn staging_count(&self) -> usize {
        self.staging_instances.len()
    }

    /// Flush the current batch to the GPU and draw.
    ///
    /// Sprites are sorted by z-order (lower values drawn first).
    /// Large batches exceeding the buffer capacity (~2.5M sprites) are automatically
    /// split into multiple draw calls.
    pub fn flush(&mut self, state: &RenderState, atlas: &TextureAtlas) {
        if self.staging_instances.is_empty() {
            return;
        }

        // Sort by z-order (stable sort preserves order for same z)
        self.staging_instances
            .sort_by(|a, b| a.z_order.cmp(&b.z_order));

        // Convert to GPU format (strip z_order)
        let gpu_instances: Vec<SpriteInstanceGpu> =
            self.staging_instances.iter().map(|i| i.to_gpu()).collect();

        self.camera_gpu_data.update(state.gpu.raw_queue(), state.camera);

        // Quad vertices: centered, size 1x1 (scaled by instance size)
        let vertices = [
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [0.5, -0.5] },
            Vertex { pos: [-0.5, 0.5] },
            Vertex { pos: [-0.5, 0.5] },
            Vertex { pos: [0.5, -0.5] },
            Vertex { pos: [0.5, 0.5] },
        ];

        state.gpu.raw_queue().write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        // Split into chunks if we exceed buffer capacity
        for chunk in gpu_instances.chunks(MAX_SPRITES_PER_BATCH) {
            state.gpu.raw_queue().write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(chunk),
            );

            let mut encoder = state.gpu.raw_device().create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Sprite Batch Encoder"),
                });

            {
                let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Sprite Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        depth_slice: None,
                        view: state.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: Default::default(),
                    occlusion_query_set: Default::default(),
                });

                rpass.set_pipeline(&self.pipeline);
                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                rpass.set_bind_group(0, &self.camera_gpu_data.bind_group, &[]);
                rpass.set_bind_group(1, &atlas.bind_group, &[]);
                rpass.draw(0..6, 0..chunk.len() as u32);
            }

            state.gpu.raw_queue().submit(std::iter::once(encoder.finish()));
        }

        self.staging_instances.clear();
    }

    /// Render all texture sprites from an ECS World.
    ///
    /// This is the recommended way to render texture sprites from ECS entities.
    /// It automatically queries for entities with `Transform`, `Sprite`, and `Visible`
    /// components, filters for texture sprites, and renders them sorted by z-order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn render(&mut self, world: &World, draw: &mut DrawContext) {
    ///     let atlas = self.atlas.as_ref().unwrap();
    ///     let pipeline = self.sprite_pipeline.as_mut().unwrap();
    ///     
    ///     // Render all texture sprites from the world
    ///     pipeline.render_world(world, &draw.state(), atlas);
    ///     
    ///     // You can also render primitive shapes
    ///     draw.circle((Vec2::ZERO, 50.0, Color::RED));
    /// }
    /// ```
    pub fn render_world(&mut self, world: &World, state: &RenderState, atlas: &TextureAtlas) {
        // Query all visible entities with sprites
        for (_, (transform, sprite, _)) in world.query::<(&Transform, &Sprite, &Visible)>().iter() {
            if let SpriteShape::Texture { ref region_name, size } = sprite.shape {
                if let Some(region) = atlas.get(region_name) {
                    self.staging_instances.push(SpriteInstance {
                        position: [transform.position.x, transform.position.y],
                        size: [size.x * transform.scale.x, size.y * transform.scale.y],
                        uv_rect: region.uv_rect(),
                        tint: sprite.color.into(),
                        rotation: transform.rotation,
                        z_order: sprite.z_order,
                    });
                }
            }
        }

        // Flush renders and clears the batch
        self.flush(state, atlas);
    }

    /// Access the camera GPU data.
    pub fn camera_gpu_data(&self) -> &CameraGpuData {
        &self.camera_gpu_data
    }

    /// Returns the maximum number of sprites that can be rendered in a single draw call.
    ///
    /// Batches exceeding this limit are automatically split, so this is primarily
    /// informational. The current limit is approximately 2.5 million sprites.
    pub const fn max_sprites_per_batch() -> usize {
        MAX_SPRITES_PER_BATCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // UNIT TESTS - SpriteInstance
    // =========================================================================

    #[test]
    fn test_sprite_instance_creation() {
        let instance = SpriteInstance::new(
            [100.0, 200.0],
            [32.0, 32.0],
            [0.0, 0.0, 0.5, 0.5],
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            0,
        );

        assert_eq!(instance.position, [100.0, 200.0]);
        assert_eq!(instance.size, [32.0, 32.0]);
        assert_eq!(instance.uv_rect, [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(instance.tint, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(instance.rotation, 0.0);
        assert_eq!(instance.z_order, 0);
    }

    #[test]
    fn test_sprite_instance_to_gpu() {
        let instance = SpriteInstance::new(
            [100.0, 200.0],
            [32.0, 32.0],
            [0.0, 0.0, 0.5, 0.5],
            [1.0, 1.0, 1.0, 1.0],
            0.5,
            5,
        );

        let gpu = instance.to_gpu();
        assert_eq!(gpu.position, [100.0, 200.0]);
        assert_eq!(gpu.size, [32.0, 32.0]);
        assert_eq!(gpu.uv_rect, [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(gpu.tint, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(gpu.rotation, 0.5);
        // z_order is NOT in GPU struct (used for CPU sorting only)
    }

    #[test]
    fn test_sprite_instance_with_rotation() {
        use std::f32::consts::PI;

        let instance = SpriteInstance::new(
            [0.0, 0.0],
            [64.0, 64.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 0.0, 0.0, 1.0], // Red tint
            PI / 4.0,             // 45 degree rotation
            0,
        );

        assert!((instance.rotation - PI / 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_sprite_instance_with_negative_z_order() {
        let background = SpriteInstance::new(
            [0.0, 0.0],
            [800.0, 600.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            -100, // Behind everything
        );

        let foreground = SpriteInstance::new(
            [0.0, 0.0],
            [32.0, 32.0],
            [0.0, 0.0, 0.5, 0.5],
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            100, // In front
        );

        assert!(background.z_order < foreground.z_order);
    }

    #[test]
    fn test_sprite_instance_gpu_preserves_all_visual_data() {
        let instance = SpriteInstance::new(
            [50.0, 75.0],
            [128.0, 256.0],
            [0.25, 0.25, 0.75, 0.75],
            [0.5, 0.5, 0.5, 0.8],
            1.57,
            999,
        );

        let gpu = instance.to_gpu();

        // All visual data should be preserved
        assert_eq!(instance.position, gpu.position);
        assert_eq!(instance.size, gpu.size);
        assert_eq!(instance.uv_rect, gpu.uv_rect);
        assert_eq!(instance.tint, gpu.tint);
        assert_eq!(instance.rotation, gpu.rotation);
    }

    // =========================================================================
    // UNIT TESTS - Z-Order Sorting
    // =========================================================================

    #[test]
    fn test_z_order_sorting_basic() {
        let mut instances = vec![
            SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 5),
            SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 1),
            SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 3),
        ];

        instances.sort_by(|a, b| a.z_order.cmp(&b.z_order));

        assert_eq!(instances[0].z_order, 1);
        assert_eq!(instances[1].z_order, 3);
        assert_eq!(instances[2].z_order, 5);
    }

    #[test]
    fn test_z_order_sorting_with_negatives() {
        let mut instances = vec![
            SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 0),
            SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, -10),
            SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 10),
        ];

        instances.sort_by(|a, b| a.z_order.cmp(&b.z_order));

        assert_eq!(instances[0].z_order, -10);
        assert_eq!(instances[1].z_order, 0);
        assert_eq!(instances[2].z_order, 10);
    }

    #[test]
    fn test_z_order_sorting_stability() {
        // Same z_order should maintain insertion order (stable sort)
        let mut instances = vec![
            SpriteInstance::new([1.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 0),
            SpriteInstance::new([2.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 0),
            SpriteInstance::new([3.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0; 4], 0.0, 0),
        ];

        instances.sort_by(|a, b| a.z_order.cmp(&b.z_order));

        // Stable sort preserves original order for equal keys
        assert_eq!(instances[0].position[0], 1.0);
        assert_eq!(instances[1].position[0], 2.0);
        assert_eq!(instances[2].position[0], 3.0);
    }

    // =========================================================================
    // REGRESSION TESTS
    // =========================================================================

    #[test]
    fn test_sprite_instance_struct_size() {
        // Ensure struct size doesn't change unexpectedly (would break GPU buffer layouts)
        let expected_size = std::mem::size_of::<[f32; 2]>() * 2  // position + size
            + std::mem::size_of::<[f32; 4]>() * 2                // uv_rect + tint
            + std::mem::size_of::<f32>()                         // rotation
            + std::mem::size_of::<i32>();                        // z_order

        assert_eq!(std::mem::size_of::<SpriteInstance>(), expected_size);
    }

    #[test]
    fn test_sprite_instance_gpu_struct_size() {
        // GPU struct should NOT include z_order
        let expected_size = std::mem::size_of::<[f32; 2]>() * 2  // position + size
            + std::mem::size_of::<[f32; 4]>() * 2                // uv_rect + tint
            + std::mem::size_of::<f32>();                        // rotation

        assert_eq!(std::mem::size_of::<SpriteInstanceGpu>(), expected_size);
    }

    #[test]
    fn test_sprite_instance_debug_impl() {
        let instance = SpriteInstance::new(
            [10.0, 20.0],
            [30.0, 40.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            0,
        );

        let debug_str = format!("{:?}", instance);
        assert!(debug_str.contains("SpriteInstance"));
        assert!(debug_str.contains("10.0"));
        assert!(debug_str.contains("20.0"));
    }

    #[test]
    fn test_sprite_instance_copy_semantics() {
        let original = SpriteInstance::new(
            [1.0, 2.0],
            [3.0, 4.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            0.5,
            10,
        );

        let copied = original; // Copy
        let cloned = original.clone(); // Clone

        assert_eq!(original.position, copied.position);
        assert_eq!(original.z_order, cloned.z_order);
    }

    // =========================================================================
    // UNIT TESTS - Buffer Capacity
    // =========================================================================

    #[test]
    fn test_max_sprites_per_batch_calculation() {
        // SpriteInstanceGpu is 52 bytes, buffer is 128MB
        // 128MB / 52 bytes = 2,576,980 sprites
        let expected = (128 * 1024 * 1024) / 52;
        assert_eq!(MAX_SPRITES_PER_BATCH, expected);
        assert!(MAX_SPRITES_PER_BATCH > 2_500_000, "Should support at least 2.5M sprites");
    }

    #[test]
    fn test_max_sprites_per_batch_public_accessor() {
        // Verify the public accessor returns the same value
        #[cfg(feature = "textures")]
        assert_eq!(SpritePipeline::max_sprites_per_batch(), MAX_SPRITES_PER_BATCH);
    }
}
