//! egui integration for the Graviplex engine.
//!
//! This module is feature-gated behind the `gui` feature (enabled by default).
//!
//! Games can implement the `GameLoop::gui()` method to add custom UI elements.
//! For FPS display, use the `FpsPlugin` from `graviplex::plugin`.

pub mod gui_renderer;

pub use gui_renderer::UiPipeline as EguiRenderer;

use crate::core::stats::FpsCounter;

/// Gui wrapper for egui integration.
/// Games add their own UI via the GameLoop::gui() method.
pub struct Gui {
    ctx: egui::Context,
    state: egui_winit::State,
}

/// Output from running the GUI
pub struct GuiOutput {
    pub shapes: Vec<egui::epaint::ClippedShape>,
    pub textures_delta: egui::TexturesDelta,
    pub pixels_per_point: f32,
}

impl Gui {
    /// Create a new Gui instance.
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
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
        Self { ctx, state }
    }

    /// Handle a window event.
    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    /// Get the egui context (for games to add UI).
    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    /// Begin a frame (call before GameLoop::gui).
    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.ctx.begin_pass(raw_input);
    }

    /// End a frame (call after GameLoop::gui).
    /// 
    /// Note: FPS display is now handled by `FpsPlugin`. Add it via
    /// `.add_plugin(FpsPlugin::default())` on your `AppBuilder`.
    pub fn end_frame(&mut self, _fps_counter: &FpsCounter) -> GuiOutput {
        let output = self.ctx.end_pass();
        GuiOutput {
            shapes: output.shapes,
            textures_delta: output.textures_delta,
            pixels_per_point: output.pixels_per_point,
        }
    }

    /// Run an empty frame (used during initialization).
    pub fn run_empty(&mut self, window: &winit::window::Window) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);
        self.ctx.run(raw_input, |_ctx| {})
    }

    /// Tessellate shapes into primitives.
    pub fn tessellate(
        &self,
        shapes: Vec<egui::epaint::ClippedShape>,
        pixels_per_point: f32,
    ) -> Vec<egui::ClippedPrimitive> {
        self.ctx.tessellate(shapes, pixels_per_point)
    }
}
