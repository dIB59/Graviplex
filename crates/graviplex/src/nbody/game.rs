//! NBodyGame - Implements GameLoop for the n-body simulation

use graviplex_engine::{Camera2D, GameLoop, GpuContext, InputState};

use crate::nbody::GpuEngine;

/// N-body simulation game.
pub struct NBodyGame {
    pub particle_count: u32,
    pub gpu_engine: Option<GpuEngine>,
    pub gravity: f32,
    pub theta: f32,
    pub show_quadtree: bool,
}

impl NBodyGame {
    /// Create a new n-body simulation with the given particle count.
    pub fn new(particle_count: u32) -> Self {
        Self {
            particle_count,
            gpu_engine: None,
            gravity: 500.0,
            theta: 0.5,
            show_quadtree: false,
        }
    }
}

impl GameLoop for NBodyGame {
    fn init(&mut self, gpu: &GpuContext) {
        let engine = GpuEngine::new(&gpu.device, &gpu.queue, self.particle_count);
        engine.init(&gpu.queue);
        self.gpu_engine = Some(engine);
    }

    fn update(&mut self, dt: f32, gpu: &GpuContext) {
        if let Some(engine) = &self.gpu_engine {
            engine.update(&gpu.device, &gpu.queue, dt, self.gravity, self.theta);
        }
    }

    fn render(&mut self, _gpu: &GpuContext, _view: &wgpu::TextureView, _camera: &Camera2D) {
        // Particles are rendered via instance_buffer
        // Additional rendering (quadtree lines) can be added here
    }

    fn handle_input(&mut self, _input: &InputState, _camera: &Camera2D) -> bool {
        false
    }

    fn gui(&mut self, ctx: &egui::Context) {
        egui::Window::new("N-Body Simulation")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Physics");
                ui.add(egui::Slider::new(&mut self.gravity, 0.0..=1000.0).text("Gravity"));
                ui.add(egui::Slider::new(&mut self.theta, 0.1..=1.5).text("Theta (Accuracy)"));

                ui.separator();
                ui.heading("Info");
                ui.label(format!("Particles: {}", self.particle_count));

                ui.separator();
                ui.heading("Debug");
                ui.checkbox(&mut self.show_quadtree, "Show Quadtree");
            });
    }

    fn instance_count(&self) -> u32 {
        self.particle_count
    }

    fn instance_buffer(&self) -> Option<&wgpu::Buffer> {
        self.gpu_engine.as_ref().map(|e| &e.particle_buffer)
    }
}
