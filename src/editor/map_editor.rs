//! Main map editor component.

use std::collections::HashMap;

use crate::core::color::Color;
use crate::core::math::Vec2;
use crate::ecs::{World, Transform, Sprite, Visible, SpriteShape};
use crate::input::InputState;
use crate::renderer::Camera2D;

use super::asset_palette::AssetPalette;
use super::map_object::{MapObject, MapObjectId, PlacedObject, ObjectVisual};
use super::serialization::{MapData, MapError, save_map, load_map};
use super::tools::{EditorTool, ToolState, GridConfig};

/// Editor configuration.
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// Whether the editor is enabled.
    pub enabled: bool,
    /// Whether to show the editor UI panel.
    pub show_panel: bool,
    /// Panel width in pixels.
    pub panel_width: f32,
    /// Whether to show object outlines.
    pub show_outlines: bool,
    /// Selection highlight color.
    pub selection_color: Color,
    /// Hover highlight color.
    pub hover_color: Color,
    /// Grid configuration.
    pub grid: GridConfig,
    /// Undo history size.
    pub max_undo_steps: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_panel: true,
            panel_width: 250.0,
            show_outlines: true,
            selection_color: Color::rgba(0.2, 0.6, 1.0, 0.8),
            hover_color: Color::rgba(1.0, 1.0, 0.2, 0.5),
            grid: GridConfig::default(),
            max_undo_steps: 50,
        }
    }
}

/// Current editor state.
#[derive(Debug, Clone, Default)]
pub struct EditorState {
    /// Currently hovered object.
    pub hovered: Option<MapObjectId>,
    /// Selected objects.
    pub selected: Vec<MapObjectId>,
    /// Object being dragged.
    pub dragging: Option<MapObjectId>,
    /// Mouse position in world coordinates.
    pub mouse_world_pos: Vec2,
    /// Status message to display.
    pub status_message: Option<String>,
    /// Whether the editor panel has focus (to block game input).
    pub panel_focused: bool,
}

/// The main map editor.
pub struct MapEditor {
    /// Editor configuration.
    pub config: EditorConfig,
    /// Current editor state.
    pub state: EditorState,
    /// Asset palette.
    pub palette: AssetPalette,
    /// Tool state.
    pub tool: ToolState,
    /// All placed objects.
    objects: Vec<PlacedObject>,
    /// Quick lookup by ID.
    object_index: HashMap<MapObjectId, usize>,
    /// Current map name.
    map_name: String,
    /// Whether the map has unsaved changes.
    dirty: bool,
    /// Undo history.
    undo_stack: Vec<Vec<PlacedObject>>,
    /// Redo history.
    redo_stack: Vec<Vec<PlacedObject>>,
}

impl Default for MapEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl MapEditor {
    /// Create a new map editor.
    pub fn new() -> Self {
        Self {
            config: EditorConfig::default(),
            state: EditorState::default(),
            palette: AssetPalette::new(),
            tool: ToolState::new(),
            objects: Vec::new(),
            object_index: HashMap::new(),
            map_name: "Untitled".to_string(),
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Create a new map editor with configuration.
    pub fn with_config(config: EditorConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    // =========================================================================
    // Asset Registration
    // =========================================================================

    /// Register a map object that can be placed.
    pub fn register_object(&mut self, object: MapObject) {
        self.palette.register(object);
    }

    /// Register a simple circle object.
    pub fn register_circle(
        &mut self,
        name: impl Into<String>,
        radius: f32,
        color: Color,
    ) {
        self.palette.register(MapObject::circle(name, radius, color));
    }

    /// Register a simple rectangle object.
    pub fn register_rect(
        &mut self,
        name: impl Into<String>,
        width: f32,
        height: f32,
        color: Color,
    ) {
        self.palette.register(MapObject::rect(name, width, height, color));
    }

    /// Register a texture-based object.
    pub fn register_texture(
        &mut self,
        name: impl Into<String>,
        texture_name: impl Into<String>,
        size: Vec2,
    ) {
        self.palette.register(MapObject::texture(name, texture_name, size));
    }

    /// Register a texture-based object with a category.
    pub fn register_texture_with_category(
        &mut self,
        name: impl Into<String>,
        texture_name: impl Into<String>,
        size: Vec2,
        category: impl Into<String>,
    ) {
        let obj = MapObject::texture(name, texture_name, size)
            .with_category(category);
        self.palette.register(obj);
    }

    // =========================================================================
    // Object Management
    // =========================================================================

    /// Place an object at the given position.
    pub fn place_object(&mut self, object_type: &str, position: Vec2) -> Option<MapObjectId> {
        // Verify the object type exists
        if self.palette.get_object(object_type).is_none() {
            return None;
        }

        // Snap to grid if enabled
        let position = self.config.grid.snap(position);

        // Get the default layer from the object definition
        let layer = self.palette.get_object(object_type)
            .map(|o| o.default_layer)
            .unwrap_or(0);

        // Create the placed object
        let obj = PlacedObject::new(object_type, position).with_layer(layer);
        let id = obj.id;

        // Save undo state
        self.save_undo();

        // Add to our list
        let index = self.objects.len();
        self.objects.push(obj);
        self.object_index.insert(id, index);
        self.dirty = true;

        Some(id)
    }

    /// Delete an object by ID.
    pub fn delete_object(&mut self, id: MapObjectId) -> bool {
        if let Some(&index) = self.object_index.get(&id) {
            // Save undo state
            self.save_undo();

            // Remove the object
            self.objects.swap_remove(index);
            self.object_index.remove(&id);

            // Update the index of the swapped object
            if index < self.objects.len() {
                let swapped_id = self.objects[index].id;
                self.object_index.insert(swapped_id, index);
            }

            // Clear selection if deleted
            self.state.selected.retain(|&selected_id| selected_id != id);

            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Delete all selected objects.
    pub fn delete_selected(&mut self) {
        let selected: Vec<_> = self.state.selected.clone();
        for id in selected {
            self.delete_object(id);
        }
    }

    /// Get an object by ID.
    pub fn get_object(&self, id: MapObjectId) -> Option<&PlacedObject> {
        self.object_index.get(&id)
            .and_then(|&index| self.objects.get(index))
    }

    /// Get a mutable reference to an object by ID.
    pub fn get_object_mut(&mut self, id: MapObjectId) -> Option<&mut PlacedObject> {
        self.object_index.get(&id)
            .and_then(|&index| self.objects.get_mut(index))
    }

    /// Get all placed objects.
    pub fn objects(&self) -> &[PlacedObject] {
        &self.objects
    }

    /// Clear all objects.
    pub fn clear(&mut self) {
        self.save_undo();
        self.objects.clear();
        self.object_index.clear();
        self.state.selected.clear();
        self.state.hovered = None;
        self.dirty = true;
    }

    // =========================================================================
    // Selection
    // =========================================================================

    /// Select an object.
    pub fn select(&mut self, id: MapObjectId) {
        if !self.state.selected.contains(&id) {
            self.state.selected.push(id);
            if let Some(obj) = self.get_object_mut(id) {
                obj.selected = true;
            }
        }
    }

    /// Deselect an object.
    pub fn deselect(&mut self, id: MapObjectId) {
        self.state.selected.retain(|&selected_id| selected_id != id);
        if let Some(obj) = self.get_object_mut(id) {
            obj.selected = false;
        }
    }

    /// Toggle selection of an object.
    pub fn toggle_select(&mut self, id: MapObjectId) {
        if self.state.selected.contains(&id) {
            self.deselect(id);
        } else {
            self.select(id);
        }
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        for id in self.state.selected.clone() {
            if let Some(obj) = self.get_object_mut(id) {
                obj.selected = false;
            }
        }
        self.state.selected.clear();
    }

    /// Select all objects.
    pub fn select_all(&mut self) {
        self.state.selected.clear();
        for obj in &mut self.objects {
            if !obj.locked {
                obj.selected = true;
                self.state.selected.push(obj.id);
            }
        }
    }

    /// Find object at a position.
    pub fn object_at(&self, pos: Vec2) -> Option<MapObjectId> {
        // Check in reverse order (top-most first)
        for obj in self.objects.iter().rev() {
            if let Some(object_def) = self.palette.get_object(&obj.object_type) {
                if obj.contains_point(pos, object_def) {
                    return Some(obj.id);
                }
            }
        }
        None
    }

    /// Find all objects in a rectangle.
    pub fn objects_in_rect(&self, min: Vec2, max: Vec2) -> Vec<MapObjectId> {
        let mut result = Vec::new();
        for obj in &self.objects {
            if let Some(object_def) = self.palette.get_object(&obj.object_type) {
                let (obj_min, obj_max) = obj.bounds(object_def);
                // Check if rectangles overlap
                if obj_min.x <= max.x && obj_max.x >= min.x
                    && obj_min.y <= max.y && obj_max.y >= min.y
                {
                    result.push(obj.id);
                }
            }
        }
        result
    }

    // =========================================================================
    // Movement
    // =========================================================================

    /// Move an object to a new position.
    pub fn move_object(&mut self, id: MapObjectId, position: Vec2) {
        let snapped = self.config.grid.snap(position);
        if let Some(obj) = self.get_object_mut(id) {
            if !obj.locked {
                obj.position = snapped;
                self.dirty = true;
            }
        }
    }

    /// Move selected objects by a delta.
    pub fn move_selected(&mut self, delta: Vec2) {
        self.save_undo();
        let grid = self.config.grid.clone();
        for id in self.state.selected.clone() {
            if let Some(obj) = self.get_object_mut(id) {
                if !obj.locked {
                    let new_pos = obj.position + delta;
                    obj.position = grid.snap(new_pos);
                }
            }
        }
        self.dirty = true;
    }

    // =========================================================================
    // Undo/Redo
    // =========================================================================

    fn save_undo(&mut self) {
        self.undo_stack.push(self.objects.clone());
        if self.undo_stack.len() > self.config.max_undo_steps {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo the last action.
    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(self.objects.clone());
            self.objects = state;
            self.rebuild_index();
            self.dirty = true;
        }
    }

    /// Redo the last undone action.
    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(self.objects.clone());
            self.objects = state;
            self.rebuild_index();
            self.dirty = true;
        }
    }

    fn rebuild_index(&mut self) {
        self.object_index.clear();
        for (i, obj) in self.objects.iter().enumerate() {
            self.object_index.insert(obj.id, i);
        }
        // Update selection state
        self.state.selected.retain(|id| self.object_index.contains_key(id));
    }

    // =========================================================================
    // Save/Load
    // =========================================================================

    /// Get the current map as serializable data.
    pub fn to_map_data(&self) -> MapData {
        let mut data = MapData::new(&self.map_name);
        data.objects = MapData::from_placed_objects(&self.objects);
        data
    }

    /// Load map data into the editor.
    pub fn from_map_data(&mut self, data: &MapData) {
        self.clear();
        self.map_name = data.name.clone();
        
        for obj in data.to_placed_objects() {
            let id = obj.id;
            let index = self.objects.len();
            self.objects.push(obj);
            self.object_index.insert(id, index);
        }
        
        self.dirty = false;
    }

    /// Save the map to a file.
    pub fn save(&self, path: &str) -> Result<(), MapError> {
        let data = self.to_map_data();
        save_map(path, &data)
    }

    /// Load a map from a file.
    pub fn load(&mut self, path: &str) -> Result<(), MapError> {
        let data = load_map(path)?;
        self.from_map_data(&data);
        Ok(())
    }

    /// Check if there are unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark as saved.
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    // =========================================================================
    // Input Handling
    // =========================================================================

    /// Handle input events. Returns true if input was consumed.
    pub fn handle_input(
        &mut self,
        _world: &mut World,
        input: &InputState,
        camera: &Camera2D,
    ) -> bool {
        if !self.config.enabled || self.state.panel_focused {
            return false;
        }

        // Update mouse world position
        let mouse_screen = input.mouse_pos();
        self.state.mouse_world_pos = camera.screen_to_world(mouse_screen).into();

        // Handle keyboard shortcuts
        if input.is_key_just_pressed(winit::keyboard::KeyCode::Delete)
            || input.is_key_just_pressed(winit::keyboard::KeyCode::Backspace)
        {
            self.delete_selected();
            return true;
        }

        // Undo/Redo
        let ctrl = input.is_key_down(winit::keyboard::KeyCode::ControlLeft)
            || input.is_key_down(winit::keyboard::KeyCode::ControlRight)
            || input.is_key_down(winit::keyboard::KeyCode::SuperLeft)
            || input.is_key_down(winit::keyboard::KeyCode::SuperRight);

        if ctrl && input.is_key_just_pressed(winit::keyboard::KeyCode::KeyZ) {
            if input.is_key_down(winit::keyboard::KeyCode::ShiftLeft) {
                self.redo();
            } else {
                self.undo();
            }
            return true;
        }

        // Select all
        if ctrl && input.is_key_just_pressed(winit::keyboard::KeyCode::KeyA) {
            self.select_all();
            return true;
        }

        // Tool shortcuts
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyV) {
            self.tool.tool = EditorTool::Select;
            return true;
        }
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyP) {
            self.tool.tool = EditorTool::Place;
            return true;
        }
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyE) {
            self.tool.tool = EditorTool::Erase;
            return true;
        }
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyH) {
            self.tool.tool = EditorTool::Pan;
            return true;
        }
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyB) {
            self.tool.tool = EditorTool::Paint;
            return true;
        }

        // Toggle grid
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyG) {
            self.config.grid.enabled = !self.config.grid.enabled;
            return true;
        }

        // Update hover state
        self.state.hovered = self.object_at(self.state.mouse_world_pos);

        // Handle mouse input based on tool
        let mouse_pos = self.state.mouse_world_pos;
        let left_pressed = input.is_mouse_down(winit::event::MouseButton::Left);
        let left_just_pressed = input.is_mouse_just_pressed(winit::event::MouseButton::Left);
        let left_just_released = input.is_mouse_just_released(winit::event::MouseButton::Left);

        match self.tool.tool {
            EditorTool::Select => {
                if left_just_pressed {
                    if let Some(id) = self.state.hovered {
                        if !input.is_key_down(winit::keyboard::KeyCode::ShiftLeft) {
                            if !self.state.selected.contains(&id) {
                                self.clear_selection();
                            }
                        }
                        self.toggle_select(id);
                        // Start dragging
                        if let Some(obj) = self.get_object(id) {
                            self.tool.drag_offset = Some(obj.position - mouse_pos);
                            self.state.dragging = Some(id);
                        }
                    } else {
                        // Start selection rectangle
                        self.clear_selection();
                        self.tool.begin(mouse_pos);
                    }
                    return true;
                }

                if left_pressed && self.tool.active {
                    self.tool.update(mouse_pos);
                    return true;
                }

                // Drag selected objects
                if left_pressed {
                    if let Some(offset) = self.tool.drag_offset {
                        let new_pos = mouse_pos + offset;
                        let grid = self.config.grid.clone();
                        if let Some(dragging_id) = self.state.dragging {
                            let delta = if let Some(obj) = self.get_object(dragging_id) {
                                new_pos - obj.position
                            } else {
                                Vec2::ZERO
                            };
                            // Move all selected
                            for id in self.state.selected.clone() {
                                if let Some(obj) = self.get_object_mut(id) {
                                    if !obj.locked {
                                        obj.position = grid.snap(obj.position + delta);
                                    }
                                }
                            }
                            self.dirty = true;
                        }
                    }
                    return true;
                }

                if left_just_released {
                    if self.tool.active {
                        // Select objects in rectangle
                        if let Some((min, max)) = self.tool.selection_rect() {
                            let ids = self.objects_in_rect(min, max);
                            for id in ids {
                                self.select(id);
                            }
                        }
                        self.tool.end();
                    } else {
                        self.tool.drag_offset = None;
                        self.state.dragging = None;
                    }
                    return true;
                }
            }

            EditorTool::Place => {
                if left_just_pressed {
                    // Get selected asset name first to avoid borrow issues
                    let selected = self.palette.selected().map(|s| s.to_string());
                    if let Some(selected_asset) = selected {
                        self.place_object(&selected_asset, mouse_pos);
                    }
                    return true;
                }
            }

            EditorTool::Erase => {
                if left_pressed {
                    if let Some(id) = self.state.hovered {
                        self.delete_object(id);
                    }
                    return true;
                }
            }

            EditorTool::Paint => {
                if left_just_pressed {
                    self.tool.begin(mouse_pos);
                }
                if left_pressed && self.tool.should_paint(mouse_pos) {
                    // Get selected asset name first to avoid borrow issues
                    let selected = self.palette.selected().map(|s| s.to_string());
                    if let Some(selected_asset) = selected {
                        self.place_object(&selected_asset, mouse_pos);
                        self.tool.record_paint(mouse_pos);
                    }
                    return true;
                }
                if left_just_released {
                    self.tool.end();
                }
            }

            EditorTool::Pan | EditorTool::Measure => {
                // These are handled by the camera/renderer
            }
        }

        false
    }

    // =========================================================================
    // Synchronization with ECS World
    // =========================================================================

    /// Sync placed objects to the ECS world (creates entities).
    pub fn sync_to_world(&self, world: &mut World) {
        for obj in &self.objects {
            if let Some(object_def) = self.palette.get_object(&obj.object_type) {
                let sprite = match &object_def.visual {
                    ObjectVisual::Circle { radius, color } => {
                        Sprite::circle(*radius * obj.scale.x, *color)
                    }
                    ObjectVisual::Rect { color } => {
                        Sprite::rect(
                            object_def.size.x * obj.scale.x,
                            object_def.size.y * obj.scale.y,
                            *color,
                        )
                    }
                    ObjectVisual::Texture { texture_name, tint } => {
                        Sprite {
                            shape: SpriteShape::Texture {
                                region_name: texture_name.clone(),
                                size: Vec2::new(
                                    object_def.size.x * obj.scale.x,
                                    object_def.size.y * obj.scale.y,
                                ),
                            },
                            color: *tint,
                            z_order: obj.layer,
                        }
                    }
                    ObjectVisual::Line { color, thickness: _ } => {
                        // Lines need special handling
                        Sprite::rect(object_def.size.x, object_def.size.y, *color)
                    }
                };

                let transform = Transform {
                    position: obj.position,
                    rotation: obj.rotation,
                    scale: obj.scale,
                };

                world.spawn((transform, sprite.with_z_order(obj.layer), Visible));
            }
        }
    }

    // =========================================================================
    // GUI
    // =========================================================================

    /// Render the editor UI.
    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ctx: &egui::Context, _world: &World) {
        if !self.config.enabled {
            return;
        }

        // Track if panel has focus
        self.state.panel_focused = false;

        // Main editor panel
        if self.config.show_panel {
            egui::SidePanel::left("editor_panel")
                .default_width(self.config.panel_width)
                .show(ctx, |ui| {
                    self.state.panel_focused = ui.ui_contains_pointer();
                    
                    // Header
                    ui.horizontal(|ui| {
                        ui.heading("🗺 Map Editor");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.dirty {
                                ui.label("●").on_hover_text("Unsaved changes");
                            }
                        });
                    });

                    ui.separator();

                    // Toolbar
                    ui.horizontal_wrapped(|ui| {
                        for tool in EditorTool::all() {
                            let is_selected = self.tool.tool == *tool;
                            let btn = egui::Button::new(tool.icon())
                                .selected(is_selected);
                            let response = ui.add(btn);
                            if response.clicked() {
                                self.tool.tool = *tool;
                            }
                            response.on_hover_text(format!(
                                "{} ({})",
                                tool.name(),
                                tool.shortcut().map(|c| c.to_string()).unwrap_or_default()
                            ));
                        }
                    });

                    ui.separator();

                    // Grid settings
                    egui::CollapsingHeader::new("⊞ Grid")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.checkbox(&mut self.config.grid.enabled, "Snap to grid");
                            ui.checkbox(&mut self.config.grid.visible, "Show grid");
                            ui.horizontal(|ui| {
                                ui.label("Size:");
                                ui.add(egui::DragValue::new(&mut self.config.grid.cell_size.x)
                                    .speed(1.0)
                                    .range(1.0..=256.0));
                                ui.label("x");
                                ui.add(egui::DragValue::new(&mut self.config.grid.cell_size.y)
                                    .speed(1.0)
                                    .range(1.0..=256.0));
                            });
                        });

                    ui.separator();

                    // Asset palette
                    egui::CollapsingHeader::new("📦 Assets")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.palette.ui(ui);
                        });

                    ui.separator();

                    // Selection info
                    if !self.state.selected.is_empty() {
                        egui::CollapsingHeader::new("🎯 Selection")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(format!("{} object(s) selected", self.state.selected.len()));
                                
                                if self.state.selected.len() == 1 {
                                    let id = self.state.selected[0];
                                    // Get values first to avoid borrow issues
                                    let obj_values = self.get_object(id).map(|obj| {
                                        (obj.position, obj.rotation, obj.scale, obj.layer, obj.locked)
                                    });
                                    
                                    if let Some((mut pos, mut rot, mut scale, mut layer, mut locked)) = obj_values {
                                        let original = (pos, rot, scale, layer, locked);
                                        
                                        ui.horizontal(|ui| {
                                            ui.label("Position:");
                                            ui.add(egui::DragValue::new(&mut pos.x).speed(1.0));
                                            ui.add(egui::DragValue::new(&mut pos.y).speed(1.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Rotation:");
                                            let mut degrees = rot.to_degrees();
                                            if ui.add(egui::DragValue::new(&mut degrees).speed(1.0).suffix("°")).changed() {
                                                rot = degrees.to_radians();
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Scale:");
                                            ui.add(egui::DragValue::new(&mut scale.x).speed(0.1));
                                            ui.add(egui::DragValue::new(&mut scale.y).speed(0.1));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Layer:");
                                            ui.add(egui::DragValue::new(&mut layer));
                                        });
                                        ui.checkbox(&mut locked, "Locked");
                                        
                                        // Apply changes back
                                        if (pos, rot, scale, layer, locked) != original {
                                            if let Some(obj) = self.get_object_mut(id) {
                                                obj.position = pos;
                                                obj.rotation = rot;
                                                obj.scale = scale;
                                                obj.layer = layer;
                                                obj.locked = locked;
                                            }
                                            self.dirty = true;
                                        }
                                    }
                                }

                                if ui.button("🗑 Delete Selected").clicked() {
                                    self.delete_selected();
                                }
                            });
                    }

                    ui.separator();

                    // Map info
                    ui.horizontal(|ui| {
                        ui.label("Objects:");
                        ui.label(self.objects.len().to_string());
                    });

                    // Status
                    if let Some(msg) = &self.state.status_message {
                        ui.separator();
                        ui.label(msg);
                    }
                });
        }

        // Keyboard shortcuts info (bottom)
        if self.config.enabled {
            egui::TopBottomPanel::bottom("editor_shortcuts")
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Tool: {} | Grid: {} | Pos: ({:.0}, {:.0})",
                            self.tool.tool.name(),
                            if self.config.grid.enabled { "ON" } else { "OFF" },
                            self.state.mouse_world_pos.x,
                            self.state.mouse_world_pos.y,
                        ));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label("V:Select P:Place E:Erase G:Grid Del:Delete");
                        });
                    });
                });
        }
    }

    /// Toggle the editor on/off.
    pub fn toggle(&mut self) {
        self.config.enabled = !self.config.enabled;
    }

    /// Enable the editor.
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable the editor.
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Check if the editor is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}
