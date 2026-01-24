//! Scene and state management for game flow control.
//!
//! This module provides a scene/state stack for managing game flow:
//! menus, gameplay, pause screens, dialogue, cutscenes, etc.
//!
//! # Integration with App
//!
//! `SceneManager` is an opt-in feature for games that need scene transitions.
//! For simple games, you can use `GameLoop` directly without scenes.
//!
//! To use scenes, create a wrapper game struct that owns the SceneManager:
//!
//! ```ignore
//! use graviplex::prelude::*;
//! use graviplex::scene::{Scene, SceneManager, Transition};
//!
//! struct SceneGame {
//!     scenes: SceneManager,
//! }
//!
//! impl SceneGame {
//!     fn new() -> Self {
//!         Self {
//!             scenes: SceneManager::with_scene(Box::new(MainMenuScene)),
//!         }
//!     }
//! }
//!
//! impl GameLoop for SceneGame {
//!     fn init(&mut self, world: &mut World, gfx: &Graphics) {
//!         self.scenes.init(world, gfx);
//!     }
//!
//!     fn update(&mut self, world: &mut World, res: &Resources) {
//!         let transition = self.scenes.update(world, res);
//!         self.scenes.apply_transition(transition, world, gfx);
//!     }
//!
//!     fn render(&mut self, world: &World, draw: &mut DrawContext) {
//!         self.scenes.render(world, draw);
//!     }
//! }
//! ```
//!
//! # Example Scene
//!
//! ```ignore
//! use graviplex::prelude::*;
//!
//! struct MenuScene;
//! struct GameplayScene;
//!
//! impl Scene for MenuScene {
//!     fn update(&mut self, world: &mut World, res: &Resources) -> Transition {
//!         if res.input.is_key_just_pressed(KeyCode::Enter) {
//!             return Transition::Push(Box::new(GameplayScene));
//!         }
//!         Transition::None
//!     }
//!
//!     fn render(&mut self, world: &World, draw: &mut DrawContext) {
//!         draw.text("Press ENTER to start", Vec2::ZERO, Color::WHITE);
//!     }
//! }
//! ```

use crate::ecs::{Resources, World};
use crate::renderer::DrawContext;

#[cfg(feature = "gui")]
use crate::gui::EguiRenderer;

// =============================================================================
// TRANSITION
// =============================================================================

/// Transition commands returned by scene update methods.
pub enum Transition {
    /// Stay on current scene.
    None,
    /// Push a new scene on top of the stack.
    Push(Box<dyn Scene>),
    /// Pop the current scene off the stack.
    Pop,
    /// Pop current and push a new scene.
    Switch(Box<dyn Scene>),
    /// Clear all scenes and push a new one.
    Replace(Box<dyn Scene>),
    /// Exit the game.
    Quit,
}

// =============================================================================
// SCENE TRAIT
// =============================================================================

/// A game scene/state that can update and render.
///
/// Scenes are managed in a stack by [`SceneManager`]. They have lifecycle
/// hooks for entering, exiting, pausing (when another scene is pushed on top),
/// and resuming.
///
/// # Example
///
/// ```ignore
/// struct GameplayScene {
///     player: Entity,
/// }
///
/// impl Scene for GameplayScene {
///     fn on_enter(&mut self, world: &mut World, gfx: &Graphics) {
///         self.player = world.spawn((
///             Transform::from_position(Vec2::ZERO),
///             Sprite::circle(32.0, Color::BLUE),
///             Visible,
///         ));
///     }
///
///     fn update(&mut self, world: &mut World, res: &Resources) -> Transition {
///         // Game logic here
///         if res.input.is_key_just_pressed(KeyCode::Escape) {
///             return Transition::Push(Box::new(PauseScene));
///         }
///         Transition::None
///     }
///
///     fn render(&mut self, world: &World, draw: &mut DrawContext) {
///         draw.render_world(world);
///     }
/// }
/// ```
pub trait Scene: Send {
    /// Called when this scene becomes the active scene.
    ///
    /// Use this to initialize scene-specific entities and resources.
    fn on_enter(&mut self, _world: &mut World, _gfx: &crate::renderer::Graphics) {}

    /// Called when this scene is removed from the stack.
    ///
    /// Use this to clean up scene-specific entities.
    fn on_exit(&mut self, _world: &mut World) {}

    /// Called when another scene is pushed on top of this one.
    ///
    /// The scene remains in the stack but is not active.
    fn on_pause(&mut self, _world: &mut World) {}

    /// Called when this scene becomes active again after a pop.
    fn on_resume(&mut self, _world: &mut World) {}

    /// Update game logic. Return a [`Transition`] to change scenes.
    fn update(&mut self, world: &mut World, res: &Resources) -> Transition;

    /// Render the scene.
    fn render(&mut self, world: &World, draw: &mut DrawContext);

    /// Render GUI elements (requires `gui` feature).
    #[cfg(feature = "gui")]
    fn gui(&mut self, _world: &mut World, _gui: &mut EguiRenderer) {}

    /// Whether scenes below this one should still be rendered.
    ///
    /// Useful for transparent overlays like pause menus.
    fn is_transparent(&self) -> bool {
        false
    }

    /// Whether scenes below this one should still be updated.
    ///
    /// Usually false - only the top scene updates.
    fn updates_below(&self) -> bool {
        false
    }
}

// =============================================================================
// SCENE MANAGER
// =============================================================================

/// Manages a stack of scenes for game state transitions.
///
/// The scene manager maintains a stack of scenes. The topmost scene
/// is the "active" scene that receives update and render calls.
/// Scenes below can optionally still render if they're transparent.
///
/// # Example
///
/// ```ignore
/// let mut scenes = SceneManager::new();
/// scenes.push(Box::new(MainMenuScene));
///
/// // In game loop:
/// let transition = scenes.update(world, res);
/// scenes.apply_transition(transition, world, gfx);
/// scenes.render(world, draw);
/// ```
pub struct SceneManager {
    stack: Vec<Box<dyn Scene>>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneManager {
    /// Create a new empty scene manager.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Create a scene manager with an initial scene.
    pub fn with_scene(scene: Box<dyn Scene>) -> Self {
        Self { stack: vec![scene] }
    }

    /// Initialize the scene manager, calling on_enter for the top scene.
    pub fn init(&mut self, world: &mut World, gfx: &crate::renderer::Graphics) {
        if let Some(scene) = self.stack.last_mut() {
            scene.on_enter(world, gfx);
        }
    }

    /// Push a new scene onto the stack.
    pub fn push(&mut self, scene: Box<dyn Scene>, world: &mut World, gfx: &crate::renderer::Graphics) {
        if let Some(current) = self.stack.last_mut() {
            current.on_pause(world);
        }
        self.stack.push(scene);
        if let Some(new) = self.stack.last_mut() {
            new.on_enter(world, gfx);
        }
    }

    /// Pop the current scene off the stack.
    pub fn pop(&mut self, world: &mut World) -> Option<Box<dyn Scene>> {
        let scene = self.stack.pop();
        if let Some(mut s) = scene {
            s.on_exit(world);
            if let Some(resumed) = self.stack.last_mut() {
                resumed.on_resume(world);
            }
            return Some(s);
        }
        None
    }

    /// Switch to a new scene (pop current, push new).
    pub fn switch(&mut self, scene: Box<dyn Scene>, world: &mut World, gfx: &crate::renderer::Graphics) {
        self.pop(world);
        self.push(scene, world, gfx);
    }

    /// Replace all scenes with a new one.
    pub fn replace(&mut self, scene: Box<dyn Scene>, world: &mut World, gfx: &crate::renderer::Graphics) {
        while !self.stack.is_empty() {
            self.pop(world);
        }
        self.push(scene, world, gfx);
    }

    /// Clear all scenes.
    pub fn clear(&mut self, world: &mut World) {
        while !self.stack.is_empty() {
            self.pop(world);
        }
    }

    /// Update the active scene(s).
    pub fn update(&mut self, world: &mut World, res: &Resources) -> Transition {
        if let Some(scene) = self.stack.last_mut() {
            return scene.update(world, res);
        }
        Transition::None
    }

    /// Render scene(s), respecting transparency.
    pub fn render(&mut self, world: &World, draw: &mut DrawContext) {
        // Find the first non-transparent scene to start rendering from
        let start_idx = self
            .stack
            .iter()
            .rposition(|s| !s.is_transparent())
            .unwrap_or(0);

        // Render from that scene upward
        for scene in self.stack[start_idx..].iter_mut() {
            scene.render(world, draw);
        }
    }

    /// Render GUI for the active scene.
    #[cfg(feature = "gui")]
    pub fn gui(&mut self, world: &mut World, gui: &mut EguiRenderer) {
        if let Some(scene) = self.stack.last_mut() {
            scene.gui(world, gui);
        }
    }

    /// Apply a transition command.
    pub fn apply_transition(
        &mut self,
        transition: Transition,
        world: &mut World,
        gfx: &crate::renderer::Graphics,
    ) -> bool {
        match transition {
            Transition::None => false,
            Transition::Push(scene) => {
                self.push(scene, world, gfx);
                false
            }
            Transition::Pop => {
                self.pop(world);
                self.is_empty()
            }
            Transition::Switch(scene) => {
                self.switch(scene, world, gfx);
                false
            }
            Transition::Replace(scene) => {
                self.replace(scene, world, gfx);
                false
            }
            Transition::Quit => true,
        }
    }

    /// Check if there are no scenes.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the number of scenes in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyScene {
        entered: bool,
        exited: bool,
    }

    impl DummyScene {
        fn new() -> Self {
            Self {
                entered: false,
                exited: false,
            }
        }
    }

    impl Scene for DummyScene {
        fn on_enter(&mut self, _world: &mut World, _gfx: &crate::renderer::Graphics) {
            self.entered = true;
        }

        fn on_exit(&mut self, _world: &mut World) {
            self.exited = true;
        }

        fn update(&mut self, _world: &mut World, _res: &Resources) -> Transition {
            Transition::None
        }

        fn render(&mut self, _world: &World, _draw: &mut DrawContext) {}
    }

    #[test]
    fn test_scene_manager_creation() {
        let manager = SceneManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }
}
