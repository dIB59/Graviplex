use std::collections::{hash_set, HashSet};
use std::sync::Arc;

use crate::render::View;
use crate::render::{self, Vertex};
use crate::render_backend::pipeline_builder::PipelineBuilder;
use pollster::FutureExt;
use rand::Rng;
use ultraviolet::Vec2;
use wgpu::util::DeviceExt;
use wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
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
    view: View,
    view_buffer: wgpu::Buffer,
    view_bind_group: wgpu::BindGroup,
    last_frame_time: std::time::Instant,
    pressed_keys: hash_set::HashSet<KeyCode>,
    camera_speed: f32,
    zoom_speed: f32,
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

        let view = View {
            position: [0.0, 0.0],
            scale: 400.0,
            zoom_speed: 1.0,
            screen_size: [800.0, 800.0],
        };

        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Buffer"),
            contents: bytemuck::cast_slice(&[view]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let view_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("View Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let view_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("View Bind Group"),
            layout: &view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &view_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let render_pipeline = PipelineBuilder::new("shader.wgsl", &device)
            .add_bind_group_layout(&view_bind_group_layout)
            .add_vertex_buffer_layout(render::Vertex::desc())
            .add_vertex_buffer_layout(render::Instance::desc())
            .build_pipeline();

        Self {
            window: None,
            surface: None,
            config: None,
            queue,
            device,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            view,
            view_buffer,
            view_bind_group,
            last_frame_time: std::time::Instant::now(),
            pressed_keys: HashSet::new(),
            camera_speed: 5.0, // Units per second
            zoom_speed: 1.1,   // Zoom factor per scroll step
        }
    }
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

        let view = View {
            position: [0.0, 0.0],
            scale: 400.0,
            zoom_speed: 1.0,
            screen_size: [800.0, 800.0],
        };

        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Buffer"),
            contents: bytemuck::cast_slice(&[view]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let view_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("View Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let view_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("View Bind Group"),
            layout: &view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &view_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let render_pipeline = PipelineBuilder::new("shader.wgsl", &device)
            .add_bind_group_layout(&view_bind_group_layout)
            .add_vertex_buffer_layout(render::Vertex::desc())
            .add_vertex_buffer_layout(render::Instance::desc())
            .set_pixel_format(format)
            .build_pipeline();

        self.config = Some(surface_config);
        self.surface = Some(surface);
        self.queue = queue;
        self.device = device;
        self.render_pipeline = render_pipeline;
        self.view = view;
        self.view_buffer = view_buffer;
        self.view_bind_group = view_bind_group;
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
                }
            }
            WindowEvent::RedrawRequested => {
                // Check if surface exists first
                if self.surface.is_some() {
                    // Move render_frame call here to avoid borrowing conflicts
                    self.render_frame();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                println!("{:?}", event);
                self.handle_keyboard_input(event);
            }
            // Handle mouse scroll for zooming
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_scroll(delta);
            }
            _ => (),
        }
    }

    // Make sure to request redraws continuously for smooth movement
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
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

    let mut rng = rand::rng();
    for (i, base) in base_vertices.iter().enumerate() {
        let mut jitter = |v: f32| v + rng.random_range(-0.25..0.25);
        let x = jitter(base[0]) + center[0];
        let y = jitter(base[1]) + center[1];

        vertices[i] = Vertex { pos: [x, y] };
    }

    vertices
}

impl App {
    fn render_frame(&mut self) {
        {
            self.update_camera_from_input();
        }
        let mut vertices = Vec::new();
        vertices.extend_from_slice(&random_triangle([0.0, 0.0], 0.5));
        let mut instances = Vec::new();

        for _ in 0..5_000 {
            instances.push(render::Instance::random());
        }
        let frame = self
            .surface
            .as_ref()
            .expect("wos")
            .get_current_texture()
            .expect("Unable to get frame");
        let tex_view = TextureViewDescriptor {
            label: Some("CUSTOME TEXTURE VIEW DES"),
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
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // per-instance
            rpass.set_bind_group(0, &self.view_bind_group, &[]); // Bind the bind group at index 0
            rpass.draw(0..vertices.len() as u32, 0..instances.len() as u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    // Process input and update camera
    fn update_camera_from_input(&mut self) {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Calculate movement speed - use a more reasonable formula
        // Base speed that feels good, adjusted for zoom level
        let base_speed = 200.0; // Much higher base speed
        let movement_speed = base_speed / self.view.scale * delta_time;

        println!(
            "Delta time: {:.4}, Movement speed: {:.6}, Scale: {:.2}",
            delta_time, movement_speed, self.view.scale
        );
        println!("Pressed keys: {:?}", self.pressed_keys);

        let mut movement = [0.0f32; 2];

        // WASD movement - removed the extra multipliers for now
        if self.pressed_keys.contains(&KeyCode::KeyW) {
            movement[1] += movement_speed;
            log::debug!("Moving UP: {}", movement_speed);
        }
        if self.pressed_keys.contains(&KeyCode::KeyS) {
            movement[1] -= movement_speed;
            log::debug!("Moving DOWN: {}", movement_speed);
        }
        if self.pressed_keys.contains(&KeyCode::KeyA) {
            movement[0] -= movement_speed;
            log::debug!("Moving LEFT: {}", movement_speed);
        }
        if self.pressed_keys.contains(&KeyCode::KeyD) {
            movement[0] += movement_speed;
            log::debug!("Moving RIGHT: {}", movement_speed);
        }

        // Apply movement
        if movement[0] != 0.0 || movement[1] != 0.0 {
            let old_pos = self.view.position;
            self.view.position[0] += movement[0];
            self.view.position[1] += movement[1];

            log::debug!(
                "Camera moved from [{:.4}, {:.4}] to [{:.4}, {:.4}]",
                old_pos[0],
                old_pos[1],
                self.view.position[0],
                self.view.position[1]
            );

            // Update the buffer
            self.queue
                .write_buffer(&self.view_buffer, 0, bytemuck::cast_slice(&[self.view]));
        }
    }
    fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        let zoom_factor = match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    self.zoom_speed
                } else {
                    1.0 / self.zoom_speed
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                let y = pos.y as f32;
                if y > 0.0 {
                    1.0 + (y * 0.01)
                } else {
                    1.0 / (1.0 + (-y * 0.01))
                }
            }
        };

        self.view.scale *= zoom_factor;

        // Clamp zoom to reasonable bounds (higher max for Retina displays)
        self.view.scale = self.view.scale.clamp(10.0, 5000.0);

        // Update the buffer
        self.queue
            .write_buffer(&self.view_buffer, 0, bytemuck::cast_slice(&[self.view]));
    }

    // Handle key press/release
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
