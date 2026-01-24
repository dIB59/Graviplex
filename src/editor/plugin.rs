//! Map Editor Plugin for easy integration.

use std::path::Path;

use crate::core::color::Color;
use crate::core::math::Vec2;
use crate::core::stats::FpsCounter;
use crate::ecs::World;
use crate::input::InputState;
use crate::plugin::Plugin;
use crate::renderer::{Camera2D, DrawContext};
use crate::Time;

use super::map_editor::{MapEditor, EditorConfig};
use super::map_object::{MapObject, ObjectVisual};
use super::tools::EditorTool;

/// Map Editor Plugin - provides in-game level editing capabilities.
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
/// use graviplex::editor::MapEditorPlugin;
///
/// App::build(MyGame::new())
///     .add_plugin(MapEditorPlugin::new("assets/sprites"))
///     .run()
///     .unwrap();
/// ```
pub struct MapEditorPlugin {
    /// The map editor instance.
    pub editor: MapEditor,
    /// Asset directory to scan.
    asset_dir: Option<String>,
    /// Whether assets have been scanned.
    assets_scanned: bool,
    /// Cached camera for coordinate conversion.
    last_screen_size: [f32; 2],
}

impl Default for MapEditorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MapEditorPlugin {
    /// Create a new map editor plugin.
    pub fn new() -> Self {
        Self {
            editor: MapEditor::new(),
            asset_dir: None,
            assets_scanned: false,
            last_screen_size: [1280.0, 720.0],
        }
    }

    /// Create with a specific configuration.
    pub fn with_config(config: EditorConfig) -> Self {
        Self {
            editor: MapEditor::with_config(config),
            asset_dir: None,
            assets_scanned: false,
            last_screen_size: [1280.0, 720.0],
        }
    }

    /// Set the asset directory to scan for textures.
    pub fn with_asset_dir(mut self, dir: impl Into<String>) -> Self {
        self.asset_dir = Some(dir.into());
        self
    }

    /// Enable the editor by default.
    pub fn enabled(mut self) -> Self {
        self.editor.config.enabled = true;
        self
    }

    /// Start with the editor disabled.
    pub fn disabled(mut self) -> Self {
        self.editor.config.enabled = false;
        self
    }

    /// Get mutable access to the underlying editor.
    pub fn editor_mut(&mut self) -> &mut MapEditor {
        &mut self.editor
    }

    /// Get read-only access to the underlying editor.
    pub fn editor(&self) -> &MapEditor {
        &self.editor
    }

    /// Set grid size.
    pub fn with_grid_size(mut self, size: f32) -> Self {
        self.editor.config.grid.cell_size = Vec2::splat(size);
        self
    }

    /// Enable grid snapping.
    pub fn with_grid_snap(mut self, enabled: bool) -> Self {
        self.editor.config.grid.enabled = enabled;
        self
    }

    /// Register a placeable object.
    pub fn register(mut self, object: MapObject) -> Self {
        self.editor.register_object(object);
        self
    }

    /// Scan a directory for image assets and register them.
    fn scan_asset_directory(&mut self) {
        if self.assets_scanned {
            return;
        }
        self.assets_scanned = true;

        let Some(dir) = self.asset_dir.clone() else {
            println!("[MapEditor] No asset directory configured");
            return;
        };

        let path = Path::new(&dir);
        if !path.exists() || !path.is_dir() {
            eprintln!("[MapEditor] Asset directory not found: {}", dir);
            return;
        }

        println!("[MapEditor] Scanning asset directory: {}", dir);
        self.scan_directory_recursive(path, "Sprites");
        println!("[MapEditor] Found {} assets", self.editor.palette.len());
    }

    fn scan_directory_recursive(&mut self, path: &Path, category: &str) {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Use directory name as sub-category
                if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    let sub_category = if category == "Sprites" {
                        dir_name.to_string()
                    } else {
                        format!("{}/{}", category, dir_name)
                    };
                    self.scan_directory_recursive(&entry_path, &sub_category);
                }
            } else if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                // Check if it's an image file
                let ext_lower = ext.to_lowercase();
                if matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp") {
                    if let Some(file_name) = entry_path.file_stem().and_then(|n| n.to_str()) {
                        // Get relative path for texture name
                        let texture_path = entry_path.to_string_lossy().to_string();
                        
                        // Create a texture-based object
                        // Default size - could be improved by reading image dimensions
                        let size = Vec2::new(64.0, 64.0);
                        
                        // Format display name nicely
                        let display_name = file_name
                            .replace('_', " ")
                            .replace('-', " ");
                        
                        let object = MapObject::texture(&display_name, &texture_path, size)
                            .with_category(category);
                        
                        println!("[MapEditor] Registered: {} ({})", display_name, category);
                        self.editor.register_object(object);
                    }
                }
            }
        }
    }

    /// Handle input for the editor. Call this from your game's handle_input.
    pub fn handle_input(&mut self, world: &mut World, input: &InputState, camera: &Camera2D) -> bool {
        // Update screen size from camera
        self.last_screen_size = camera.screen_size;
        
        // Toggle editor with F1 or backtick
        if input.is_key_just_pressed(winit::keyboard::KeyCode::F1)
            || input.is_key_just_pressed(winit::keyboard::KeyCode::Backquote)
        {
            self.editor.toggle();
            return true;
        }

        self.editor.handle_input(world, input, camera)
    }

    /// Render the editor overlays.
    pub fn render_editor(&self, draw: &mut DrawContext) {
        if !self.editor.is_enabled() {
            return;
        }

        // Draw grid
        if self.editor.config.grid.visible {
            self.render_grid(draw);
        }

        // Draw placed objects
        self.render_objects(draw);

        // Draw placement preview
        self.render_preview(draw);

        // Draw selection rectangles
        self.render_selections(draw);
    }

    fn render_grid(&self, draw: &mut DrawContext) {
        let grid_size = self.editor.config.grid.cell_size;
        let grid_color = Color::rgba(0.3, 0.3, 0.3, 0.2);
        let major_color = Color::rgba(0.5, 0.5, 0.5, 0.3);
        
        let half_width = self.last_screen_size[0] * 0.6;
        let half_height = self.last_screen_size[1] * 0.6;
        
        let start_x = (-half_width / grid_size.x).floor() as i32;
        let end_x = (half_width / grid_size.x).ceil() as i32;
        let start_y = (-half_height / grid_size.y).floor() as i32;
        let end_y = (half_height / grid_size.y).ceil() as i32;
        
        // Vertical lines
        for i in start_x..=end_x {
            let x = i as f32 * grid_size.x;
            let color = if i % 4 == 0 { major_color } else { grid_color };
            draw.line((
                Vec2::new(x, start_y as f32 * grid_size.y),
                Vec2::new(x, end_y as f32 * grid_size.y),
                color,
            ));
        }
        
        // Horizontal lines
        for i in start_y..=end_y {
            let y = i as f32 * grid_size.y;
            let color = if i % 4 == 0 { major_color } else { grid_color };
            draw.line((
                Vec2::new(start_x as f32 * grid_size.x, y),
                Vec2::new(end_x as f32 * grid_size.x, y),
                color,
            ));
        }
        
        // Origin axes
        draw.line((
            Vec2::new(-half_width, 0.0),
            Vec2::new(half_width, 0.0),
            Color::rgba(1.0, 0.3, 0.3, 0.4),
        ));
        draw.line((
            Vec2::new(0.0, -half_height),
            Vec2::new(0.0, half_height),
            Color::rgba(0.3, 1.0, 0.3, 0.4),
        ));
    }

    fn render_objects(&self, draw: &mut DrawContext) {
        for obj in self.editor.objects() {
            if let Some(object_def) = self.editor.palette.get_object(&obj.object_type) {
                let is_selected = obj.selected;
                let is_hovered = self.editor.state.hovered == Some(obj.id);
                
                match &object_def.visual {
                    ObjectVisual::Circle { radius, color } => {
                        let scaled_radius = radius * obj.scale.x;
                        let mut c = *color;
                        
                        if is_selected {
                            c = Color::rgba(
                                (c.r * 1.2).min(1.0),
                                (c.g * 1.2).min(1.0),
                                (c.b * 1.2).min(1.0),
                                c.a,
                            );
                        } else if is_hovered {
                            c = Color::rgba(c.r, c.g, c.b, c.a * 0.8);
                        }
                        
                        draw.circle(crate::core::geometry::Circle::new(obj.position, scaled_radius, c));
                        
                        // Selection outline
                        if is_selected {
                            self.draw_circle_outline(draw, obj.position, scaled_radius + 4.0, self.editor.config.selection_color);
                        }
                    }
                    ObjectVisual::Rect { color } => {
                        let scaled_size = Vec2::new(
                            object_def.size.x * obj.scale.x,
                            object_def.size.y * obj.scale.y,
                        );
                        let mut c = *color;
                        
                        if is_selected {
                            c = Color::rgba(
                                (c.r * 1.2).min(1.0),
                                (c.g * 1.2).min(1.0),
                                (c.b * 1.2).min(1.0),
                                c.a,
                            );
                        } else if is_hovered {
                            c = Color::rgba(c.r, c.g, c.b, c.a * 0.8);
                        }
                        
                        self.draw_filled_rect(draw, obj.position, scaled_size, c);
                        
                        // Selection outline
                        if is_selected {
                            self.draw_rect_outline(draw, obj.position, scaled_size + Vec2::splat(6.0), self.editor.config.selection_color);
                        }
                    }
                    ObjectVisual::Texture { tint, texture_name } => {
                        let scaled_size = Vec2::new(
                            object_def.size.x * obj.scale.x,
                            object_def.size.y * obj.scale.y,
                        );
                        
                        // Apply selection/hover tint
                        let render_tint = if is_selected {
                            Color::rgba(
                                (tint.r * 1.3).min(1.0),
                                (tint.g * 1.3).min(1.0),
                                (tint.b * 1.3).min(1.0),
                                tint.a,
                            )
                        } else if is_hovered {
                            Color::rgba(tint.r, tint.g, tint.b, tint.a * 0.8)
                        } else {
                            *tint
                        };
                        
                        // Try to render the actual texture
                        #[cfg(feature = "textures")]
                        {
                            if draw.has_texture(texture_name) {
                                draw.texture_ex(
                                    texture_name,
                                    obj.position,
                                    scaled_size,
                                    render_tint,
                                    obj.rotation,
                                    0,
                                );
                            } else {
                                // Texture not in atlas - draw placeholder
                                self.draw_texture_placeholder(draw, obj.position, scaled_size, is_selected, is_hovered);
                            }
                        }
                        
                        #[cfg(not(feature = "textures"))]
                        {
                            // No texture support - draw placeholder
                            self.draw_texture_placeholder(draw, obj.position, scaled_size, is_selected, is_hovered);
                        }
                        
                        if is_selected {
                            self.draw_rect_outline(draw, obj.position, scaled_size + Vec2::splat(6.0), self.editor.config.selection_color);
                        }
                    }
                    ObjectVisual::Line { color, .. } => {
                        draw.line((
                            obj.position,
                            obj.position + object_def.size,
                            *color,
                        ));
                    }
                }
            }
        }
    }

    fn render_preview(&self, draw: &mut DrawContext) {
        if !matches!(self.editor.tool.tool, EditorTool::Place | EditorTool::Paint) {
            return;
        }

        let Some(selected) = self.editor.palette.selected() else {
            return;
        };

        let Some(object_def) = self.editor.palette.get_object(selected) else {
            return;
        };

        let preview_pos = self.editor.config.grid.snap(self.editor.state.mouse_world_pos);
        let preview_alpha = 0.6;

        match &object_def.visual {
            ObjectVisual::Circle { radius, color } => {
                let c = Color::rgba(color.r, color.g, color.b, preview_alpha);
                draw.circle(crate::core::geometry::Circle::new(preview_pos, *radius, c));
                self.draw_circle_outline(draw, preview_pos, *radius, Color::rgba(1.0, 1.0, 1.0, 0.8));
            }
            ObjectVisual::Rect { color } => {
                let c = Color::rgba(color.r, color.g, color.b, preview_alpha);
                self.draw_filled_rect(draw, preview_pos, object_def.size, c);
                self.draw_rect_outline(draw, preview_pos, object_def.size, Color::rgba(1.0, 1.0, 1.0, 0.8));
            }
            ObjectVisual::Texture { tint, texture_name } => {
                let preview_tint = Color::rgba(tint.r, tint.g, tint.b, preview_alpha);
                
                // Try to render the actual texture
                #[cfg(feature = "textures")]
                {
                    if draw.has_texture(texture_name) {
                        draw.texture_ex(
                            texture_name,
                            preview_pos,
                            object_def.size,
                            preview_tint,
                            0.0,
                            0,
                        );
                        // Outline for preview visibility
                        self.draw_rect_outline(draw, preview_pos, object_def.size, Color::rgba(1.0, 1.0, 1.0, 0.6));
                    } else {
                        // Texture not in atlas - draw placeholder
                        let c = Color::rgba(0.3, 0.5, 0.8, preview_alpha);
                        self.draw_filled_rect(draw, preview_pos, object_def.size, c);
                        self.draw_rect_outline(draw, preview_pos, object_def.size, Color::rgba(0.6, 0.8, 1.0, 0.9));
                        
                        // Draw X pattern
                        let half = object_def.size * 0.5;
                        let cross_color = Color::rgba(1.0, 1.0, 1.0, 0.4);
                        draw.line((preview_pos - half, preview_pos + half, cross_color));
                        draw.line((
                            Vec2::new(preview_pos.x - half.x, preview_pos.y + half.y),
                            Vec2::new(preview_pos.x + half.x, preview_pos.y - half.y),
                            cross_color,
                        ));
                    }
                }
                
                #[cfg(not(feature = "textures"))]
                {
                    // No texture support - draw placeholder
                    let c = Color::rgba(0.3, 0.5, 0.8, preview_alpha);
                    self.draw_filled_rect(draw, preview_pos, object_def.size, c);
                    self.draw_rect_outline(draw, preview_pos, object_def.size, Color::rgba(0.6, 0.8, 1.0, 0.9));
                    
                    // Draw X pattern
                    let half = object_def.size * 0.5;
                    let cross_color = Color::rgba(1.0, 1.0, 1.0, 0.4);
                    draw.line((preview_pos - half, preview_pos + half, cross_color));
                    draw.line((
                        Vec2::new(preview_pos.x - half.x, preview_pos.y + half.y),
                        Vec2::new(preview_pos.x + half.x, preview_pos.y - half.y),
                        cross_color,
                    ));
                }
            }
            ObjectVisual::Line { color, .. } => {
                let c = Color::rgba(color.r, color.g, color.b, preview_alpha);
                draw.line((preview_pos, preview_pos + object_def.size, c));
            }
        }
    }

    fn render_selections(&self, draw: &mut DrawContext) {
        // Draw selection rectangle if dragging
        if self.editor.tool.active && self.editor.tool.tool == EditorTool::Select {
            if let Some((min, max)) = self.editor.tool.selection_rect() {
                let size = max - min;
                let center = min + size * 0.5;
                self.draw_rect_outline(draw, center, size, Color::rgba(0.3, 0.7, 1.0, 0.8));
                // Fill
                let fill = Color::rgba(0.3, 0.7, 1.0, 0.1);
                self.draw_filled_rect(draw, center, size, fill);
            }
        }
    }

    fn draw_rect_outline(&self, draw: &mut DrawContext, center: Vec2, size: Vec2, color: Color) {
        let half = size * 0.5;
        let min = center - half;
        let max = center + half;
        
        draw.line((Vec2::new(min.x, min.y), Vec2::new(max.x, min.y), color));
        draw.line((Vec2::new(max.x, min.y), Vec2::new(max.x, max.y), color));
        draw.line((Vec2::new(max.x, max.y), Vec2::new(min.x, max.y), color));
        draw.line((Vec2::new(min.x, max.y), Vec2::new(min.x, min.y), color));
    }

    fn draw_filled_rect(&self, draw: &mut DrawContext, center: Vec2, size: Vec2, color: Color) {
        // Draw horizontal lines to fill the rect
        let half = size * 0.5;
        let min = center - half;
        let max = center + half;
        
        let line_spacing = 2.0;
        let num_lines = (size.y / line_spacing) as i32;
        
        for i in 0..=num_lines {
            let y = min.y + (i as f32) * line_spacing;
            if y <= max.y {
                draw.line((Vec2::new(min.x, y), Vec2::new(max.x, y), color));
            }
        }
    }

    fn draw_circle_outline(&self, draw: &mut DrawContext, center: Vec2, radius: f32, color: Color) {
        let segments = 32;
        let step = std::f32::consts::TAU / segments as f32;
        
        for i in 0..segments {
            let angle1 = i as f32 * step;
            let angle2 = (i + 1) as f32 * step;
            
            let p1 = center + Vec2::new(angle1.cos(), angle1.sin()) * radius;
            let p2 = center + Vec2::new(angle2.cos(), angle2.sin()) * radius;
            
            draw.line((p1, p2, color));
        }
    }

    fn draw_texture_placeholder(&self, draw: &mut DrawContext, center: Vec2, size: Vec2, is_selected: bool, is_hovered: bool) {
        // Draw a visible placeholder for textures not in atlas
        let base_color = if is_selected {
            Color::rgba(0.4, 0.6, 1.0, 0.9)
        } else if is_hovered {
            Color::rgba(0.3, 0.5, 0.8, 0.8)
        } else {
            Color::rgba(0.2, 0.4, 0.6, 0.7)
        };
        
        // Filled background
        self.draw_filled_rect(draw, center, size, base_color);
        
        // Border
        let border_color = Color::rgba(0.6, 0.8, 1.0, 0.9);
        self.draw_rect_outline(draw, center, size, border_color);
        
        // Draw an X to indicate it's a texture placeholder
        let half = size * 0.5;
        let cross_color = Color::rgba(1.0, 1.0, 1.0, 0.5);
        draw.line((
            center - half,
            center + half,
            cross_color,
        ));
        draw.line((
            Vec2::new(center.x - half.x, center.y + half.y),
            Vec2::new(center.x + half.x, center.y - half.y),
            cross_color,
        ));
    }
}

impl Plugin for MapEditorPlugin {
    fn name(&self) -> &'static str {
        "MapEditorPlugin"
    }

    fn init(&mut self, _world: &mut World) {
        // Scan asset directory on init
        self.scan_asset_directory();
        
        // Register some default primitives if no assets found
        if self.editor.palette.is_empty() {
            // Basic shapes
            self.editor.register_object(
                MapObject::circle("Circle (Small)", 16.0, Color::rgb(0.8, 0.2, 0.2))
                    .with_category("Primitives")
            );
            self.editor.register_object(
                MapObject::circle("Circle (Medium)", 32.0, Color::rgb(0.2, 0.8, 0.2))
                    .with_category("Primitives")
            );
            self.editor.register_object(
                MapObject::circle("Circle (Large)", 64.0, Color::rgb(0.2, 0.2, 0.8))
                    .with_category("Primitives")
            );
            self.editor.register_object(
                MapObject::rect("Box (Small)", 32.0, 32.0, Color::rgb(0.8, 0.8, 0.2))
                    .with_category("Primitives")
            );
            self.editor.register_object(
                MapObject::rect("Box (Medium)", 64.0, 64.0, Color::rgb(0.2, 0.8, 0.8))
                    .with_category("Primitives")
            );
            self.editor.register_object(
                MapObject::rect("Box (Large)", 128.0, 64.0, Color::rgb(0.8, 0.2, 0.8))
                    .with_category("Primitives")
            );
            
            // Common game objects
            self.editor.register_object(
                MapObject::circle("Player Spawn", 20.0, Color::rgb(0.2, 1.0, 0.4))
                    .with_category("Spawns")
            );
            self.editor.register_object(
                MapObject::circle("Enemy Spawn", 20.0, Color::rgb(1.0, 0.2, 0.2))
                    .with_category("Spawns")
            );
            self.editor.register_object(
                MapObject::rect("Platform", 160.0, 24.0, Color::rgb(0.5, 0.4, 0.3))
                    .with_category("Environment")
            );
            self.editor.register_object(
                MapObject::rect("Wall", 32.0, 128.0, Color::rgb(0.4, 0.4, 0.4))
                    .with_category("Environment")
            );
        }
    }

    fn update(&mut self, _world: &mut World, _time: &Time) {
        // Plugin update - handled via handle_input instead
    }

    fn render(&self, _world: &World, draw: &mut DrawContext) {
        self.render_editor(draw);
    }

    #[cfg(feature = "gui")]
    fn gui(&self, ctx: &egui::Context, _fps: &FpsCounter) {
        if !self.editor.is_enabled() {
            return;
        }

        // We can't mutate self in gui(), so we use interior mutability
        // For now, just render a status indicator
        egui::TopBottomPanel::bottom("editor_status_bar")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🗺 Map Editor");
                    ui.separator();
                    ui.label(format!("Tool: {}", self.editor.tool.tool.name()));
                    ui.separator();
                    ui.label(format!(
                        "Grid: {} | Snap: {}",
                        if self.editor.config.grid.visible { "ON" } else { "OFF" },
                        if self.editor.config.grid.enabled { "ON" } else { "OFF" },
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Mouse: ({:.0}, {:.0})",
                        self.editor.state.mouse_world_pos.x,
                        self.editor.state.mouse_world_pos.y,
                    ));
                    ui.separator();
                    ui.label(format!("Objects: {}", self.editor.objects().len()));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("F1/` to toggle | V:Select P:Place E:Erase B:Paint G:Grid");
                    });
                });
            });
    }
}
