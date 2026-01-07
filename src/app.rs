use std::sync::Arc;
use wgpu::include_wgsl;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::core::time::Time;
use crate::gui::gui_renderer::UiPipeline;
use crate::gui::Gui;
use crate::input::InputState;
use crate::renderer::{Camera2D, CameraController, GpuContext, Instance, RenderPipeline, Vertex};
use crate::simulation::Simulation;

pub struct App {
    window: Option<Arc<Window>>,
    gpu: GpuContext,
    pipeline: Option<RenderPipeline>,
    camera: Camera2D,
    camera_controller: CameraController,
    time: Time,
    input: InputState,
    simulation: Simulation,
    gui: Option<Gui>,
    gui_renderer: Option<UiPipeline>,
}

pub const NUM_OF_BODIES: i32 = 50000;

impl Default for App {
    fn default() -> Self {
        let mut simulation = Simulation::default();
        simulation.generate_bodies(NUM_OF_BODIES);

        Self {
            window: None,
            gpu: GpuContext::new(),
            pipeline: None,
            camera: Camera2D::new([0.0, 0.0], 10.0, [1200.0, 1200.0]),
            camera_controller: CameraController::new()
                .with_move_speed(250.0)
                .with_zoom_range(0.01, 10.0),
            time: Time::new(),
            input: InputState::new(),
            simulation,
            gui: None,
            gui_renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Simulation"))
                    .expect("Unable to create window"),
            );

            let scale_factor = window.scale_factor();
            println!("Window scale factor: {}", scale_factor);

            self.gpu.init_surface(window.clone());

            let size = window.inner_size();
            self.camera.screen_size = [size.width as f32, size.height as f32];

            let format = self
                .gpu
                .config
                .as_ref()
                .expect("Unable to get TextureFormat")
                .format;

            self.pipeline = Some(RenderPipeline::new(
                "Circle Shader",
                include_wgsl!("../src/shaders/circle_shader.wgsl"),
                &self.gpu.device,
                format,
                &self.camera,
            ));

            // Create GUI FIRST
            let mut gui = Gui::new(event_loop);

            // Run one frame to generate font texture delta
            log::debug!("Running initial egui frame to generate font texture...");
            let initial_output = gui.run(&window);
            log::debug!(
                "Initial texture deltas: set={}, free={}",
                initial_output.textures_delta.set.len(),
                initial_output.textures_delta.free.len()
            );

            // NOW create the UI pipeline
            let mut ui_pipeline = UiPipeline::new(&self.gpu.device, &self.gpu.queue, format);

            // Handle the initial texture deltas (this includes the font texture!)
            ui_pipeline.handle_textures(initial_output.textures_delta);

            self.window = Some(window);
            self.gui = Some(gui);
            self.gui_renderer = Some(ui_pipeline);

            log::debug!("Initialization complete!");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(gui) = &mut self.gui {
            if let Some(window) = &self.window {
                let response = gui.handle_event(window, &event);
                // If GUI consumed the event, don't pass it to your app
                if response.consumed {
                    return;
                }
            }
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.gpu.resize(size.width, size.height);
                self.camera.screen_size = [size.width as f32, size.height as f32];
                if let Some(pipeline) = &self.pipeline {
                    pipeline
                        .camera_gpu_data()
                        .update(&self.gpu.queue, &self.camera);
                }
            }

            WindowEvent::RedrawRequested => self.render(),

            WindowEvent::KeyboardInput { event, .. } => {
                self.input.handle_keyboard_event(event);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if self
                    .camera_controller
                    .handle_scroll(&mut self.camera, delta)
                {
                    if let Some(pipeline) = &self.pipeline {
                        pipeline
                            .camera_gpu_data()
                            .update(&self.gpu.queue, &self.camera);
                    }
                }
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

impl App {
    fn render(&mut self) {
        self.time.update();
        self.simulation.update(self.time.delta());

        // 2. Update camera if it moved
        if self.camera_controller.update_movement(
            &mut self.camera,
            self.time.delta(),
            self.input.pressed_keys(),
        ) {
            if let Some(pipeline) = &self.pipeline {
                pipeline
                    .camera_gpu_data()
                    .update(&self.gpu.queue, &self.camera);
            }
        }

        // Prepare render data
        let vertices = vec![
            Vertex { pos: [0.0, 5.0] },
            Vertex { pos: [4.33, -2.5] },
            Vertex { pos: [-4.33, -2.5] },
        ];
        let instances: Vec<Instance> = self
            .simulation
            .bodies()
            .iter()
            .map(Instance::from)
            .collect();

        // Render
        if let Ok(frame) = self.gpu.get_current_frame() {
            let view = frame.texture.create_view(&Default::default());

            if let Some(pipeline) = &self.pipeline {
                pipeline.render(
                    &self.gpu.device,
                    &self.gpu.queue,
                    &view,
                    &vertices,
                    &instances,
                );
            }

            if let Some(ui) = &mut self.gui_renderer {
                if let Some(gui) = &mut self.gui {
                    // 1. single egui frame
                    let full = gui.run(&self.window.clone().expect("WINDOW NOT FOUND FOR UI")); // shapes + textures

                    ui.handle_textures(full.textures_delta);

                    let prim = gui.tessellate(full.shapes, full.pixels_per_point);

                    // 2. flatten to slices UiPipeline expects
                    let (mut vtx, mut idx) = (vec![], vec![]);
                    for p in &prim {
                        if let egui::epaint::Primitive::Mesh(ref m) = p.primitive {
                            let base = vtx.len() as u32;
                            vtx.extend_from_slice(&m.vertices);
                            idx.extend(m.indices.iter().map(|i| base + i));
                        }
                    }

                    // 3. draw
                    let mut enc = self.gpu.device.create_command_encoder(&Default::default());

                    // Get a reference to the window (unwrap since we know it exists at render time)
                    let window = self.window.as_ref().unwrap();

                    // Get the window's inner size in physical pixels
                    // Physical pixels = actual pixels on the screen (affected by DPI/monitor scaling)
                    let size = window.inner_size();

                    // Get the scale factor (e.g., 1.0 for standard displays, 2.0 for Retina/HiDPI)
                    // This tells us how many physical pixels = 1 logical pixel
                    let scale_factor = window.scale_factor();

                    // Convert physical pixels to logical pixels for egui
                    // Logical pixels are what egui uses internally - they're DPI-independent
                    // For example: 1920 physical pixels ÷ 2.0 scale = 960 logical pixels
                    let logical_size = [
                        size.width as f32 / scale_factor as f32,  // Logical width
                        size.height as f32 / scale_factor as f32, // Logical height
                    ];

                    // Pass the logical size to the UI renderer
                    // This ensures egui's coordinate system matches what the user sees
                    ui.render(&mut enc, &view, &vtx, &idx, &prim, logical_size);
                    self.gpu.queue.submit(std::iter::once(enc.finish()));
                }
            }
            frame.present();
        }
    }
}
