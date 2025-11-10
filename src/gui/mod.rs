use crate::app::NUM_OF_BODIES;

pub mod gui_renderer;

pub struct Gui {
    ctx: egui::Context,
    state: egui_winit::State,
    click_count: u32,
}

impl Gui {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::default()); // normal fonts
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

    pub fn run(&mut self, window: &winit::window::Window) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);
        let click_count = &mut self.click_count;
        self.ctx
            .run(raw_input, |ctx| Self::build_ui(ctx, click_count))
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

    pub fn build_ui(ctx: &egui::Context, click_count: &mut u32) {
        egui::Window::new("Simulation Controls")
            .default_width(1500.0)
            .show(ctx, |ui| {
                ui.heading("Statistics");
                ui.label(format!("Bodies: {}", NUM_OF_BODIES));
                ui.separator();

                let button =
                    egui::Button::new("Click Me").min_size(egui::Vec2 { x: 100.0, y: 100.0 });

                if ui.add(button).clicked() {
                    *click_count += 1;
                    println!("Button clicked {} times", click_count);
                }

                ui.label(format!("Button clicked: {} times", click_count));
            });
    }
}
