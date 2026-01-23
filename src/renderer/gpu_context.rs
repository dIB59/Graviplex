//! GPU context management for wgpu resources.

use pollster::FutureExt;
use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

/// Graphics device wrapper managing wgpu resources.
///
/// This is the core GPU abstraction that provides access to the device and queue
/// for creating buffers, pipelines, and other GPU resources.
///
/// # Example
///
/// ```ignore
/// fn init(&mut self, gfx: &Graphics) {
///     let buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
///         label: Some("My Buffer"),
///         contents: bytemuck::cast_slice(&data),
///         usage: BufferUsages::VERTEX,
///     });
/// }
/// ```
pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: Option<Surface<'static>>,
    pub config: Option<SurfaceConfiguration>,
}

impl GpuContext {
    /// Create a new GPU context (device and queue only, no surface).
    pub fn new() -> Self {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .expect("Unable to create adapter");

        let (device, queue) = Self::request_device(&adapter);

        Self {
            device,
            queue,
            surface: None,
            config: None,
        }
    }

    /// Initialize the surface for window rendering.
    ///
    /// This should be called once when the window is created.
    /// The vsync parameter controls whether vertical sync is enabled.
    pub fn init_surface(&mut self, window: Arc<Window>, vsync: bool) {
        let instance = Instance::new(&InstanceDescriptor::default());

        let surface = instance
            .create_surface(window.clone())
            .expect("Unable to create surface");

        // Request a new adapter with surface compatibility
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Unable to create adapter");

        let (device, queue) = Self::request_device(&adapter);

        let size = window.inner_size();
        let format = surface.get_capabilities(&adapter).formats[0];

        let present_mode = if vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            width: size.width,
            height: size.height,
            present_mode,
            format,
            desired_maximum_frame_latency: Default::default(),
            alpha_mode: Default::default(),
            view_formats: Default::default(),
        };

        surface.configure(&device, &config);

        self.device = device;
        self.queue = queue;
        self.surface = Some(surface);
        self.config = Some(config);
    }

    /// Returns the surface texture format.
    ///
    /// This is useful for creating pipelines that need to match the surface format.
    /// Panics if the surface hasn't been initialized yet.
    pub fn surface_format(&self) -> TextureFormat {
        self.config
            .as_ref()
            .expect("Surface not initialized - call init_surface first")
            .format
    }

    /// Resize the surface when the window size changes.
    pub fn resize(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
    }

    /// Get the current frame for rendering.
    pub fn get_current_frame(&self) -> Result<SurfaceTexture, SurfaceError> {
        self.surface
            .as_ref()
            .expect("Surface not initialized")
            .get_current_texture()
    }

    fn request_device(adapter: &Adapter) -> (Device, Queue) {
        #[cfg(target_os = "macos")]
        let features = Features::SHADER_F16;

        #[cfg(not(target_os = "macos"))]
        let features = Features::SHADER_F16 | Features::CONSERVATIVE_RASTERIZATION;

        adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_limits: Limits::default(),
                required_features: features,
                ..Default::default()
            })
            .block_on()
            .expect("Unable to create device")
    }
}

// =============================================================================
// Texture Atlas Integration (feature-gated)
// =============================================================================

#[cfg(feature = "textures")]
impl GpuContext {
    /// Build a texture atlas from an AtlasBuilder.
    ///
    /// This is the preferred way to create atlases as it doesn't require
    /// direct access to wgpu device/queue.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let atlas = gfx.build_atlas(
    ///     AtlasBuilder::new()
    ///         .add_image("player", "assets/player.png")?
    ///         .add_circle("bullet", 16, [255, 255, 0, 255], None),
    ///     2048
    /// )?;
    /// ```
    pub fn build_atlas(
        &self,
        builder: super::texture_atlas::AtlasBuilder,
        max_size: u32,
    ) -> Result<super::texture_atlas::TextureAtlas, super::texture_atlas::AtlasError> {
        builder.build(&self.device, &self.queue, max_size)
    }

    /// Create a sprite pipeline for rendering textured sprites.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let atlas = gfx.build_atlas(builder, 2048)?;
    /// let pipeline = gfx.create_sprite_pipeline(&atlas);
    /// ```
    pub fn create_sprite_pipeline(
        &self,
        atlas: &super::texture_atlas::TextureAtlas,
    ) -> super::sprite_pipeline::SpritePipeline {
        let camera = super::Camera2D::default();
        super::sprite_pipeline::SpritePipeline::new(
            &self.device,
            self.surface_format(),
            &camera,
            atlas,
        )
    }

    /// Create a sprite pipeline with a custom camera.
    pub fn create_sprite_pipeline_with_camera(
        &self,
        atlas: &super::texture_atlas::TextureAtlas,
        camera: &super::Camera2D,
    ) -> super::sprite_pipeline::SpritePipeline {
        super::sprite_pipeline::SpritePipeline::new(
            &self.device,
            self.surface_format(),
            camera,
            atlas,
        )
    }
}

impl Default for GpuContext {
    fn default() -> Self {
        Self::new()
    }
}
