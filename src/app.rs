use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Couldn't create window"),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("{:?}", window_id);
                log::info!("Close button pressed. Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                log::info!("{:?}", window_id);
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
