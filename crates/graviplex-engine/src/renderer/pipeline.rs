use super::{Camera2D, Instance, Vertex};
use crate::renderer::camera::CameraGpuData;
use wgpu::*;

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    camera_gpu_data: CameraGpuData,
}

impl RenderPipeline {
    /// Creates a new, Shader is file path location.
    /// eg include_str!("../shaders/circle_shader.wgsl")
    pub fn new(
        name: &str,
        shader_module_descriptor: ShaderModuleDescriptor,
        device: &Device,
        format: TextureFormat,
        camera: &Camera2D,
    ) -> Self {
        let camera_gpu_data = CameraGpuData::new(device, camera);
        let shader = device.create_shader_module(shader_module_descriptor);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(("Render Pipeline Layout ".to_owned() + name).as_str()),
            bind_group_layouts: &[&camera_gpu_data.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(("Render Pipeline ".to_owned() + name).as_str()),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
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
            label: Some(("Vertex Buffer ".to_owned() + name).as_str()),
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: 1 << 28,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some(("Instance Buffer ".to_owned() + name).as_str()),
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: 1 << 28,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
            camera_gpu_data,
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        vertices: &[Vertex],
        instance_count: u32,
        external_instance_buffer: Option<&Buffer>,
    ) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    depth_slice: None,
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: Default::default(),
                occlusion_query_set: Default::default(),
            });

            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));

            let inst_buffer = if let Some(buf) = external_instance_buffer {
                buf
            } else {
                // Warning: We need the instances slice if we want to write it
                // But for now we just handle GPU path cleanly
                &self.instance_buffer
            };

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, inst_buffer.slice(..));
            rpass.set_bind_group(0, &self.camera_gpu_data.bind_group, &[]);
            rpass.draw(0..vertices.len() as u32, 0..instance_count);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn camera_gpu_data(&self) -> &CameraGpuData {
        &self.camera_gpu_data
    }
}
