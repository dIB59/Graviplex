use crate::renderer::camera::CameraGpuData;
use crate::renderer::{Camera2D, CircleInstance, RenderState, Vertex};
use wgpu::*;

/// A built-in pipeline for rendering anti-aliased circles.
///
/// This pipeline uses instanced rendering to draw many circles efficiently.
/// It supports both standard `CircleInstance` data and custom buffers.
pub struct CirclePipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) instance_buffer: Buffer,
    pub(crate) camera_gpu_data: CameraGpuData,
    pub(crate) staging_instances: Vec<CircleInstance>,
}

impl CirclePipeline {
    /// Create a new CirclePipeline with default settings.
    pub fn new(device: &Device, format: TextureFormat, camera: &Camera2D) -> Self {
        Self::with_instance_layout(device, format, camera, CircleInstance::desc())
    }

    /// Create a new CirclePipeline with a custom instance layout.
    /// Useful for simulations with specialized particle data.
    pub fn with_instance_layout(
        device: &Device,
        format: TextureFormat,
        camera: &Camera2D,
        instance_layout: VertexBufferLayout<'static>,
    ) -> Self {
        let camera_gpu_data = CameraGpuData::new(device, camera);
        let shader = device.create_shader_module(include_wgsl!("../shaders/circle_shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Circle Pipeline Layout"),
            bind_group_layouts: &[&camera_gpu_data.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Circle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), instance_layout],
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

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Circle Vertex Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: 1 << 16, // Sufficient for basic vertex data
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Circle Instance Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: 1 << 27, // 128MB for instances (~4M instances of CircleInstance/PhysicsInstance)
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

    /// Add a circle instance to the current batch.
    pub fn draw_circle_instance(&mut self, instance: CircleInstance) {
        self.staging_instances.push(instance);
    }

    /// Add a circle to the current batch.
    pub fn draw_circle(&mut self, position: [f32; 2], radius: f32, color: [f32; 4]) {
        self.staging_instances.push(CircleInstance {
            position,
            radius,
            color,
        });
    }

    /// Add multiple circles to the current batch.
    pub fn draw_circles(&mut self, instances: &[CircleInstance]) {
        self.staging_instances.extend_from_slice(instances);
    }

    /// Clear the current batch without drawing.
    pub fn clear_batch(&mut self) {
        self.staging_instances.clear();
    }

    /// Flush the current batch to the GPU and draw.
    pub fn flush(&mut self, state: &RenderState) {
        if self.staging_instances.is_empty() {
            return;
        }

        self.camera_gpu_data.update(&state.gpu.raw_queue(), state.camera);

        // Upload defaults if not done (though we could just do it once in new)
        let vertices = [
            Vertex { pos: [-1.0, -1.0] },
            Vertex { pos: [1.0, -1.0] },
            Vertex { pos: [-1.0, 1.0] },
            Vertex { pos: [-1.0, 1.0] },
            Vertex { pos: [1.0, -1.0] },
            Vertex { pos: [1.0, 1.0] },
        ];
        state.gpu.raw_queue().write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        state.gpu.raw_queue().write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.staging_instances),
        );

        let mut encoder = state.gpu.raw_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Circle Batch Encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Circle Render Pass"),
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
            rpass.draw(0..6, 0..self.staging_instances.len() as u32);
        }

        state.gpu.raw_queue().submit(std::iter::once(encoder.finish()));
        self.staging_instances.clear();
    }

    /// Render circles using an external instance buffer (e.g. from GPU compute).
    /// This bypasses the internal batching.
    pub fn render_with_external_buffer(
        &self,
        state: &RenderState,
        instance_count: u32,
        external_instance_buffer: &Buffer,
    ) {
        if instance_count == 0 {
            return;
        }

        self.camera_gpu_data.update(&state.gpu.raw_queue(), state.camera);

        let vertices = [
            Vertex { pos: [-1.0, -1.0] },
            Vertex { pos: [1.0, -1.0] },
            Vertex { pos: [-1.0, 1.0] },
            Vertex { pos: [-1.0, 1.0] },
            Vertex { pos: [1.0, -1.0] },
            Vertex { pos: [1.0, 1.0] },
        ];
        state.gpu.raw_queue().write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let mut encoder = state.gpu.raw_device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Circle Render (External) Encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Circle Render Pass"),
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
            rpass.set_vertex_buffer(1, external_instance_buffer.slice(..));
            rpass.set_bind_group(0, &self.camera_gpu_data.bind_group, &[]);
            rpass.draw(0..6, 0..instance_count);
        }

        state.gpu.raw_queue().submit(std::iter::once(encoder.finish()));
    }

    pub fn camera_gpu_data(&self) -> &CameraGpuData {
        &self.camera_gpu_data
    }

    pub fn staging_count(&self) -> usize {
        self.staging_instances.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_instance_data() {
        let instance = CircleInstance {
            position: [0.0, 0.0],
            radius: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        assert_eq!(instance.position, [0.0, 0.0]);
        assert_eq!(instance.radius, 1.0);
    }
}
