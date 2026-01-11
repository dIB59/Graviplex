use crate::simulation::SimulationCommand;
use std::sync::mpsc::Sender;

pub mod gui_renderer;

pub struct Gui {
    ctx: egui::Context,
    state: egui_winit::State,
    sender: Sender<SimulationCommand>,
    // Local UI state
    gravity_constant: f32,
    theta: f64,
    paused: bool,
    particle_count: i32,
    interaction_radius: f32,
    interaction_strength: f32,
    click_count: u32,
    show_quadtree: bool,
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
            interaction_radius: 100.0,
            interaction_strength: 10.0,
            click_count: 0,
            show_quadtree: false,
        }
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn run(
        &mut self,
        window: &winit::window::Window,
        fps: f32,
        tps: f32,
        body_count: usize,
        camera_scale: f32,
        is_interacting: bool,
    ) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);

        let world_radius = self.interaction_radius;
        let interaction_strength_val = self.interaction_strength;
        let pixels_per_point = self.ctx.pixels_per_point();

        self.ctx.run(raw_input, |ctx| {
            if is_interacting {
                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let color = if interaction_strength_val > 0.0 {
                        egui::Color32::from_rgba_unmultiplied(100, 200, 255, 40)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(255, 100, 100, 40)
                    };

                    // Scale world radius to logical pixels (points)
                    let visual_radius = (world_radius * camera_scale) / pixels_per_point;

                    ctx.debug_painter().circle_stroke(
                        pos,
                        visual_radius,
                        egui::Stroke::new(2.0, color),
                    );
                    ctx.debug_painter().circle_filled(pos, visual_radius, color);
                }
            }

            Self::build_ui(
                ctx,
                &self.sender,
                &mut self.gravity_constant,
                &mut self.theta,
                &mut self.paused,
                &mut self.particle_count,
                &mut self.interaction_radius,
                &mut self.interaction_strength,
                body_count,
                &mut self.click_count,
                &mut self.show_quadtree,
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
        gravity: &mut f32,
        theta: &mut f64,
        paused: &mut bool,
        particle_count: &mut i32,
        interaction_radius: &mut f32,
        interaction_strength: &mut f32,
        body_count: usize,
        click_count: &mut u32,
        show_quadtree: &mut bool,
        fps: f32,
        tps: f32,
    ) {
        egui::Window::new("Simulation Controls")
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Statistics");
                ui.label(format!("Bodies: {}", body_count));
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

                ui.heading("Interaction");
                ui.add(egui::Slider::new(interaction_radius, 10.0..=5000.0).text("Radius"));
                ui.add(egui::Slider::new(interaction_strength, 10.0..=1000.0).text("Strength"));
                ui.label("Left-click: Pull | Right-click: Repel");

                ui.separator();
                ui.heading("Debug");
                ui.checkbox(show_quadtree, "Show Quadtree");
            });
    }
    pub fn interaction_params(&self) -> (f32, f32) {
        (self.interaction_radius, self.interaction_strength)
    }

    pub fn show_quadtree(&self) -> bool {
        self.show_quadtree
    }
}
