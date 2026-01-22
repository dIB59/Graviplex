//! Generic App struct that runs games implementing GameLoop

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::core::time::Time;
use crate::gui::gui_renderer::UiPipeline;
use crate::gui::Gui;
use crate::input::InputState;
use crate::renderer::{
    Camera2D, CameraController, CirclePipeline, DrawContext, GpuContext, LinePipeline,
};
use crate::GameLoop;

/// The main application struct that manages the game loop.
///
/// Generic over `T: GameLoop` - your game implements this trait.
pub struct App<T: GameLoop> {
    game: T,
    window: Option<Arc<Window>>,
    gpu: GpuContext,
    camera: Camera2D,
    camera_controller: CameraController,
    time: Time,
    input: InputState,
    gui: Option<Gui>,
    gui_renderer: Option<UiPipeline>,
    circle_pipeline: Option<CirclePipeline>,
    line_pipeline: Option<LinePipeline>,
    exit_time: Option<f32>,
}

impl<T: GameLoop> App<T> {
    /// Create a new App with the given game.
    pub fn new(game: T) -> Self {
        Self {
            game,
            window: None,
            gpu: GpuContext::new(),
            camera: Camera2D::new([0.0, 0.0], 10.0, [1200.0, 1200.0]),
            camera_controller: CameraController::new()
                .with_move_speed(250.0)
                .with_zoom_range(0.0001, 10.0),
            time: Time::new(),
            input: InputState::new(),
            gui: None,
            gui_renderer: None,
            circle_pipeline: None,
            line_pipeline: None,
            exit_time: None,
        }
    }

    /// Set an optional exit time in seconds.
    pub fn with_exit_time(mut self, seconds: f32) -> Self {
        self.exit_time = Some(seconds);
        self
    }

    /// Run the application and return performance statistics.
    pub fn run(self) -> Result<crate::AppStats, winit::error::EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = self;
        event_loop.run_app(&mut app)?;

        Ok(crate::AppStats {
            average_fps: app.time.average_fps(),
            frame_count: app.time.frame_count(),
            total_time: app.time.elapsed(),
        })
    }

    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }

        self.time.update();

        // Check for timed exit
        if let Some(exit_time) = self.exit_time {
            if self.time.elapsed() >= exit_time {
                event_loop.exit();
                return;
            }
        }

        // Update camera
        self.camera_controller.update_movement(
            &mut self.camera,
            self.time.delta(),
            self.input.pressed_keys(),
        );

        // Let game handle input
        self.game.handle_input(&self.input, &self.camera);

        // Update game
        self.game.update(self.time.delta() as f32, &self.gpu);

        let Ok(frame) = self.gpu.get_current_frame() else {
            return;
        };
        let view = frame.texture.create_view(&Default::default());

        // Let game render its content using DrawContext
        if let (Some(circle_pipeline), Some(line_pipeline)) =
            (&mut self.circle_pipeline, &mut self.line_pipeline)
        {
            let mut draw = DrawContext {
                gpu: &self.gpu,
                view: &view,
                camera: &self.camera,
                circle_pipeline,
                line_pipeline,
            };

            self.game.render(&mut draw);

            // Automatically flush at the end of the frame
            draw.flush();
        }

        // Render GUI
        self.render_gui(&view);

        frame.present();
    }

    fn render_gui(&mut self, view: &wgpu::TextureView) {
        let (Some(window), Some(ui_renderer), Some(gui)) =
            (&self.window, &mut self.gui_renderer, &mut self.gui)
        else {
            return;
        };

        // Let game add GUI elements
        gui.begin_frame(window);
        self.game.gui(gui.ctx());
        let full = gui.end_frame(self.time.fps());

        ui_renderer.handle_textures(full.textures_delta);
        let primitives = gui.tessellate(full.shapes, full.pixels_per_point);

        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        let physical_size = [size.width as f32, size.height as f32];
        ui_renderer.render(
            &mut encoder,
            view,
            &primitives,
            physical_size,
            scale_factor as f32,
        );
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
    }
}

impl<T: GameLoop> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Graviplex"))
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

        let mut gui = Gui::new(event_loop);
        let initial_output = gui.run_empty(&window);
        let mut ui_pipeline = UiPipeline::new(&self.gpu.device, &self.gpu.queue, format);
        ui_pipeline.handle_textures(initial_output.textures_delta);

        // Initialize standard pipelines
        let circle_pipeline = CirclePipeline::new(&self.gpu.device, format, &self.camera);
        let line_pipeline = LinePipeline::new(
            "Default",
            wgpu::include_wgsl!("shaders/line_shader.wgsl"),
            &self.gpu.device,
            format,
            &self.camera,
        );

        self.circle_pipeline = Some(circle_pipeline);
        self.line_pipeline = Some(line_pipeline);

        // Initialize game
        self.game.init(&self.gpu);

        self.window = Some(window);
        self.gui = Some(gui);
        self.gui_renderer = Some(ui_pipeline);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let GUI handle event first
        if let (Some(gui), Some(window)) = (&mut self.gui, &self.window) {
            if gui.handle_event(window, &event).consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.gpu.resize(size.width, size.height);
                self.camera.screen_size = [size.width as f32, size.height as f32];
            }

            WindowEvent::RedrawRequested => self.render_frame(event_loop),

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
                self.camera_controller
                    .handle_scroll(&mut self.camera, delta);
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
