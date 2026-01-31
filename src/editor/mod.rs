//! In-game map editor for placing and arranging assets.
//!
//! This module provides a visual editor overlay that allows you to:
//! - Browse available assets in a palette
//! - Drag and drop assets onto the map
//! - Select, move, and delete placed objects
//! - Snap to grid for precise placement
//! - Save and load maps as JSON
//!
//! # Using the Plugin (Recommended)
//!
//! The easiest way to use the map editor is via the plugin system:
//!
//! ```ignore
//! use graviplex::prelude::*;
//! use graviplex::editor::MapEditorPlugin;
//!
//! struct MyGame {
//!     editor_plugin: MapEditorPlugin,
//! }
//!
//! impl GameLoop for MyGame {
//!     fn init(&mut self, world: &mut World, gfx: &Graphics) {
//!         // Plugin auto-scans asset directory and registers primitives
//!     }
//!
//!     fn handle_input(&mut self, world: &mut World, input: &InputState, camera: &Camera2D) -> bool {
//!         // F1 or ` toggles the editor
//!         self.editor_plugin.handle_input(world, input, camera)
//!     }
//!
//!     fn render(&mut self, world: &World, draw: &mut DrawContext) {
//!         // Render your game...
//!         
//!         // Then render editor overlays
//!         self.editor_plugin.render_editor(draw);
//!     }
//!
//!     fn gui(&mut self, ctx: &egui::Context) {
//!         self.editor_plugin.editor.ui(ctx, &World::new());
//!     }
//! }
//! ```
//!
//! # Manual Integration
//!
//! For more control, use MapEditor directly:
//!
//! ```ignore
//! use graviplex::editor::{MapEditor, MapObject};
//!
//! let mut editor = MapEditor::new();
//! editor.register_circle("Player", 32.0, Color::GREEN);
//! editor.register_object(
//!     MapObject::rect("Wall", 128.0, 32.0, Color::GRAY)
//!         .with_category("Environment")
//! );
//! ```

mod asset_palette;
mod map_editor;
mod map_object;
mod plugin;
mod serialization;
mod tools;

pub use asset_palette::{AssetEntry, AssetPalette, AssetKind};
pub use map_editor::{MapEditor, EditorConfig, EditorState};
pub use map_object::{MapObject, MapObjectId, PlacedObject, ObjectVisual, PropertyValue, CollisionShape};
pub use plugin::{MapEditorPlugin, PendingSpriteSheet};
pub use serialization::{MapData, MapError, save_map, load_map, save_map_to_string, load_map_from_string};
pub use tools::{EditorTool, ToolState, GridConfig};
