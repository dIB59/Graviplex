// ui_pipeline.rs
use egui::epaint::Primitive;
use egui::ClippedPrimitive;
use wgpu::*;

pub struct UiPipeline {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    vtx_buf: Buffer,
    idx_buf: Buffer,
    font_view: TextureView,
    sampler: Sampler,
    bind_group: BindGroup,
    bind_layout: BindGroupLayout,
    uniform_buf: Buffer, // Add this
}

impl UiPipeline {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("../shaders/shader.wgsl"));
        let bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ui"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ui"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ui"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<egui::epaint::Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            format: VertexFormat::Float32x2,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            offset: 8,
                            format: VertexFormat::Float32x2,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            offset: 16,
                            format: VertexFormat::Unorm8x4,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
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
            multiview: None,
            cache: None,
        });
        let (_font_tex, font_view) =
            Self::make_font_tex(device, queue, 1, 1, &[255, 255, 255, 255]);
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });
        let vtx_buf = device.create_buffer(&BufferDescriptor {
            label: Some("ui vtx"),
            size: 1 << 18,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let idx_buf = device.create_buffer(&BufferDescriptor {
            label: Some("ui idx"),
            size: 1 << 19,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buf = device.create_buffer(&BufferDescriptor {
            label: Some("ui uniform"),
            size: 8, // 2 f32s
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = Self::make_bg(device, &bind_layout, &font_view, &sampler, &uniform_buf);
        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            vtx_buf,
            idx_buf,
            font_view,
            sampler,
            bind_group,
            bind_layout,
            uniform_buf,
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        dst: &TextureView,
        vertices: &[egui::epaint::Vertex],
        indices: &[u32],
        primitives: &[ClippedPrimitive],
        screensize: [f32; 2],
    ) {
        let vtx_bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<egui::epaint::Vertex>(),
            )
        };
        self.queue.write_buffer(&self.vtx_buf, 0, vtx_bytes);
        self.queue
            .write_buffer(&self.idx_buf, 0, bytemuck::cast_slice(indices));
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ui"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: dst,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&screensize));
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vtx_buf.slice(..));
        rpass.set_index_buffer(self.idx_buf.slice(..), IndexFormat::Uint32);
        let mut base = 0;
        for prim in primitives {
            if let Primitive::Mesh(ref mesh) = prim.primitive {
                let egui::Rect { min, max } = prim.clip_rect;
                rpass.set_scissor_rect(
                    min.x as u32,
                    min.y as u32,
                    (max.x - min.x) as u32,
                    (max.y - min.y) as u32,
                );
                let cnt = mesh.indices.len() as u32;
                rpass.draw_indexed(base..base + cnt, 0, 0..1);
                base += cnt;
            }
        }
    }

    pub fn update_font(&mut self, data: &[u8], w: u32, h: u32) {
        let (_tex, view) = Self::make_font_tex(&self.device, &self.queue, w, h, data);
        self.font_view = view;
        self.bind_group = Self::make_bg(
            &self.device,
            &self.bind_layout,
            &self.font_view,
            &self.sampler,
            &self.uniform_buf,
        );
    }

    fn make_font_tex(
        device: &Device,
        queue: &wgpu::Queue,
        w: u32,
        h: u32,
        data: &[u8],
    ) -> (Texture, TextureView) {
        let tex = device.create_texture(&TextureDescriptor {
            label: Some("font"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm, // <── single channel
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data, // data is w*h bytes
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w), // <── one byte per pixel
                rows_per_image: None,
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        (
            tex.clone(),
            tex.clone().create_view(&TextureViewDescriptor::default()),
        )
    }

    fn make_bg(
        device: &Device,
        layout: &BindGroupLayout,
        view: &TextureView,
        sampler: &Sampler,
        uniform: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("ui"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        })
    }
}
