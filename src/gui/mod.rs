use std::sync::Arc;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct Gui {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl Gui {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, window: &Window) -> Self {
        let ctx = egui::Context::default();

        let state = egui_winit::State::new(ctx.clone(), egui::ViewportId::ROOT, window, None, None);

        let renderer = egui_wgpu::Renderer::new(device, format, None, 1);

        Self {
            ctx,
            state,
            renderer,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn run(
        &mut self,
        window: &Window,
        run_ui: impl FnOnce(&egui::Context),
    ) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);
        self.ctx.run(raw_input, run_ui)
    }

    pub fn handle_platform_output(
        &mut self,
        window: &Window,
        platform_output: egui::PlatformOutput,
    ) {
        self.state.handle_platform_output(window, platform_output);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        view: &wgpu::TextureView,
        full_output: egui::FullOutput,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let tris = self
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        // Update textures
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        // Update buffers
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        }

        // Free textures
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
