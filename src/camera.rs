use bytemuck::{Pod, Zeroable};
use std::collections::HashSet;
use wgpu::{Buffer, Queue};
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct View {
    pub position: [f32; 2],
    pub scale: f32,
    pub zoom_speed: f32,
    pub screen_size: [f32; 2],
}

unsafe impl Zeroable for View {}
unsafe impl Pod for View {}

impl View {
    /// Update camera position based on pressed keys and delta time.
    pub fn update_from_input(
        &mut self,
        delta_time: f32,
        pressed_keys: &HashSet<KeyCode>,
        queue: &Queue,
        view_buffer: &Buffer,
    ) {
        let base_speed = 200.0; // world units per second
        let movement_speed = base_speed / self.scale * delta_time;

        let mut movement = [0.0f32; 2];
        if pressed_keys.contains(&KeyCode::KeyW) {
            movement[1] += movement_speed;
        }
        if pressed_keys.contains(&KeyCode::KeyS) {
            movement[1] -= movement_speed;
        }
        if pressed_keys.contains(&KeyCode::KeyA) {
            movement[0] -= movement_speed;
        }
        if pressed_keys.contains(&KeyCode::KeyD) {
            movement[0] += movement_speed;
        }

        if movement != [0.0, 0.0] {
            self.position[0] += movement[0];
            self.position[1] += movement[1];
            queue.write_buffer(view_buffer, 0, bytemuck::cast_slice(&[*self]));
        }
    }

    /// Apply a zoom multiplier and update GPU buffer.
    pub fn apply_zoom(&mut self, zoom_factor: f32, queue: &Queue, view_buffer: &Buffer) {
        self.scale *= zoom_factor;
        self.scale = self.scale.clamp(10.0, 5000.0);
        queue.write_buffer(view_buffer, 0, bytemuck::cast_slice(&[*self]));
    }

    /// Handle mouse scroll events for zoom.
    pub fn handle_scroll(&mut self, delta: MouseScrollDelta, queue: &Queue, view_buffer: &Buffer) {
        let zoom_factor = match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    self.zoom_speed
                } else {
                    1.0 / self.zoom_speed
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                let y = pos.y as f32;
                if y > 0.0 {
                    1.0 + (y * 0.01)
                } else {
                    1.0 / (1.0 + (-y * 0.01))
                }
            }
        };
        self.apply_zoom(zoom_factor, queue, view_buffer);
    }
}
