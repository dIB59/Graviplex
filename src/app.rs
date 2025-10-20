use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::core::time::Time;
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
}

impl Default for App {
    fn default() -> Self {
        let mut simulation = Simulation::default();
        simulation.generate_bodies(25000);

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
            self.pipeline = Some(RenderPipeline::new(&self.gpu.device, format, &self.camera));

            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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

        // Update camera
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

        // Update simulation based on time
        self.simulation.update(0.01);

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

            frame.present();
        }
    }
}
