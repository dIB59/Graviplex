use pollster::FutureExt;
use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: Option<Surface<'static>>,
    pub config: Option<SurfaceConfiguration>,
}

impl GpuContext {
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

    pub fn init_surface(&mut self, window: Arc<Window>) {
        let instance = Instance::new(&InstanceDescriptor::default());
        
        let surface = instance
            .create_surface(window.clone())
            .expect("Unable to create surface");

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Unable to create adapter");

        let (device, queue) = Self::request_device(&adapter);
        
        let size = window.inner_size();
        let format = surface.get_capabilities(&adapter).formats[0];

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
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

    pub fn resize(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
    }

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
            .request_device(
                &DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    required_limits: Limits::default(),
                    required_features: features,
                    ..Default::default()
                },
            )
            .block_on()
            .expect("Unable to create device")
    }
}
