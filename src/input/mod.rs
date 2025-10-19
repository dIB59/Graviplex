use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
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

    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }
}
