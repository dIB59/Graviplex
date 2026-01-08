use crate::simulation::SimulationCommand;
use std::sync::mpsc::Sender;

pub mod gui_renderer;

pub struct Gui {
    ctx: egui::Context,
    state: egui_winit::State,
    sender: Sender<SimulationCommand>,
    // Local UI state
    gravity_constant: f64,
    theta: f64,
    paused: bool,
    particle_count: i32,
    click_count: u32,
}

impl Gui {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        sender: Sender<SimulationCommand>,
    ) -> Self {
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::default());
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            None,
            None,
            None,
        );
        Self {
            ctx,
            state,
            sender,
            gravity_constant: 100.0,
            theta: 0.5,
            paused: false,
            particle_count: crate::app::NUM_OF_BODIES,
            click_count: 0,
        }
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn run(&mut self, window: &winit::window::Window, fps: f32, tps: f32) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);

        let sender = &self.sender;
        let gravity = &mut self.gravity_constant;
        let theta = &mut self.theta;
        let paused = &mut self.paused;
        let particle_count = &mut self.particle_count;
        let click_count = &mut self.click_count;

        self.ctx.run(raw_input, |ctx| {
            Self::build_ui(
                ctx,
                sender,
                gravity,
                theta,
                paused,
                particle_count,
                click_count,
                fps,
                tps,
            )
        })
    }

    pub fn handle_platform_output(
        &mut self,
        window: &winit::window::Window,
        platform_output: egui::PlatformOutput,
    ) {
        self.state.handle_platform_output(window, platform_output);
    }

    pub fn tessellate(
        &self,
        shapes: Vec<egui::epaint::ClippedShape>,
        pixels_per_point: f32,
    ) -> Vec<egui::ClippedPrimitive> {
        self.ctx.tessellate(shapes, pixels_per_point)
    }

    fn build_ui(
        ctx: &egui::Context,
        sender: &Sender<SimulationCommand>,
        gravity: &mut f64,
        theta: &mut f64,
        paused: &mut bool,
        particle_count: &mut i32,
        click_count: &mut u32,
        fps: f32,
        tps: f32,
    ) {
        egui::Window::new("Simulation Controls")
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Statistics");
                ui.label(format!("Bodies: {}", crate::app::NUM_OF_BODIES));
                ui.label(format!("FPS: {:.1}", fps));
                ui.label(format!("TPS: {:.1}", tps));
                ui.separator();

                ui.heading("Physics");
                if ui
                    .add(egui::Slider::new(gravity, 0.0..=1000.0).text("Gravity (G)"))
                    .changed()
                {
                    let _ = sender.send(SimulationCommand::UpdateGravity(*gravity));
                }

                if ui
                    .add(egui::Slider::new(theta, 0.1..=1.5).text("Theta (Accuracy)"))
                    .changed()
                {
                    let _ = sender.send(SimulationCommand::SetTheta(*theta));
                }

                ui.horizontal(|ui| {
                    if ui.checkbox(paused, "Paused").changed() {
                        let _ = sender.send(SimulationCommand::Pause(*paused));
                    }
                    if *paused {
                        if ui.button("Step").clicked() {
                            let _ = sender.send(SimulationCommand::Step);
                        }
                    }
                });

                ui.heading("World");
                ui.add(egui::Slider::new(particle_count, 100..=500000).text("Particle Count"));

                if ui.button("Regenerate Simulation").clicked() {
                    let _ = sender.send(SimulationCommand::Reset(*particle_count));
                }

                ui.separator();
                ui.heading("Misc");
                if ui.button("Magic Click").clicked() {
                    *click_count += 1;
                }
                ui.label(format!("Magic Clicks: {}", click_count));
            });
    }
}
