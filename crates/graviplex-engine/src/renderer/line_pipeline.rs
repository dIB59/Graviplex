use crate::renderer::camera::CameraGpuData;
use crate::renderer::Camera2D;
use bytemuck::NoUninit;
use wgpu::*;

/// Vertex for line rendering - just a single float (0.0 or 1.0) to interpolate between start/end
#[repr(C)]
#[derive(Clone, Copy, NoUninit, Debug)]
pub struct LineVertex {
    pub pos: [f32; 2],
}

impl LineVertex {
    const ATTRIBS: [VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Instance data for a line segment
#[repr(C)]
#[derive(Copy, Clone, NoUninit, Debug)]
pub struct LineInstance {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
}

impl LineInstance {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x4];

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<LineInstance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct LinePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    camera_gpu_data: CameraGpuData,
}

impl LinePipeline {
    /// Line vertices: two points for a line segment
    const LINE_VERTICES: [LineVertex; 2] = [
        LineVertex { pos: [0.0, 0.0] },
        LineVertex { pos: [1.0, 0.0] },
    ];

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
            label: Some(&format!("Line Pipeline Layout {}", name)),
            bind_group_layouts: &[&camera_gpu_data.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("Line Pipeline {}", name)),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[LineVertex::desc(), LineInstance::desc()],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
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
            label: Some(&format!("Line Vertex Buffer {}", name)),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: std::mem::size_of::<[LineVertex; 2]>() as u64,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&format!("Line Instance Buffer {}", name)),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: 1 << 27, // 128MB for line instances
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
            camera_gpu_data,
        }
    }

    /// Render lines after the main scene (no clear, just overlay)
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        view: &TextureView,
        instances: &[LineInstance],
    ) {
        if instances.is_empty() {
            return;
        }

        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&Self::LINE_VERTICES),
        );
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Line Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    depth_slice: None,
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load, // Don't clear, overlay on existing content
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
            rpass.draw(0..2, 0..instances.len() as u32);
        }
    }

    pub fn camera_gpu_data(&self) -> &CameraGpuData {
        &self.camera_gpu_data
    }
}
