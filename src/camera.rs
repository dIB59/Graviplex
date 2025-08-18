use bytemuck::{Pod, Zeroable};
use std::collections::HashSet;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

/// Camera component that handles 2D view transformations
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Camera2D {
    pub position: [f32; 2],
    pub scale: f32,
    pub zoom_speed: f32,
    pub screen_size: [f32; 2],
}

unsafe impl Zeroable for Camera2D {}
unsafe impl Pod for Camera2D {}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            scale: 400.0,
            zoom_speed: 1.1,
            screen_size: [800.0, 800.0],
        }
    }
}

impl Camera2D {
    pub fn new(position: [f32; 2], scale: f32) -> Self {
        Self {
            position,
            scale,
            zoom_speed: 1.1,
            screen_size: [800.0, 800.0],
        }
    }

    pub fn with_zoom_speed(mut self, zoom_speed: f32) -> Self {
        self.zoom_speed = zoom_speed;
        self
    }

    pub fn with_screen_size(mut self, screen_size: [f32; 2]) -> Self {
        self.screen_size = screen_size;
        self
    }
}

/// Camera GPU resources
pub struct CameraGpuData {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl CameraGpuData {
    pub fn new(device: &Device, camera: &Camera2D) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[*camera]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update(&self, queue: &Queue, camera: &Camera2D) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*camera]));
    }
}

/// Camera controller for handling input
pub struct CameraController {
    pub move_speed: f32,
    pub zoom_sensitivity: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 200.0,
            zoom_sensitivity: 0.01,
            min_zoom: 10.0,
            max_zoom: 5000.0,
        }
    }
}

impl CameraController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_move_speed(mut self, speed: f32) -> Self {
        self.move_speed = speed;
        self
    }

    pub fn with_zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }

    /// Update camera position based on pressed keys and delta time
    pub fn update_movement(
        &self,
        camera: &mut Camera2D,
        delta_time: f32,
        pressed_keys: &HashSet<KeyCode>,
    ) -> bool {
        let movement_speed = self.move_speed / camera.scale * delta_time;
        let mut movement = [0.0f32; 2];
        let mut moved = false;

        if pressed_keys.contains(&KeyCode::KeyW) || pressed_keys.contains(&KeyCode::ArrowUp) {
            movement[1] += movement_speed;
            moved = true;
        }
        if pressed_keys.contains(&KeyCode::KeyS) || pressed_keys.contains(&KeyCode::ArrowDown) {
            movement[1] -= movement_speed;
            moved = true;
        }
        if pressed_keys.contains(&KeyCode::KeyA) || pressed_keys.contains(&KeyCode::ArrowLeft) {
            movement[0] -= movement_speed;
            moved = true;
        }
        if pressed_keys.contains(&KeyCode::KeyD) || pressed_keys.contains(&KeyCode::ArrowRight) {
            movement[0] += movement_speed;
            moved = true;
        }

        if moved {
            camera.position[0] += movement[0];
            camera.position[1] += movement[1];
        }

        moved
    }

    /// Handle mouse scroll events for zoom
    pub fn handle_scroll(&self, camera: &mut Camera2D, delta: MouseScrollDelta) -> bool {
        let zoom_factor = match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    camera.zoom_speed
                } else {
                    1.0 / camera.zoom_speed
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                let y = pos.y as f32;
                if y > 0.0 {
                    1.0 + (y * self.zoom_sensitivity)
                } else {
                    1.0 / (1.0 + (-y * self.zoom_sensitivity))
                }
            }
        };

        let old_scale = camera.scale;
        camera.scale *= zoom_factor;
        camera.scale = camera.scale.clamp(self.min_zoom, self.max_zoom);

        old_scale != camera.scale
    }
}

/// Camera plugin that manages camera systems
pub struct CameraPlugin {
    pub camera: Camera2D,
    pub controller: CameraController,
    pub gpu_data: CameraGpuData,
}

impl CameraPlugin {
    /// Creates a new [`CameraPlugin`].
    pub fn new(device: &Device) -> Self {
        let camera = Camera2D::default();
        Self {
            gpu_data: CameraGpuData::new(device, &camera),
            camera,
            controller: CameraController::default(),
        }
    }

    pub fn with_camera(mut self, camera: Camera2D) -> Self {
        self.camera = camera;
        self
    }

    pub fn with_controller(mut self, controller: CameraController) -> Self {
        self.controller = controller;
        self
    }

    /// Update camera based on input and time
    pub fn update(&mut self, delta_time: f32, pressed_keys: &HashSet<KeyCode>, queue: &Queue) {
        let moved = self
            .controller
            .update_movement(&mut self.camera, delta_time, pressed_keys);

        if moved {
            self.gpu_data.update(queue, &self.camera);
        }
    }

    /// Handle scroll input
    pub fn handle_scroll(&mut self, delta: MouseScrollDelta, queue: &Queue) {
        let zoomed = self.controller.handle_scroll(&mut self.camera, delta);

        if zoomed {
            self.gpu_data.update(queue, &self.camera);
        }
    }

    /// Handle window resize
    pub fn handle_resize(&mut self, new_size: [f32; 2], queue: &Queue) {
        self.camera.screen_size = new_size;
        self.gpu_data.update(queue, &self.camera);
    }

    /// Get the camera's bind group layout (for pipeline creation)
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.gpu_data.bind_group_layout
    }

    /// Get the camera's bind group (for render pass)
    pub fn bind_group(&self) -> &BindGroup {
        &self.gpu_data.bind_group
    }

    /// Get the camera data
    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    /// Get mutable camera data
    pub fn camera_mut(&mut self) -> &mut Camera2D {
        &mut self.camera
    }

    /// Force update GPU buffer (useful for manual camera changes)
    pub fn force_update_gpu(&self, queue: &Queue) {
        self.gpu_data.update(queue, &self.camera);
    }
}
