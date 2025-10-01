use std::collections::HashSet;
use std::sync::Arc;

use crate::camera::{Camera2D, CameraController, CameraPlugin}; // Import the camera plugin
use crate::render::{self, Vertex};
use crate::render_backend::pipeline_builder::PipelineBuilder;
use crate::simulation::Simulation;
use pollster::FutureExt;
use wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct App {
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    config: Option<SurfaceConfiguration>,
    queue: Queue,
    device: Device,
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera_plugin: CameraPlugin, // Replace view with camera plugin
    last_frame_time: std::time::Instant,
    pressed_keys: HashSet<KeyCode>,
    simulation: Simulation,
}

impl Default for App {
    fn default() -> Self {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .expect("Unable to create adapter");

        #[cfg(target_os = "macos")]
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_limits: Limits::default(),
                required_features: Features::SHADER_F16,
                ..Default::default()
            })
            .block_on()
            .expect("Unable to create device");

        #[cfg(target_os = "windows")]
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_limits: Limits::default(),
                required_features: Features::SHADER_F16 | Features::CONSERVATIVE_RASTERIZATION,
                ..Default::default()
            })
            .block_on()
            .expect("Unable to create device");

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            size: 1 << 28,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            size: 1 << 28,
            mapped_at_creation: false,
        });

        // Create camera plugin with custom settings
        let camera = Camera2D::new([0.0, 0.0], 10.0, [1200.0, 1200.0])
            .with_zoom_speed(1.1)
            .with_screen_size([1200.0, 200.0]);

        let controller = CameraController::new()
            .with_move_speed(250.0) // Adjust movement speed for new scale
            .with_zoom_range(0.1, 10.0);

        let camera_plugin = CameraPlugin::new(&device)
            .with_camera(camera)
            .with_controller(controller);

        // Initialize GPU resources

        let render_pipeline = PipelineBuilder::new("circle_shader.wgsl", &device)
            .add_bind_group_layout(camera_plugin.bind_group_layout())
            .add_vertex_buffer_layout(render::Vertex::desc())
            .add_vertex_buffer_layout(render::Instance::desc())
            .build_pipeline();

        let mut simulation = Simulation::default();

        simulation.generate_bodies(1000);

        Self {
            window: None,
            surface: None,
            config: None,
            queue,
            device,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            camera_plugin,
            last_frame_time: std::time::Instant::now(),
            pressed_keys: HashSet::new(),
            simulation,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("Simulation");

            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Unable to create window"),
            );
            self.window = Some(window);
        }

        let size = self.window.as_ref().expect("window not found").inner_size();

        let instance = Instance::new(&InstanceDescriptor::default());

        let surface = instance
            .create_surface(self.window.clone().expect("window not found"))
            .expect("Unable to create surface");

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Unable to create adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_limits: Limits::default(),
                ..Default::default()
            })
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            size: 1 << 28,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            size: 1 << 28,
            mapped_at_creation: false,
        });

        // Reinitialize camera plugin for the new device
        let camera = Camera2D::new([0.0, 0.0], 10.0, [1200.0, 1200.0])
            .with_zoom_speed(1.1)
            .with_screen_size([size.width as f32, size.height as f32]);

        let controller = CameraController::new()
            .with_move_speed(250.0) // Adjust movement speed for new scale
            .with_zoom_range(0.1, 10.0);

        let camera_plugin = CameraPlugin::new(&device)
            .with_camera(camera)
            .with_controller(controller);

        let render_pipeline = PipelineBuilder::new("circle_shader.wgsl", &device)
            .add_bind_group_layout(camera_plugin.bind_group_layout())
            .add_vertex_buffer_layout(render::Vertex::desc())
            .add_vertex_buffer_layout(render::Instance::desc())
            .set_pixel_format(format)
            .build_pipeline();

        self.config = Some(surface_config);
        self.surface = Some(surface);
        self.queue = queue;
        self.device = device;
        self.render_pipeline = render_pipeline;
        self.camera_plugin = camera_plugin;
        self.vertex_buffer = vertex_buffer;
        self.instance_buffer = instance_buffer;
        self.last_frame_time = std::time::Instant::now();
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
                    surface.configure(&self.device, &config);
                    self.config = Some(config);

                    // Update camera screen size
                    self.camera_plugin
                        .handle_resize([size.width as f32, size.height as f32], &self.queue);
                }
            }
            WindowEvent::RedrawRequested => {
                if self.surface.is_some() {
                    self.render_frame();
                    self.simulation.update(0.01);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_plugin.handle_scroll(delta, &self.queue);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

pub fn random_triangle(center: [f32; 2], size: f32) -> [Vertex; 3] {
    // Base equilateral triangle around origin
    let base_vertices = [
        [0.0, size],
        [size * 0.866, -size * 0.5], // 60° rotated
        [-size * 0.866, -size * 0.5],
    ];

    let mut vertices = [Vertex { pos: [0.0, 0.0] }; 3];

    for (i, base) in base_vertices.iter().enumerate() {
        let x = base[0] + center[0];
        let y = base[1] + center[1];

        vertices[i] = Vertex { pos: [x, y] };
    }

    vertices
}

impl App {
    fn render_frame(&mut self) {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Update camera through plugin
        self.camera_plugin
            .update(delta_time, &self.pressed_keys, &self.queue);

        let mut vertices: Vec<Vertex> = Vec::new();
        let simulation_instances = self.simulation.bodies();

        vertices.extend_from_slice(&random_triangle([0.0, 0.0], 5.0));
        let mut instances: Vec<render::Instance> = Vec::new();

        for i in simulation_instances {
            instances.push(i.into());
        }

        let frame = self
            .surface
            .as_ref()
            .expect("surface not available")
            .get_current_texture()
            .expect("Unable to get frame");

        let tex_view = TextureViewDescriptor {
            label: Some("Custom Texture View"),
            ..Default::default()
        };
        let view: TextureView = frame.texture.create_view(&tex_view);

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    depth_slice: None,
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
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
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.set_bind_group(0, self.camera_plugin.bind_group(), &[]); // Use camera plugin's bind group
            rpass.draw(0..vertices.len() as u32, 0..instances.len() as u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    fn handle_keyboard_input(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(keycode);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&keycode);
                }
            }
        }
    }
}
