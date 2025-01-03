use std::sync::Arc;

use pollster::FutureExt;
use wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Default, Debug)]
pub struct App {
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    queue: Option<Queue>,
    device: Option<Device>,
    config: Option<SurfaceConfiguration>,
    render_pipeline: Option<RenderPipeline>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("winit example");

            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Unable to create window"),
            );
            self.window = Some(window);
        }

        let size = self.window.as_ref().expect("window not found").inner_size();

        let instance = Instance::new(InstanceDescriptor::default());

        let surface = instance
            .create_surface(self.window.clone().expect("window not found"))
            .expect("Unable to create surface");

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Unable to create adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    required_limits: Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .block_on()
            .expect("Unable to create device");

        let format: TextureFormat = surface.get_capabilities(&adapter).formats[0];

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            format,
            desired_maximum_frame_latency: Default::default(),
            alpha_mode: Default::default(),
            view_formats: Default::default(),
        };

        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: Default::default(),
            cache: Default::default(),
        });

        self.queue = Some(queue);
        self.device = Some(device);
        self.render_pipeline = Some(render_pipline);
        self.config = Some(surface_config);
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(surface) = &self.surface {
                    let mut config = self.config.as_ref().unwrap().clone();
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&self.device.as_ref().unwrap(), &config);
                    self.config = Some(config);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(surface), Some(device), Some(queue), Some(pipeline)) = (
                    &self.surface,
                    &self.device,
                    &self.queue,
                    &self.render_pipeline,
                ) {
                    self.render_frame(surface, device, queue, pipeline);
                }
            }
            _ => (),
        }
    }
}

impl App {
    fn render_frame(
        &self,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        pipeline: &RenderPipeline,
    ) {
        let frame = surface.get_current_texture().expect("Unable to get frame");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: Default::default(),
                occlusion_query_set: Default::default(),
            });

            rpass.set_pipeline(pipeline);
            rpass.draw(0..9, 0..3);
        }

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
