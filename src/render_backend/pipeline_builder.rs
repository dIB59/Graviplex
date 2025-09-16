use std::{borrow::Cow, env::current_dir, fs};

use log::{debug, warn};
use wgpu::{FragmentState, ShaderModuleDescriptor};

pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    shader_filename: String,
    vertex_entry: String,
    fragment_entry: String,
    pixel_format: wgpu::TextureFormat,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    set_shader_module_called: bool,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(shader_file_name: &str, device: &'a wgpu::Device) -> Self {
        PipelineBuilder {
            device: device,
            shader_filename: shader_file_name.to_string(),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            pixel_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            vertex_buffer_layouts: Vec::new(),
            bind_group_layouts: Vec::new(),
            set_shader_module_called: false,
        }
    }

    pub fn reset(&mut self) {
        self.vertex_buffer_layouts.clear();
    }

    pub fn add_vertex_buffer_layout(
        &mut self,
        layout: wgpu::VertexBufferLayout<'static>,
    ) -> &mut PipelineBuilder<'a> {
        self.vertex_buffer_layouts.push(layout);
        self
    }

    pub fn add_bind_group_layout(
        &mut self,
        layout: &'a wgpu::BindGroupLayout,
    ) -> &mut PipelineBuilder<'a> {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn set_shader_module(
        &mut self,
        shader_filename: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) {
        self.shader_filename = shader_filename.to_string();
        self.vertex_entry = vertex_entry.to_string();
        self.fragment_entry = fragment_entry.to_string();
        self.set_shader_module_called = true;
    }

    pub fn set_pixel_format(
        &mut self,
        pixel_format: wgpu::TextureFormat,
    ) -> &mut PipelineBuilder<'a> {
        self.pixel_format = pixel_format;
        self
    }

    pub fn build_pipeline(&self) -> wgpu::RenderPipeline {
        if !self.set_shader_module_called {
            warn!(
                "PipelineBuilder: `set_shader_module()` was not called before `build_pipeline()`. 
Default shader values may be used unintentionally."
            );
        }
        let mut filepath = current_dir().unwrap();
        filepath.push("src");
        filepath.push(self.shader_filename.as_str());
        let filepath = &filepath.into_os_string().into_string().unwrap();
        debug!("{}", filepath);

        // Read shader code at runtime
        let shader_source = fs::read_to_string(&filepath).expect("Failed to read shader file");

        // Create shader module using Cow::Owned
        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader numero 1"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
            });

        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Render Pipeline Layout: {}", self.shader_filename)),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &[],
        };

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&pipeline_layout_descriptor);

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

            primitive: wgpu::PrimitiveState::default(),
            fragment: Some(FragmentState {
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

        self.device
            .create_render_pipeline(&render_pipeline_descriptor)
    }
}
