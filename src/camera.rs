use bytemuck::{Pod, Zeroable};
use wgpu::{Buffer, Queue};
use winit::event::MouseScrollDelta;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct View {
    pub position: [f32; 2], // Changed from Vec2 to [f32; 2] for GPU compatibility
    pub scale: f32,
    pub zoom_speed: f32, // Padding for proper GPU alignment (16-byte boundary)
    pub screen_size: [f32; 2], // Changed from x,y u16 to screen_size [f32; 2] for shader
}

unsafe impl Zeroable for View {}
unsafe impl Pod for View {}

impl View {
    pub fn update_from_input(
        &mut self,
        delta_time: f32,
        pressed_keys: &std::collections::HashSet<winit::keyboard::KeyCode>,
        queue: &wgpu::Queue,
        view_buffer: &wgpu::Buffer,
    ) {
        let base_speed = 2.0;
        let movement_speed = base_speed / self.scale * delta_time;

        let mut movement = [0.0f32; 2];
        use winit::keyboard::KeyCode::*;
        if pressed_keys.contains(&KeyW) {
            movement[1] += movement_speed;
        }
        if pressed_keys.contains(&KeyS) {
            movement[1] -= movement_speed;
        }
        if pressed_keys.contains(&KeyA) {
            movement[0] -= movement_speed;
        }
        if pressed_keys.contains(&KeyD) {
            movement[0] += movement_speed;
        }

        if movement != [0.0, 0.0] {
            self.position[0] += movement[0];
            self.position[1] += movement[1];

            queue.write_buffer(view_buffer, 0, bytemuck::cast_slice(&[*self]));
        }
    }

    pub fn apply_zoom(&mut self, zoom_factor: f32, queue: &Queue, view_buffer: &Buffer) {
        self.scale *= zoom_factor;
        self.scale = self.scale.clamp(10.0, 5000.0);

        queue.write_buffer(view_buffer, 0, bytemuck::cast_slice(&[*self]));
    }

    fn handle_scroll(
        &mut self,
        delta: MouseScrollDelta,
        queue: &Queue,
        view_buffer: &wgpu::Buffer,
    ) {
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

        self.scale *= zoom_factor;

        // Clamp zoom to reasonable bounds (higher max for Retina displays)
        self.scale = self.scale.clamp(10.0, 5000.0);

        // Update the buffer
        queue.write_buffer(view_buffer, 0, bytemuck::cast_slice(&[*self]));
    }
}
