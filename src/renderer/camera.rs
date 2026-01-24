use bytemuck::{Pod, Zeroable};
use std::collections::HashSet;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

/// Camera follow behavior configuration.
///
/// This struct handles smooth camera following with configurable smoothing
/// and maximum offset constraints. Use it to make the camera follow a target
/// (like a player) with a natural "lag behind" effect that conveys speed.
///
/// # Example
/// ```ignore
/// let mut camera_follow = CameraFollow::new()
///     .with_smoothing(5.0)      // Lower = more lag behind player
///     .with_max_offset(200.0);  // Max pixels camera can lag behind
///
/// // In update loop:
/// camera_follow.set_target([player.x, player.y]);
/// camera_follow.update(&mut camera, delta_time);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CameraFollow {
    /// The target position the camera should follow
    pub target: Option<[f32; 2]>,
    /// Smoothing factor (lower = more lag, higher = snappier)
    /// - 2.0-4.0 = lots of lag, shows speed well
    /// - 5.0-8.0 = balanced
    /// - 10.0+ = snappy, minimal lag
    pub smoothing: f32,
    /// Maximum distance the camera can lag behind the target (in world units)
    /// The camera will be pulled forward if it falls too far behind.
    /// Set to 0.0 or negative for no limit.
    pub max_offset: f32,
    /// Deadzone - camera won't move if target is within this distance
    pub deadzone: f32,
}

impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            target: None,
            smoothing: 8.0,
            max_offset: 0.0, // No limit by default
            deadzone: 0.0,
        }
    }
}

impl CameraFollow {
    /// Create a new CameraFollow with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the smoothing factor
    ///
    /// Lower values = more lag (camera falls behind player more)
    /// - `2.0-4.0` = lots of lag, great for showing speed
    /// - `5.0-8.0` = balanced
    /// - `10.0+` = snappy, minimal lag
    /// - `100.0+` = nearly instant snap
    pub fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// Set the maximum lag distance
    ///
    /// The camera will be pulled forward if it falls more than this
    /// distance behind the target. Set to 0.0 for no limit.
    pub fn with_max_offset(mut self, max_offset: f32) -> Self {
        self.max_offset = max_offset;
        self
    }

    /// Set the deadzone radius
    ///
    /// The camera won't move if the target is within this distance
    /// of the current camera position. Good for reducing jitter.
    pub fn with_deadzone(mut self, deadzone: f32) -> Self {
        self.deadzone = deadzone;
        self
    }

    /// Set the target position for the camera to follow
    pub fn set_target(&mut self, target: [f32; 2]) {
        self.target = Some(target);
    }

    /// Clear the target (camera will stop following)
    pub fn clear_target(&mut self) {
        self.target = None;
    }

    /// Update the camera position based on the current target
    ///
    /// Call this once per frame with the delta time.
    /// Returns `true` if the camera position changed.
    pub fn update(&self, camera: &mut Camera2D, delta_time: f32) -> bool {
        let Some(target) = self.target else {
            return false;
        };

        let dx = target[0] - camera.position[0];
        let dy = target[1] - camera.position[1];
        let distance = (dx * dx + dy * dy).sqrt();

        // Don't move if within deadzone
        if distance <= self.deadzone {
            return false;
        }

        // Calculate interpolation factor using exponential smoothing
        let t = if self.smoothing <= 1.0 {
            1.0 // Instant snap
        } else {
            // Frame-rate independent exponential decay
            1.0 - (-self.smoothing * delta_time).exp()
        };

        // Calculate new position
        let mut new_x = camera.position[0] + dx * t;
        let mut new_y = camera.position[1] + dy * t;

        // Apply max offset constraint
        if self.max_offset > 0.0 {
            let new_dx = target[0] - new_x;
            let new_dy = target[1] - new_y;
            let new_distance = (new_dx * new_dx + new_dy * new_dy).sqrt();

            if new_distance > self.max_offset {
                // Clamp to max offset distance from target
                let scale = self.max_offset / new_distance;
                new_x = target[0] - new_dx * scale;
                new_y = target[1] - new_dy * scale;
            }
        }

        let changed = (new_x - camera.position[0]).abs() > 0.001
            || (new_y - camera.position[1]).abs() > 0.001;

        camera.position[0] = new_x;
        camera.position[1] = new_y;

        changed
    }

    /// Get the current offset from target (useful for UI effects)
    pub fn current_offset(&self, camera: &Camera2D) -> [f32; 2] {
        match self.target {
            Some(target) => [
                camera.position[0] - target[0],
                camera.position[1] - target[1],
            ],
            None => [0.0, 0.0],
        }
    }

    /// Get the current offset distance from target
    pub fn current_offset_distance(&self, camera: &Camera2D) -> f32 {
        let offset = self.current_offset(camera);
        (offset[0] * offset[0] + offset[1] * offset[1]).sqrt()
    }
}

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
    pub fn new(position: [f32; 2], scale: f32, screen_size: [f32; 2]) -> Self {
        Self {
            position,
            scale,
            zoom_speed: 1.1,
            screen_size,
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

    pub fn screen_to_world(&self, screen_pos: [f32; 2]) -> [f32; 2] {
        let x = (screen_pos[0] - self.screen_size[0] / 2.0) / self.scale + self.position[0];
        let y = (self.screen_size[1] / 2.0 - screen_pos[1]) / self.scale + self.position[1];
        [x, y]
    }

    /// Returns the visible world bounds as (min, max) corners.
    ///
    /// Useful for culling objects outside the camera view.
    pub fn visible_bounds(&self) -> (crate::core::math::Vec2, crate::core::math::Vec2) {
        let half_width = self.screen_size[0] / (2.0 * self.scale);
        let half_height = self.screen_size[1] / (2.0 * self.scale);
        
        let min = crate::core::math::Vec2::new(
            self.position[0] - half_width,
            self.position[1] - half_height,
        );
        let max = crate::core::math::Vec2::new(
            self.position[0] + half_width,
            self.position[1] + half_height,
        );
        
        (min, max)
    }

    /// Returns the visible world width in world units.
    pub fn visible_width(&self) -> f32 {
        self.screen_size[0] / self.scale
    }

    /// Returns the visible world height in world units.
    pub fn visible_height(&self) -> f32 {
        self.screen_size[1] / self.scale
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
            min_zoom: 1.0,
            max_zoom: 15000.0,
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
