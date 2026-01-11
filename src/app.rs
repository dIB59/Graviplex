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
use crate::renderer::{
    Camera2D, CameraController, GpuContext, LineInstance, LinePipeline, RenderPipeline, Vertex,
};
use crate::simulation::SimulationBridge;

pub struct App {
    window: Option<Arc<Window>>,
    gpu: GpuContext,
    pipeline: Option<RenderPipeline>,
    line_pipeline: Option<LinePipeline>,
    camera: Camera2D,
    camera_controller: CameraController,
    time: Time,
    input: InputState,
    simulation_bridge: SimulationBridge,
    gpu_simulation: Option<crate::simulation::GpuEngine>,
    gui: Option<Gui>,
    gui_renderer: Option<UiPipeline>,
}

pub const NUM_OF_BODIES: i32 = 1 << 14;

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            gpu: GpuContext::new(),
            pipeline: None,
            line_pipeline: None,
            camera: Camera2D::new([0.0, 0.0], 10.0, [1200.0, 1200.0]),
            camera_controller: CameraController::new()
                .with_move_speed(250.0)
                .with_zoom_range(0.0001, 10.0),
            time: Time::new(),
            input: InputState::new(),
            simulation_bridge: SimulationBridge::new(NUM_OF_BODIES),
            gpu_simulation: None,
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

            self.line_pipeline = Some(LinePipeline::new(
                "Quadtree Lines",
                include_wgsl!("../src/shaders/line_shader.wgsl"),
                &self.gpu.device,
                format,
                &self.camera,
            ));

            let mut gui = Gui::new(event_loop, self.simulation_bridge.sender());

            // Initialize GPU Simulation
            let gpu_sim = crate::simulation::GpuEngine::new(
                &self.gpu.device,
                &self.gpu.queue,
                NUM_OF_BODIES as u32,
            );
            gpu_sim.init(&self.gpu.queue);
            self.gpu_simulation = Some(gpu_sim);

            let initial_output = gui.run(
                &window,
                0.0,
                0.0,
                self.simulation_bridge.get_body_count(),
                self.camera.scale,
                false,
            );
            let mut ui_pipeline = UiPipeline::new(&self.gpu.device, &self.gpu.queue, format);
            ui_pipeline.handle_textures(initial_output.textures_delta);

            self.window = Some(window);
            self.gui = Some(gui);
            self.gui_renderer = Some(ui_pipeline);
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
                if let Some(line_pipeline) = &self.line_pipeline {
                    line_pipeline
                        .camera_gpu_data()
                        .update(&self.gpu.queue, &self.camera);
                }
            }

            WindowEvent::RedrawRequested => self.render(),

            WindowEvent::KeyboardInput { event, .. } => {
                self.input.handle_keyboard_event(event);
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.input.handle_cursor_moved(position);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                self.input.handle_mouse_input(state, button);
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
                    if let Some(line_pipeline) = &self.line_pipeline {
                        line_pipeline
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
            if let Some(line_pipeline) = &self.line_pipeline {
                line_pipeline
                    .camera_gpu_data()
                    .update(&self.gpu.queue, &self.camera);
            }
        }

        // Particle Interaction
        let mut interaction_params = None;
        if let Some(gui) = &self.gui {
            let left_down = self.input.is_mouse_down(winit::event::MouseButton::Left);
            let right_down = self.input.is_mouse_down(winit::event::MouseButton::Right);

            if left_down || right_down {
                let (radius, strength) = gui.interaction_params();
                let mouse_pos = self.input.mouse_pos();
                let world_pos = self.camera.screen_to_world(mouse_pos);

                let actual_strength = if left_down {
                    strength as f64
                } else {
                    -(strength as f64) * 3.0 // Repel is slightly stronger for effect
                };

                interaction_params = Some((
                    [world_pos[0] as f32, world_pos[1] as f32],
                    radius,
                    actual_strength as f32,
                ));

                let _ = self.simulation_bridge.sender().send(
                    crate::simulation::bridge::SimulationCommand::Interaction {
                        pos: [world_pos[0] as f64, world_pos[1] as f64],
                        radius: radius as f64,
                        strength: actual_strength,
                    },
                );
            }
        }

        let vertices = vec![
            Vertex { pos: [0.0, 5.0] },
            Vertex { pos: [4.33, -2.5] },
            Vertex { pos: [-4.33, -2.5] },
        ];

        let instance_count = NUM_OF_BODIES as u32;

        if let Some(gpu_sim) = &self.gpu_simulation {
            let (gravity, theta) = self
                .gui
                .as_ref()
                .map(|g| (g.gravity_constant(), g.theta()))
                .unwrap_or((500.0, 0.5));
            gpu_sim.update(
                &self.gpu.device,
                &self.gpu.queue,
                self.time.delta() as f32,
                gravity as f32,
                theta as f32,
                interaction_params,
            );
        }

        if let Ok(frame) = self.gpu.get_current_frame() {
            let view = frame.texture.create_view(&Default::default());

            if let Some(pipeline) = &self.pipeline {
                let ext_buffer = self.gpu_simulation.as_ref().map(|s| &s.particle_buffer);
                pipeline.render(
                    &self.gpu.device,
                    &self.gpu.queue,
                    &view,
                    &vertices,
                    instance_count,
                    ext_buffer,
                );
            }

            // Render quadtree lines (if enabled)
            let show_quadtree = self
                .gui
                .as_ref()
                .map(|g| g.show_quadtree())
                .unwrap_or(false);
            if show_quadtree {
                if let Some(line_pipeline) = &self.line_pipeline {
                    let quad_cells = self.simulation_bridge.get_quad_cells();
                    let cells_read = quad_cells.read().unwrap();

                    // Convert quad cells to line instances (4 edges per cell)
                    let line_instances: Vec<LineInstance> = cells_read
                        .iter()
                        .flat_map(|cell| {
                            let half = cell.size * 0.5;
                            let corners = [
                                [cell.center[0] - half, cell.center[1] - half], // bottom-left
                                [cell.center[0] + half, cell.center[1] - half], // bottom-right
                                [cell.center[0] + half, cell.center[1] + half], // top-right
                                [cell.center[0] - half, cell.center[1] + half], // top-left
                            ];
                            let color = [0.2, 0.6, 1.0, 0.3]; // Semi-transparent blue
                            [
                                LineInstance {
                                    start: corners[0],
                                    end: corners[1],
                                    color,
                                },
                                LineInstance {
                                    start: corners[1],
                                    end: corners[2],
                                    color,
                                },
                                LineInstance {
                                    start: corners[2],
                                    end: corners[3],
                                    color,
                                },
                                LineInstance {
                                    start: corners[3],
                                    end: corners[0],
                                    color,
                                },
                            ]
                        })
                        .collect();

                    let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
                    line_pipeline.render(&mut encoder, &self.gpu.queue, &view, &line_instances);
                    self.gpu.queue.submit(std::iter::once(encoder.finish()));
                }
            }

            if let Some(ui_renderer) = &mut self.gui_renderer {
                if let Some(gui) = &mut self.gui {
                    let left_down = self.input.is_mouse_down(winit::event::MouseButton::Left);
                    let right_down = self.input.is_mouse_down(winit::event::MouseButton::Right);

                    let full = gui.run(
                        self.window.as_ref().expect("Window not found"),
                        self.time.fps(),
                        self.simulation_bridge.get_tps(),
                        self.simulation_bridge.get_body_count(),
                        self.camera.scale,
                        left_down || right_down,
                    );

                    ui_renderer.handle_textures(full.textures_delta);
                    let primitives = gui.tessellate(full.shapes, full.pixels_per_point);

                    let (mut vtx, mut idx) = (vec![], vec![]);
                    for p in &primitives {
                        if let egui::epaint::Primitive::Mesh(ref m) = p.primitive {
                            let base = vtx.len() as u32;
                            vtx.extend_from_slice(&m.vertices);
                            idx.extend(m.indices.iter().map(|i| base + i));
                        }
                    }

                    let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
                    let window = self.window.as_ref().unwrap();
                    let size = window.inner_size();
                    let scale_factor = window.scale_factor();

                    let physical_size = [size.width as f32, size.height as f32];
                    ui_renderer.render(
                        &mut encoder,
                        &view,
                        &vtx,
                        &idx,
                        &primitives,
                        physical_size,
                        scale_factor as f32,
                    );
                    self.gpu.queue.submit(std::iter::once(encoder.finish()));
                }
            }
            frame.present();
        }
    }
}
