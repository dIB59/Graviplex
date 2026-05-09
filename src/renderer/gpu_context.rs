//! GPU context management for wgpu resources.

use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

#[cfg(not(target_arch = "wasm32"))]
use pollster::FutureExt;

/// Graphics device wrapper managing wgpu resources.
pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: Option<Surface<'static>>,
    pub config: Option<SurfaceConfiguration>,
}

impl GpuContext {
    /// Create a new GPU context (device and queue only, no surface).
    ///
    /// On native this is fully synchronous via pollster. On WASM, blocking the
    /// main thread is forbidden, so the async constructor `new_async` is the
    /// only valid one — `new` will panic at runtime under wasm32.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self::new_async().block_on()
    }

    pub async fn new_async() -> Self {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Unable to create adapter");

        let (device, queue) = Self::request_device(&adapter).await;

        Self {
            device,
            queue,
            surface: None,
            config: None,
        }
    }

    /// Initialize the surface for window rendering. Synchronous on native.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn init_surface(&mut self, window: Arc<Window>, vsync: bool) {
        self.init_surface_async(window, vsync).block_on()
    }

    pub async fn init_surface_async(&mut self, window: Arc<Window>, vsync: bool) {
        let instance = Instance::new(&InstanceDescriptor::default());

        let surface = instance
            .create_surface(window.clone())
            .expect("Unable to create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Unable to create adapter");

        let (device, queue) = Self::request_device(&adapter).await;

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

    async fn request_device(adapter: &Adapter) -> (Device, Queue) {
        // SHADER_F16 and CONSERVATIVE_RASTERIZATION are Vulkan/Metal-only and
        // unavailable through the WebGL backend. Request an empty feature set
        // on WASM and fall back gracefully.
        #[cfg(target_arch = "wasm32")]
        let features = Features::empty();

        #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
        let features = Features::SHADER_F16;

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos")))]
        let features = Features::SHADER_F16 | Features::CONSERVATIVE_RASTERIZATION;

        // WebGL imposes tighter limits than the native default. Request the
        // downlevel-webgl2 baseline on WASM so the device-creation request
        // doesn't fail with "limits exceeded".
        #[cfg(target_arch = "wasm32")]
        let required_limits = Limits::downlevel_webgl2_defaults();
        #[cfg(not(target_arch = "wasm32"))]
        let required_limits = Limits::default();

        adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_limits,
                required_features: features,
                ..Default::default()
            })
            .await
            .expect("Unable to create device")
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for GpuContext {
    fn default() -> Self {
        Self::new()
    }
}
