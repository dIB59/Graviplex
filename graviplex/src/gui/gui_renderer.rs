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
    font_tex: Texture,
    font_view: TextureView,
    sampler: Sampler,
    bind_group: BindGroup,
    bind_layout: BindGroupLayout,
    uniform_buf: Buffer,
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

        // Create initial 1x1 white RGBA texture
        let (font_tex, font_view) = Self::make_font_tex(device, queue, 1, 1, &[255, 255, 255, 255]);

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
            font_tex,
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        dst: &TextureView,
        primitives: &[ClippedPrimitive],
        screensize: [f32; 2],
        pixels_per_point: f32,
    ) {
        let (mut vertices, mut indices) = (vec![], vec![]);
        for p in primitives {
            if let Primitive::Mesh(ref m) = p.primitive {
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&m.vertices);
                indices.extend(m.indices.iter().map(|i| base + i));
            }
        }

        if vertices.is_empty() || indices.is_empty() {
            return;
        }

        let vtx_bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<egui::epaint::Vertex>(),
            )
        };
        self.queue.write_buffer(&self.vtx_buf, 0, vtx_bytes);
        self.queue
            .write_buffer(&self.idx_buf, 0, bytemuck::cast_slice(&indices));

        let logical_size = [
            screensize[0] / pixels_per_point,
            screensize[1] / pixels_per_point,
        ];
        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&logical_size));

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

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vtx_buf.slice(..));
        rpass.set_index_buffer(self.idx_buf.slice(..), IndexFormat::Uint32);

        let mut base = 0;
        for (i, prim) in primitives.iter().enumerate() {
            if let Primitive::Mesh(ref mesh) = prim.primitive {
                let egui::Rect { min, max } = prim.clip_rect;
                log::debug!(
                    "Primitive {}: clip_rect=({},{}) to ({},{}), indices={}",
                    i,
                    min.x,
                    min.y,
                    max.x,
                    max.y,
                    mesh.indices.len()
                );

                let scissor_x = (min.x * pixels_per_point).round().max(0.0) as u32;
                let scissor_y = (min.y * pixels_per_point).round().max(0.0) as u32;
                let scissor_w = ((max.x - min.x) * pixels_per_point).round().max(0.0) as u32;
                let scissor_h = ((max.y - min.y) * pixels_per_point).round().max(0.0) as u32;

                // Clamp to physical screen size
                let scissor_w =
                    scissor_w.min(screensize[0] as u32 - scissor_x.min(screensize[0] as u32));
                let scissor_h =
                    scissor_h.min(screensize[1] as u32 - scissor_y.min(screensize[1] as u32));

                log::debug!(
                    "  Scissor: x={}, y={}, w={}, h={}",
                    scissor_x,
                    scissor_y,
                    scissor_w,
                    scissor_h
                );

                rpass.set_scissor_rect(scissor_x, scissor_y, scissor_w, scissor_h);

                log::debug!(
                    "  Scissor: x={}, y={}, w={}, h={}",
                    scissor_x,
                    scissor_y,
                    scissor_w,
                    scissor_h
                );

                rpass.set_scissor_rect(scissor_x, scissor_y, scissor_w, scissor_h);
                let cnt = mesh.indices.len() as u32;
                rpass.draw_indexed(base..base + cnt, 0, 0..1);
                base += cnt;
            }
        }
    }

    // Process texture deltas from egui
    pub fn handle_textures(&mut self, textures_delta: egui::TexturesDelta) {
        for (id, delta) in textures_delta.set {
            log::debug!(
                "Texture update - ID: {:?}, size: {:?}, pos: {:?}",
                id,
                delta.image.size(),
                delta.pos
            );

            if id == egui::TextureId::default() {
                let [w, h] = delta.image.size();
                let data: Vec<u8> = match &delta.image {
                    egui::ImageData::Color(color_image) => {
                        log::debug!(
                            "Font texture: {}x{}, pixels: {}",
                            w,
                            h,
                            color_image.pixels.len()
                        );
                        color_image
                            .pixels
                            .iter()
                            .flat_map(|c| c.to_array())
                            .collect()
                    }
                };

                log::debug!("Data bytes: {}, expected: {}", data.len(), w * h * 4);

                // Check if this is a partial update (delta.pos is Some) or full texture
                if let Some([x, y]) = delta.pos {
                    // Partial update - write to existing texture at offset
                    log::debug!("Partial texture update at ({}, {})", x, y);
                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfoBase {
                            texture: &self.font_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: x as u32,
                                y: y as u32,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(w as u32 * 4),
                            rows_per_image: None,
                        },
                        wgpu::Extent3d {
                            width: w as u32,
                            height: h as u32,
                            depth_or_array_layers: 1,
                        },
                    );
                } else {
                    // Full texture replacement
                    log::debug!("Full texture update");
                    let (tex, view) =
                        Self::make_font_tex(&self.device, &self.queue, w as u32, h as u32, &data);
                    self.font_tex = tex;
                    self.font_view = view;
                    self.bind_group = Self::make_bg(
                        &self.device,
                        &self.bind_layout,
                        &self.font_view,
                        &self.sampler,
                        &self.uniform_buf,
                    );
                }
            }
        }
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
            format: TextureFormat::Rgba8Unorm, // Changed to RGBA
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
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4), // 4 bytes per pixel (RGBA)
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
            tex.create_view(&TextureViewDescriptor::default()),
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
