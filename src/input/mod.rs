use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    mouse_pos: [f32; 2],
    mouse_buttons: HashSet<winit::event::MouseButton>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_pos: [0.0, 0.0],
            mouse_buttons: HashSet::new(),
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_keyboard_event(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(keycode);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&keycode);
                }
            }
        }
    }

    pub fn handle_mouse_input(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        match state {
            winit::event::ElementState::Pressed => {
                self.mouse_buttons.insert(button);
            }
            winit::event::ElementState::Released => {
                self.mouse_buttons.remove(&button);
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_pos = [position.x as f32, position.y as f32];
    }

    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }

    pub fn mouse_pos(&self) -> [f32; 2] {
        self.mouse_pos
    }

    pub fn is_mouse_down(&self, button: winit::event::MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }
}
