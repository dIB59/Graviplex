use std::{env::current_dir, fs};

pub struct PipelineBuilder {
    shader_filename: String,
    vertex_entry: String,
    fragment_entry: String,
    pixel_format: wgpu::TextureFormat,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
}

impl PipelineBuilder {
    pub fn new(shader_file_name: &str) -> Self {
        PipelineBuilder {
            shader_filename: shader_file_name.to_string(),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            pixel_format: wgpu::TextureFormat::Rgba8Unorm,
            vertex_buffer_layouts: Vec::new(),
        }
    }

    pub fn build_pipeline(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let mut filepath = current_dir().unwrap();
        filepath.push("src/");
        filepath.push(self.shader_filename.as_str());
        let filepath = filepath.into_os_string().into_string().unwrap();
        let source_code = fs::read_to_string(filepath).expect("Can't read source code!");
        let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader Module: {}", self.shader_filename)),
            source: wgpu::ShaderSource::Wgsl(source_code.into()),
        };
        let shader_module = device.create_shader_module(shader_module_descriptor);

        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Render Pipeline Layout: {}", self.shader_filename)),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);

        let render_targets = [Some(wgpu::ColorTargetState {
            format: self.pixel_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline: {}", self.shader_filename)),

            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some(&self.vertex_entry),
                buffers: &self.vertex_buffer_layouts,
                compilation_options: Default::default(),
            },

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some(&self.fragment_entry),
                targets: &render_targets,
                compilation_options: Default::default(),
            }),

            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: Default::default(),
        };

        device.create_render_pipeline(&render_pipeline_descriptor)
    }
}
