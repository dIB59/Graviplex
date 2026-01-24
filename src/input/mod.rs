use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Tracks the current state of keyboard and mouse input.
///
/// Input state is updated automatically by the engine each frame.
/// Access it via `Resources` in your update method.
///
/// # Example
///
/// ```ignore
/// fn update(&mut self, world: &mut World, res: &Resources) {
///     // Check if a key is currently held down
///     if res.input.is_key_down(KeyCode::Space) {
///         self.player_jump();
///     }
///
///     // Check if a key was just pressed this frame
///     if res.input.is_key_just_pressed(KeyCode::Escape) {
///         self.toggle_pause();
///     }
///
///     // Get mouse position
///     let mouse = res.input.mouse_pos();
/// }
/// ```
pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    just_pressed_keys: HashSet<KeyCode>,
    just_released_keys: HashSet<KeyCode>,
    mouse_pos: [f32; 2],
    mouse_buttons: HashSet<winit::event::MouseButton>,
    just_pressed_buttons: HashSet<winit::event::MouseButton>,
    just_released_buttons: HashSet<winit::event::MouseButton>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            mouse_pos: [0.0, 0.0],
            mouse_buttons: HashSet::new(),
            just_pressed_buttons: HashSet::new(),
            just_released_buttons: HashSet::new(),
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call at the start of each frame to clear "just pressed/released" state.
    pub(crate) fn begin_frame(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_buttons.clear();
        self.just_released_buttons.clear();
    }

    pub fn handle_keyboard_event(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if !self.pressed_keys.contains(&keycode) {
                        self.just_pressed_keys.insert(keycode);
                    }
                    self.pressed_keys.insert(keycode);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&keycode);
                    self.just_released_keys.insert(keycode);
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
                if !self.mouse_buttons.contains(&button) {
                    self.just_pressed_buttons.insert(button);
                }
                self.mouse_buttons.insert(button);
            }
            winit::event::ElementState::Released => {
                self.mouse_buttons.remove(&button);
                self.just_released_buttons.insert(button);
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_pos = [position.x as f32, position.y as f32];
    }

    // =========================================================================
    // Keyboard queries
    // =========================================================================

    /// Returns `true` if the key is currently held down.
    #[inline]
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Returns `true` if the key was pressed this frame (edge detection).
    #[inline]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// Returns `true` if the key was released this frame (edge detection).
    #[inline]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }

    /// Returns the set of all currently pressed keys.
    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }

    // =========================================================================
    // Mouse queries
    // =========================================================================

    /// Returns the current mouse position in screen coordinates.
    #[inline]
    pub fn mouse_pos(&self) -> [f32; 2] {
        self.mouse_pos
    }

    /// Returns `true` if the mouse button is currently held down.
    #[inline]
    pub fn is_mouse_down(&self, button: winit::event::MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Returns `true` if the mouse button was pressed this frame.
    #[inline]
    pub fn is_mouse_just_pressed(&self, button: winit::event::MouseButton) -> bool {
        self.just_pressed_buttons.contains(&button)
    }

    /// Returns `true` if the mouse button was released this frame.
    #[inline]
    pub fn is_mouse_just_released(&self, button: winit::event::MouseButton) -> bool {
        self.just_released_buttons.contains(&button)
    }
}
