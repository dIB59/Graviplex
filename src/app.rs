//! Application entry point and configuration builder

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::core::time::Time;
use crate::ecs::{Resources, World};
use crate::input::InputState;
use crate::renderer::{
    Camera2D, CameraController, CirclePipeline, DrawContext, LinePipeline,
};
use crate::renderer::gpu_context::GpuContext;
use crate::GameLoop;

#[cfg(feature = "gui")]
use crate::gui::gui_renderer::UiPipeline;
#[cfg(feature = "gui")]
use crate::gui::Gui;

#[cfg(feature = "textures")]
use crate::renderer::{SpritePipeline, TextureAtlas, AtlasBuilder};

// =============================================================================
// APP CONFIGURATION
// =============================================================================

/// Configuration for camera initialization.
#[derive(Clone, Debug)]
pub struct CameraConfig {
    pub position: [f32; 2],
    pub scale: f32,
    pub zoom_speed: f32,
    pub move_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            scale: 10.0,
            zoom_speed: 1.1,
            move_speed: 250.0,
            min_zoom: 0.0001,
            max_zoom: 10.0,
        }
    }
}

impl CameraConfig {
    /// Create a centered camera configuration.
    pub fn centered() -> Self {
        Self::default()
    }

    /// Set the initial scale (zoom level).
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set the initial position.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    /// Set the zoom speed multiplier.
    pub fn with_zoom_speed(mut self, speed: f32) -> Self {
        self.zoom_speed = speed;
        self
    }

    /// Set the camera movement speed.
    pub fn with_move_speed(mut self, speed: f32) -> Self {
        self.move_speed = speed;
        self
    }

    /// Set the zoom range.
    pub fn with_zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }
}

// =============================================================================
// APP BUILDER
// =============================================================================

/// Builder for configuring and creating an [`App`].
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
///
/// App::build(MyGame::new())
///     .title("My Awesome Game")
///     .size(1280, 720)
///     .vsync(true)
///     .camera(CameraConfig::centered().with_scale(100.0))
///     .run()
///     .unwrap();
/// ```
pub struct AppBuilder<T: GameLoop> {
    game: T,
    title: String,
    width: u32,
    height: u32,
    vsync: bool,
    camera_config: CameraConfig,
    exit_time: Option<f32>,
    #[cfg(feature = "textures")]
    atlas_builder: Option<AtlasBuilder>,
    #[cfg(feature = "textures")]
    atlas_size: u32,
}

impl<T: GameLoop> AppBuilder<T> {
    /// Create a new AppBuilder with default settings.
    pub fn new(game: T) -> Self {
        Self {
            game,
            title: "Graviplex".to_string(),
            width: 1200,
            height: 800,
            vsync: false,
            camera_config: CameraConfig::default(),
            exit_time: None,
            #[cfg(feature = "textures")]
            atlas_builder: None,
            #[cfg(feature = "textures")]
            atlas_size: 2048,
        }
    }

    /// Set the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the window size.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Enable or disable vertical sync.
    pub fn vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Configure the camera.
    pub fn camera(mut self, config: CameraConfig) -> Self {
        self.camera_config = config;
        self
    }

    /// Set an automatic exit time (useful for testing/benchmarking).
    pub fn exit_after(mut self, seconds: f32) -> Self {
        self.exit_time = Some(seconds);
        self
    }

    /// Register a texture atlas for automatic texture sprite rendering.
    ///
    /// When an atlas is registered, `draw.render_world(world)` will automatically
    /// render texture sprites without any additional code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// App::build(MyGame::new())
    ///     .atlas(
    ///         AtlasBuilder::new()
    ///             .add_image("player", "assets/player.png")?
    ///             .add_gradient("enemy", 32, 32, [255, 0, 0, 255], [128, 0, 0, 255])
    ///     )
    ///     .run()
    /// ```
    #[cfg(feature = "textures")]
    pub fn atlas(mut self, builder: AtlasBuilder) -> Self {
        self.atlas_builder = Some(builder);
        self
    }

    /// Set the texture atlas size (default: 2048).
    ///
    /// Larger sizes allow more textures but use more GPU memory.
    #[cfg(feature = "textures")]
    pub fn atlas_size(mut self, size: u32) -> Self {
        self.atlas_size = size;
        self
    }

    /// Build and run the application.
    pub fn run(self) -> Result<crate::AppStats, winit::error::EventLoopError> {
        let app = App {
            game: self.game,
            world: World::new(),
            window: None,
            gpu: GpuContext::new(),
            camera: Camera2D::new(
                self.camera_config.position,
                self.camera_config.scale,
                [self.width as f32, self.height as f32],
            )
            .with_zoom_speed(self.camera_config.zoom_speed),
            camera_controller: CameraController::new()
                .with_move_speed(self.camera_config.move_speed)
                .with_zoom_range(self.camera_config.min_zoom, self.camera_config.max_zoom),
            time: Time::new(),
            input: InputState::new(),
            #[cfg(feature = "gui")]
            gui: None,
            #[cfg(feature = "gui")]
            gui_renderer: None,
            circle_pipeline: None,
            line_pipeline: None,
            #[cfg(feature = "textures")]
            sprite_pipeline: None,
            #[cfg(feature = "textures")]
            texture_atlas: None,
            #[cfg(feature = "textures")]
            pending_atlas_builder: self.atlas_builder,
            #[cfg(feature = "textures")]
            atlas_size: self.atlas_size,
            exit_time: self.exit_time,
            config: AppConfig {
                title: self.title,
                width: self.width,
                height: self.height,
                vsync: self.vsync,
            },
        };

        app.run_internal()
    }
}

// =============================================================================
// APP CONFIG (internal)
// =============================================================================

struct AppConfig {
    title: String,
    width: u32,
    height: u32,
    vsync: bool,
}

// =============================================================================
// APP
// =============================================================================

/// The main application struct that manages the game loop.
///
/// Use [`App::build()`] or [`App::new()`] to create an application.
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
///
/// // Simple creation
/// App::new(MyGame::new()).run().unwrap();
///
/// // With configuration
/// App::build(MyGame::new())
///     .title("My Game")
///     .size(1280, 720)
///     .run()
///     .unwrap();
/// ```
pub struct App<T: GameLoop> {
    game: T,
    world: World,
    window: Option<Arc<Window>>,
    gpu: GpuContext,
    camera: Camera2D,
    camera_controller: CameraController,
    time: Time,
    input: InputState,
    #[cfg(feature = "gui")]
    gui: Option<Gui>,
    #[cfg(feature = "gui")]
    gui_renderer: Option<UiPipeline>,
    circle_pipeline: Option<CirclePipeline>,
    line_pipeline: Option<LinePipeline>,
    #[cfg(feature = "textures")]
    sprite_pipeline: Option<SpritePipeline>,
    #[cfg(feature = "textures")]
    texture_atlas: Option<TextureAtlas>,
    #[cfg(feature = "textures")]
    pending_atlas_builder: Option<AtlasBuilder>,
    #[cfg(feature = "textures")]
    atlas_size: u32,
    exit_time: Option<f32>,
    config: AppConfig,
}

impl<T: GameLoop> App<T> {
    /// Create a new App with the given game and default settings.
    ///
    /// For more configuration options, use [`App::build()`].
    pub fn new(game: T) -> Self {
        Self {
            game,
            world: World::new(),
            window: None,
            gpu: GpuContext::new(),
            camera: Camera2D::new([0.0, 0.0], 10.0, [1200.0, 800.0]),
            camera_controller: CameraController::new()
                .with_move_speed(250.0)
                .with_zoom_range(0.0001, 10.0),
            time: Time::new(),
            input: InputState::new(),
            #[cfg(feature = "gui")]
            gui: None,
            #[cfg(feature = "gui")]
            gui_renderer: None,
            circle_pipeline: None,
            line_pipeline: None,
            #[cfg(feature = "textures")]
            sprite_pipeline: None,
            #[cfg(feature = "textures")]
            texture_atlas: None,
            #[cfg(feature = "textures")]
            pending_atlas_builder: None,
            #[cfg(feature = "textures")]
            atlas_size: 2048,
            exit_time: None,
            config: AppConfig {
                title: "Graviplex".to_string(),
                width: 1200,
                height: 800,
                vsync: false,
            },
        }
    }

    /// Create an AppBuilder for configuring the application.
    ///
    /// # Example
    ///
    /// ```ignore
    /// App::build(MyGame::new())
    ///     .title("My Game")
    ///     .size(1280, 720)
    ///     .vsync(true)
    ///     .run()
    ///     .unwrap();
    /// ```
    pub fn build(game: T) -> AppBuilder<T> {
        AppBuilder::new(game)
    }

    /// Set an optional exit time in seconds (deprecated, use AppBuilder).
    #[deprecated(since = "0.3.0", note = "Use App::build().exit_after() instead")]
    pub fn with_exit_time(mut self, seconds: f32) -> Self {
        self.exit_time = Some(seconds);
        self
    }

    /// Run the application and return performance statistics.
    pub fn run(self) -> Result<crate::AppStats, winit::error::EventLoopError> {
        self.run_internal()
    }

    fn run_internal(self) -> Result<crate::AppStats, winit::error::EventLoopError> {
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
        self.game.handle_input(&mut self.world, &self.input, &self.camera);

        // Create resources for this frame
        let resources = Resources::new(&self.time, &self.input, &self.camera);

        // Update game with world and resources
        self.game.update(&mut self.world, &resources);

        let Ok(frame) = self.gpu.get_current_frame() else {
            return;
        };
        let view = frame.texture.create_view(&Default::default());

        // Let game render its content using DrawContext
        if let (Some(circle_pipeline), Some(line_pipeline)) =
            (&mut self.circle_pipeline, &mut self.line_pipeline)
        {
            // Get optional sprite resources
            #[cfg(feature = "textures")]
            let (sprite_pipeline, texture_atlas) = {
                let sp = self.sprite_pipeline.as_mut();
                let ta = self.texture_atlas.as_ref();
                (sp, ta)
            };

            let mut draw = DrawContext {
                gpu: &self.gpu,
                view: &view,
                camera: &self.camera,
                circle_pipeline,
                line_pipeline,
                #[cfg(feature = "textures")]
                sprite_pipeline,
                #[cfg(feature = "textures")]
                texture_atlas,
            };

            self.game.render(&self.world, &mut draw);

            // Automatically flush at the end of the frame
            draw.flush();
        }

        // Render GUI
        #[cfg(feature = "gui")]
        self.render_gui(&view);

        frame.present();
    }

    #[cfg(feature = "gui")]
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

        let mut encoder = self.gpu.raw_device().create_command_encoder(&Default::default());
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
        self.gpu.raw_queue().submit(std::iter::once(encoder.finish()));
    }
}

impl<T: GameLoop> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.config.width, self.config.height));

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Unable to create window"),
        );

        self.gpu.init_surface(window.clone(), self.config.vsync);

        let size = window.inner_size();
        self.camera.screen_size = [size.width as f32, size.height as f32];

        let format = self.gpu.surface_format();

        #[cfg(feature = "gui")]
        {
            let mut gui = Gui::new(event_loop);
            let initial_output = gui.run_empty(&window);
            let mut ui_pipeline = UiPipeline::new(self.gpu.raw_device(), self.gpu.raw_queue(), format);
            ui_pipeline.handle_textures(initial_output.textures_delta);
            self.gui = Some(gui);
            self.gui_renderer = Some(ui_pipeline);
        }

        // Initialize standard pipelines
        let circle_pipeline = CirclePipeline::new(self.gpu.raw_device(), format, &self.camera);
        let line_pipeline = LinePipeline::new(
            "Default",
            wgpu::include_wgsl!("shaders/line_shader.wgsl"),
            self.gpu.raw_device(),
            format,
            &self.camera,
        );

        self.circle_pipeline = Some(circle_pipeline);
        self.line_pipeline = Some(line_pipeline);

        // Initialize texture atlas and sprite pipeline if configured
        #[cfg(feature = "textures")]
        if let Some(atlas_builder) = self.pending_atlas_builder.take() {
            match self.gpu.build_atlas(atlas_builder, self.atlas_size) {
                Ok(atlas) => {
                    let sprite_pipeline = self.gpu.create_sprite_pipeline(&atlas);
                    self.texture_atlas = Some(atlas);
                    self.sprite_pipeline = Some(sprite_pipeline);
                }
                Err(e) => {
                    log::error!("Failed to build texture atlas: {e}");
                }
            }
        }

        // Initialize game with world and GPU context
        self.game.init(&mut self.world, &self.gpu);

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let GUI handle event first
        #[cfg(feature = "gui")]
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
