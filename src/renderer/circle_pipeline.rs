use crate::renderer::camera::CameraGpuData;
use crate::renderer::{Camera2D, CircleInstance, Vertex};
use wgpu::*;

/// A built-in pipeline for rendering anti-aliased circles.
///
/// This pipeline uses instanced rendering to draw many circles efficiently.
/// It supports both standard `CircleInstance` data and custom buffers.
pub struct CirclePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    camera_gpu_data: CameraGpuData,
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
            size: 1 << 27, // 128MB for instances
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
            camera_gpu_data,
        }
    }

    /// Render circles using a slice of CircleInstance data.
    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        camera: &Camera2D,
        vertices: &[Vertex],
        instances: &[CircleInstance],
    ) {
        if instances.is_empty() {
            return;
        }

        self.camera_gpu_data.update(queue, camera);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Circle Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Circle Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    depth_slice: None,
                    view,
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
            rpass.draw(0..vertices.len() as u32, 0..instances.len() as u32);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Render circles using an external instance buffer (e.g. from GPU compute).
    pub fn render_with_external_buffer(
        &self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        camera: &Camera2D,
        vertices: &[Vertex],
        instance_count: u32,
        external_instance_buffer: &Buffer,
    ) {
        if instance_count == 0 {
            return;
        }

        self.camera_gpu_data.update(queue, camera);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Circle Render (External) Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Circle Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    depth_slice: None,
                    view,
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
            rpass.draw(0..vertices.len() as u32, 0..instance_count);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn camera_gpu_data(&self) -> &CameraGpuData {
        &self.camera_gpu_data
    }
}
