//! Generic App struct that runs games implementing GameLoop

use std::sync::Arc;
use wgpu::include_wgsl;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::core::time::Time;
use crate::gui::gui_renderer::UiPipeline;
use crate::gui::Gui;
use crate::input::InputState;
use crate::renderer::{
    Camera2D, CameraController, GpuContext, LinePipeline, RenderPipeline, Vertex,
};
use crate::GameLoop;

/// The main application struct that manages the game loop.
///
/// Generic over `T: GameLoop` - your game implements this trait.
pub struct App<T: GameLoop> {
    game: T,
    window: Option<Arc<Window>>,
    gpu: GpuContext,
    pipeline: Option<RenderPipeline>,
    line_pipeline: Option<LinePipeline>,
    camera: Camera2D,
    camera_controller: CameraController,
    time: Time,
    input: InputState,
    gui: Option<Gui>,
    gui_renderer: Option<UiPipeline>,
}

impl<T: GameLoop> App<T> {
    /// Create a new App with the given game.
    pub fn new(game: T) -> Self {
        Self {
            game,
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
            gui: None,
            gui_renderer: None,
        }
    }

    /// Run the application.
    pub fn run(self) -> Result<(), winit::error::EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = self;
        event_loop.run_app(&mut app)
    }

    fn render_frame(&mut self) {
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

        // Let game handle input
        self.game.handle_input(&self.input, &self.camera);

        // Update game
        self.game.update(self.time.delta() as f32, &self.gpu);

        let vertices = vec![
            Vertex { pos: [0.0, 5.0] },
            Vertex { pos: [4.33, -2.5] },
            Vertex { pos: [-4.33, -2.5] },
        ];

        if let Ok(frame) = self.gpu.get_current_frame() {
            let view = frame.texture.create_view(&Default::default());

            // Render particles via the game's instance buffer
            if let Some(pipeline) = &self.pipeline {
                let ext_buffer = self.game.instance_buffer();
                pipeline.render(
                    &self.gpu.device,
                    &self.gpu.queue,
                    &view,
                    &vertices,
                    self.game.instance_count(),
                    ext_buffer,
                );
            }

            // Let game render additional content
            self.game.render(&self.gpu, &view, &self.camera);

            // Render GUI
            if let Some(ui_renderer) = &mut self.gui_renderer {
                if let Some(gui) = &mut self.gui {
                    // Let game add GUI elements
                    gui.begin_frame(self.window.as_ref().unwrap());
                    self.game.gui(gui.ctx());
                    let full = gui.end_frame(self.time.fps());

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

impl<T: GameLoop> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Graviplex Engine"))
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
                include_wgsl!("shaders/circle_shader.wgsl"),
                &self.gpu.device,
                format,
                &self.camera,
            ));

            self.line_pipeline = Some(LinePipeline::new(
                "Lines",
                include_wgsl!("shaders/line_shader.wgsl"),
                &self.gpu.device,
                format,
                &self.camera,
            ));

            let mut gui = Gui::new(event_loop);
            let initial_output = gui.run_empty(&window);
            let mut ui_pipeline = UiPipeline::new(&self.gpu.device, &self.gpu.queue, format);
            ui_pipeline.handle_textures(initial_output.textures_delta);

            // Initialize game
            self.game.init(&self.gpu);

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

            WindowEvent::RedrawRequested => self.render_frame(),

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
