//! Asset palette for browsing and selecting objects to place.

use std::collections::HashMap;

use super::map_object::MapObject;

/// Entry in the asset palette.
#[derive(Debug, Clone)]
pub struct AssetEntry {
    /// The map object definition.
    pub object: MapObject,
    /// Optional preview image data (for GUI thumbnail).
    pub preview_texture_id: Option<egui::TextureId>,
}

impl AssetEntry {
    /// Create a new asset entry.
    pub fn new(object: MapObject) -> Self {
        Self {
            object,
            preview_texture_id: None,
        }
    }

    /// Set the preview texture.
    pub fn with_preview(mut self, texture_id: egui::TextureId) -> Self {
        self.preview_texture_id = Some(texture_id);
        self
    }
}

/// Asset palette for the editor.
///
/// Organizes assets by category and provides search/filter functionality.
#[derive(Default)]
pub struct AssetPalette {
    /// All registered assets.
    assets: HashMap<String, AssetEntry>,
    /// Assets organized by category.
    categories: HashMap<String, Vec<String>>,
    /// Currently selected asset (for placement).
    selected_asset: Option<String>,
    /// Search filter text.
    search_filter: String,
    /// Currently expanded categories.
    expanded_categories: HashMap<String, bool>,
}

/// Kinds of asset visuals you can convert to via the context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Texture,
    SpriteSheet,
    Tileset,
    NineSlice,
    Circle,
    Rect,
}

impl AssetPalette {
    /// Create a new empty asset palette.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new asset.
    pub fn register(&mut self, object: MapObject) {
        let name = object.name.clone();
        let category = object.category.clone();
        let is_first = self.assets.is_empty();

        self.assets.insert(name.clone(), AssetEntry::new(object));

        self.categories
            .entry(category.clone())
            .or_default()
            .push(name.clone());

        // Expand new categories by default
        self.expanded_categories.entry(category).or_insert(true);
        
        // Auto-select first asset
        if is_first {
            self.selected_asset = Some(name);
        }
    }

    /// Register an asset with a preview texture.
    pub fn register_with_preview(&mut self, object: MapObject, preview_id: egui::TextureId) {
        let name = object.name.clone();
        let category = object.category.clone();

        self.assets
            .insert(name.clone(), AssetEntry::new(object).with_preview(preview_id));

        self.categories
            .entry(category.clone())
            .or_default()
            .push(name);

        self.expanded_categories.entry(category).or_insert(true);
    }

    /// Get an asset by name.
    pub fn get(&self, name: &str) -> Option<&AssetEntry> {
        self.assets.get(name)
    }

    /// Get the map object definition by name.
    pub fn get_object(&self, name: &str) -> Option<&MapObject> {
        self.assets.get(name).map(|e| &e.object)
    }

    /// Get the currently selected asset.
    pub fn selected(&self) -> Option<&str> {
        self.selected_asset.as_deref()
    }

    /// Get the currently selected asset's object definition.
    pub fn selected_object(&self) -> Option<&MapObject> {
        self.selected_asset
            .as_ref()
            .and_then(|name| self.get_object(name))
    }

    /// Check if the palette is empty.
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Get the number of registered assets.
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Select an asset for placement.
    pub fn select(&mut self, name: impl Into<String>) {
        self.selected_asset = Some(name.into());
    }

    /// Clear the selection.
    pub fn deselect(&mut self) {
        self.selected_asset = None;
    }

    /// Get all category names.
    pub fn categories(&self) -> impl Iterator<Item = &String> {
        self.categories.keys()
    }

    /// Get assets in a category.
    pub fn assets_in_category(&self, category: &str) -> Option<&Vec<String>> {
        self.categories.get(category)
    }

    /// Set the search filter.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.search_filter = filter.into().to_lowercase();
    }

    /// Clear the search filter.
    pub fn clear_filter(&mut self) {
        self.search_filter.clear();
    }

    /// Check if an asset matches the current filter.
    pub fn matches_filter(&self, name: &str) -> bool {
        if self.search_filter.is_empty() {
            return true;
        }
        let entry = match self.assets.get(name) {
            Some(e) => e,
            None => return false,
        };
        entry.object.name.to_lowercase().contains(&self.search_filter)
            || entry
                .object
                .display_name
                .to_lowercase()
                .contains(&self.search_filter)
            || entry
                .object
                .category
                .to_lowercase()
                .contains(&self.search_filter)
    }

    /// Toggle a category's expanded state.
    pub fn toggle_category(&mut self, category: &str) {
        if let Some(expanded) = self.expanded_categories.get_mut(category) {
            *expanded = !*expanded;
        }
    }

    /// Check if a category is expanded.
    pub fn is_category_expanded(&self, category: &str) -> bool {
        self.expanded_categories.get(category).copied().unwrap_or(true)
    }

    /// Get the search filter text (for UI).
    pub fn search_filter(&self) -> &str {
        &self.search_filter
    }

    /// Get mutable reference to search filter (for UI).
    pub fn search_filter_mut(&mut self) -> &mut String {
        &mut self.search_filter
    }

    /// Convert an asset to a different visual type.
    pub fn convert_asset_to(&mut self, name: &str, kind: AssetKind) {
        use super::map_object::ObjectVisual;
        use crate::core::color::Color;

        if let Some(entry) = self.assets.get_mut(name) {
            let obj = &mut entry.object;
            match kind {
                AssetKind::Texture => {
                    let texture_name = obj.name.clone();
                    obj.visual = ObjectVisual::Texture { texture_name, tint: Color::WHITE };
                    // Keep size unchanged
                }
                AssetKind::SpriteSheet => {
                    let texture_name = obj.name.clone();
                    obj.visual = ObjectVisual::SpriteSheet { texture_name, frame_count: 1, current_frame: 0, fps: 10.0, tint: Color::WHITE };
                }
                AssetKind::Tileset => {
                    let texture_name = obj.name.clone();
                    obj.visual = ObjectVisual::Tileset { texture_name, columns: 1, rows: 1, selected_tile: 0, ignored_tiles: Vec::new(), tint: Color::WHITE };
                }
                AssetKind::NineSlice => {
                    let texture_name = obj.name.clone();
                    obj.visual = ObjectVisual::NineSlice { texture_name, left: 8, right: 8, top: 8, bottom: 8, tint: Color::WHITE };
                }
                AssetKind::Circle => {
                    let radius = (obj.size.x.max(obj.size.y)) * 0.5;
                    obj.visual = ObjectVisual::Circle { radius, color: Color::WHITE };
                    obj.size = crate::core::math::Vec2::new(radius * 2.0, radius * 2.0);
                }
                AssetKind::Rect => {
                    obj.visual = ObjectVisual::Rect { color: Color::WHITE };
                    // size stays as-is
                }
            }
        }
    }



    /// Render the palette UI.
    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            let response = ui.text_edit_singleline(&mut self.search_filter);
            if response.changed() {
                self.search_filter = self.search_filter.to_lowercase();
            }
            if ui.button("✖").clicked() {
                self.search_filter.clear();
            }
        });

        ui.separator();
        
        // Show selected asset preview at top
        if let Some(selected_name) = self.selected_asset.clone() {
            if let Some(entry) = self.assets.get(&selected_name) {
                let display_name = entry.object.display_name.clone();
                let size = entry.object.size;
                let visual = entry.object.visual.clone();
                let obj_size = entry.object.size;
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Draw preview
                        Self::draw_preview_static(ui, &visual, obj_size, 48.0);
                        ui.vertical(|ui| {
                            ui.strong(&display_name);
                            ui.label(format!("{}x{}", size.x as i32, size.y as i32));
                        });
                    });
                });
                ui.separator();
            }
        }

        // Collect category data to avoid borrow issues
        let categories: Vec<String> = self.categories.keys().cloned().collect();
        let mut clicked_asset: Option<String> = None;
        let mut toggled_category: Option<String> = None;
        // Deferred conversion request set via context menu
        let mut conversion_request: Option<(String, AssetKind)> = None;
        let search_filter = self.search_filter.clone();

        // Category tree
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for category in &categories {
                    let expanded = self.expanded_categories.get(category).copied().unwrap_or(true);
                    let assets_in_cat: Vec<String> = self.categories
                        .get(category)
                        .map(|v| v.clone())
                        .unwrap_or_default();
                    
                    let header = egui::CollapsingHeader::new(category)
                        .default_open(expanded)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for asset_name in &assets_in_cat {
                                    // Check filter
                                    if !search_filter.is_empty() {
                                        let matches = if let Some(entry) = self.assets.get(asset_name) {
                                            entry.object.name.to_lowercase().contains(&search_filter)
                                                || entry.object.display_name.to_lowercase().contains(&search_filter)
                                        } else {
                                            false
                                        };
                                        if !matches {
                                            continue;
                                        }
                                    }

                                    let (visual, obj_size, display_name, prop_count) = {
                                        let entry = match self.assets.get(asset_name) {
                                            Some(e) => e,
                                            None => continue,
                                        };
                                        (
                                            entry.object.visual.clone(),
                                            entry.object.size,
                                            entry.object.display_name.clone(),
                                            entry.object.properties.len(),
                                        )
                                    };

                                    let is_selected = self.selected_asset.as_ref() == Some(asset_name);

                                    // Create a button with preview
                                    let response = Self::asset_button_static(ui, &visual, obj_size, is_selected);

                                    if response.clicked() {
                                        clicked_asset = Some(asset_name.clone());
                                    }

                                    // Context menu for conversion / extra options
                                    response.context_menu(|ui| {
                                        if ui.button("Convert → Texture").clicked() {
                                            ui.close();
                                            conversion_request = Some((asset_name.clone(), AssetKind::Texture));
                                        }
                                        if ui.button("Convert → SpriteSheet").clicked() {
                                            ui.close();
                                            conversion_request = Some((asset_name.clone(), AssetKind::SpriteSheet));
                                        }
                                        if ui.button("Convert → Tileset").clicked() {
                                            ui.close();
                                            conversion_request = Some((asset_name.clone(), AssetKind::Tileset));
                                        }
                                        if ui.button("Convert → Circle").clicked() {
                                            ui.close();
                                            conversion_request = Some((asset_name.clone(), AssetKind::Circle));
                                        }
                                        if ui.button("Convert → Rect").clicked() {
                                            ui.close();
                                            conversion_request = Some((asset_name.clone(), AssetKind::Rect));
                                        }
                                        if ui.button("Convert → 9-Slice").clicked() {
                                            ui.close();
                                            conversion_request = Some((asset_name.clone(), AssetKind::NineSlice));
                                        }
                                    });

                                    // Tooltip with more info
                                    response.on_hover_ui(|ui| {
                                        ui.strong(&display_name);
                                        ui.label(format!("Size: {}x{}", obj_size.x as i32, obj_size.y as i32));
                                        if prop_count > 0 {
                                            ui.label(format!("Properties: {}", prop_count));
                                        }
                                    });
                                }
                            });
                        });

                    // Track expand/collapse
                    if header.header_response.clicked() {
                        toggled_category = Some(category.clone());
                    }
                }
            });
        
        // Apply deferred mutations
        if let Some(name) = clicked_asset {
            self.selected_asset = Some(name);
        }
        if let Some(cat) = toggled_category {
            if let Some(expanded) = self.expanded_categories.get_mut(&cat) {
                *expanded = !*expanded;
            }
        }

        // Handle conversion requests (from context menu)
        if let Some((name, kind)) = conversion_request {
            self.convert_asset_to(&name, kind);
        }
    }
    
    /// Draw a preview of the asset (static version to avoid borrow issues).
    #[cfg(feature = "gui")]
    fn draw_preview_static(ui: &mut egui::Ui, visual: &super::map_object::ObjectVisual, obj_size: crate::core::math::Vec2, size: f32) {
        use super::map_object::ObjectVisual;
        
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(size, size),
            egui::Sense::hover(),
        );
        
        let painter = ui.painter();
        let center = rect.center();
        
        match visual {
            ObjectVisual::Circle { radius, color } => {
                let display_radius = (size * 0.4).min(*radius);
                painter.circle_filled(
                    center,
                    display_radius,
                    egui::Color32::from_rgba_unmultiplied(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ),
                );
            }
            ObjectVisual::Rect { color } => {
                let aspect = obj_size.x / obj_size.y.max(0.001);
                let (w, h) = if aspect > 1.0 {
                    (size * 0.8, size * 0.8 / aspect)
                } else {
                    (size * 0.8 * aspect, size * 0.8)
                };
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(w.max(8.0), h.max(8.0)),
                );
                painter.rect_filled(
                    preview_rect,
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ),
                );
            }
            ObjectVisual::Texture { tint, .. } => {
                // Draw a placeholder with texture icon
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 200.0) as u8,
                        (tint.g * 200.0) as u8,
                        (tint.b * 200.0) as u8,
                        180,
                    ),
                );
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "🖼",
                    egui::FontId::proportional(size * 0.4),
                    egui::Color32::WHITE,
                );
            }
            ObjectVisual::Line { color, thickness } => {
                let half = size * 0.35;
                painter.line_segment(
                    [center - egui::vec2(half, half), center + egui::vec2(half, half)],
                    egui::Stroke::new(*thickness, egui::Color32::from_rgba_unmultiplied(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    )),
                );
            }
            ObjectVisual::SpriteSheet { tint, frame_count, .. } => {
                // Draw a placeholder with film strip icon for animations
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 180.0) as u8,
                        (tint.g * 150.0) as u8,
                        (tint.b * 220.0) as u8,
                        180,
                    ),
                );
                // Draw frame count indicator
                painter.text(
                    center - egui::vec2(0.0, size * 0.15),
                    egui::Align2::CENTER_CENTER,
                    "🎬",
                    egui::FontId::proportional(size * 0.3),
                    egui::Color32::WHITE,
                );
                painter.text(
                    center + egui::vec2(0.0, size * 0.15),
                    egui::Align2::CENTER_CENTER,
                    format!("{}f", frame_count),
                    egui::FontId::proportional(size * 0.2),
                    egui::Color32::WHITE,
                );
            }
            ObjectVisual::Tileset { tint, columns, rows, .. } => {
                // Draw a grid icon for tilesets
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 150.0) as u8,
                        (tint.g * 200.0) as u8,
                        (tint.b * 150.0) as u8,
                        180,
                    ),
                );
                // Draw mini grid
                let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100));
                let cell_w = preview_rect.width() / (*columns).min(4) as f32;
                let cell_h = preview_rect.height() / (*rows).min(4) as f32;
                for i in 0..=(*columns).min(4) {
                    let x = preview_rect.left() + cell_w * i as f32;
                    painter.line_segment([egui::pos2(x, preview_rect.top()), egui::pos2(x, preview_rect.bottom())], grid_stroke);
                }
                for i in 0..=(*rows).min(4) {
                    let y = preview_rect.top() + cell_h * i as f32;
                    painter.line_segment([egui::pos2(preview_rect.left(), y), egui::pos2(preview_rect.right(), y)], grid_stroke);
                }
                // Tile count label
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    format!("{}t", columns * rows),
                    egui::FontId::proportional(size * 0.2),
                    egui::Color32::WHITE,
                );
            }
            ObjectVisual::NineSlice { tint, .. } => {
                // Draw a 9-slice icon showing the 3x3 grid pattern
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 180.0) as u8,
                        (tint.g * 180.0) as u8,
                        (tint.b * 220.0) as u8,
                        180,
                    ),
                );
                // Draw 9-slice guides (2 vertical, 2 horizontal lines)
                let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150));
                let third_w = preview_rect.width() / 3.0;
                let third_h = preview_rect.height() / 3.0;
                // Vertical lines
                painter.line_segment([
                    egui::pos2(preview_rect.left() + third_w, preview_rect.top()),
                    egui::pos2(preview_rect.left() + third_w, preview_rect.bottom())
                ], grid_stroke);
                painter.line_segment([
                    egui::pos2(preview_rect.right() - third_w, preview_rect.top()),
                    egui::pos2(preview_rect.right() - third_w, preview_rect.bottom())
                ], grid_stroke);
                // Horizontal lines
                painter.line_segment([
                    egui::pos2(preview_rect.left(), preview_rect.top() + third_h),
                    egui::pos2(preview_rect.right(), preview_rect.top() + third_h)
                ], grid_stroke);
                painter.line_segment([
                    egui::pos2(preview_rect.left(), preview_rect.bottom() - third_h),
                    egui::pos2(preview_rect.right(), preview_rect.bottom() - third_h)
                ], grid_stroke);
                // Label
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "9",
                    egui::FontId::proportional(size * 0.3),
                    egui::Color32::WHITE,
                );
            }
        }
    }
    
    /// Create a button for an asset with preview (static version).
    #[cfg(feature = "gui")]
    fn asset_button_static(ui: &mut egui::Ui, visual: &super::map_object::ObjectVisual, obj_size: crate::core::math::Vec2, selected: bool) -> egui::Response {
        let button_size = 56.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(button_size, button_size),
            egui::Sense::click(),
        );
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Background
            let bg_color = if selected {
                egui::Color32::from_rgb(60, 100, 160)
            } else if response.hovered() {
                egui::Color32::from_rgb(50, 50, 60)
            } else {
                egui::Color32::from_rgb(35, 35, 40)
            };
            
            painter.rect_filled(rect, 4.0, bg_color);
            
            if selected {
                painter.rect_stroke(rect, 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 160, 255)), egui::StrokeKind::Inside);
            }
            
            // Draw preview centered
            Self::draw_preview_at_rect(painter, rect.shrink(4.0), visual, obj_size);
        }
        
        response
    }
    
    /// Draw asset preview at a specific rect.
    #[cfg(feature = "gui")]
    fn draw_preview_at_rect(painter: &egui::Painter, rect: egui::Rect, visual: &super::map_object::ObjectVisual, obj_size: crate::core::math::Vec2) {
        use super::map_object::ObjectVisual;
        
        let center = rect.center();
        let size = rect.width().min(rect.height());
        
        match visual {
            ObjectVisual::Circle { radius, color } => {
                let display_radius = (size * 0.4).min(*radius);
                painter.circle_filled(
                    center,
                    display_radius,
                    egui::Color32::from_rgba_unmultiplied(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ),
                );
            }
            ObjectVisual::Rect { color } => {
                let aspect = obj_size.x / obj_size.y.max(0.001);
                let (w, h) = if aspect > 1.0 {
                    (size * 0.8, size * 0.8 / aspect)
                } else {
                    (size * 0.8 * aspect, size * 0.8)
                };
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(w.max(8.0), h.max(8.0)),
                );
                painter.rect_filled(
                    preview_rect,
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ),
                );
            }
            ObjectVisual::Texture { tint, .. } => {
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 200.0) as u8,
                        (tint.g * 200.0) as u8,
                        (tint.b * 200.0) as u8,
                        180,
                    ),
                );
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "🖼",
                    egui::FontId::proportional(size * 0.4),
                    egui::Color32::WHITE,
                );
            }
            ObjectVisual::Line { color, thickness } => {
                let half = size * 0.35;
                painter.line_segment(
                    [center - egui::vec2(half, half), center + egui::vec2(half, half)],
                    egui::Stroke::new(*thickness, egui::Color32::from_rgba_unmultiplied(
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    )),
                );
            }
            ObjectVisual::SpriteSheet { tint, frame_count, .. } => {
                // Draw a placeholder with film strip icon for animations
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 180.0) as u8,
                        (tint.g * 150.0) as u8,
                        (tint.b * 220.0) as u8,
                        180,
                    ),
                );
                // Draw frame count indicator
                painter.text(
                    center - egui::vec2(0.0, size * 0.15),
                    egui::Align2::CENTER_CENTER,
                    "🎬",
                    egui::FontId::proportional(size * 0.3),
                    egui::Color32::WHITE,
                );
                painter.text(
                    center + egui::vec2(0.0, size * 0.15),
                    egui::Align2::CENTER_CENTER,
                    format!("{}f", frame_count),
                    egui::FontId::proportional(size * 0.2),
                    egui::Color32::WHITE,
                );
            }
            ObjectVisual::Tileset { tint, columns, rows, .. } => {
                // Draw a grid icon for tilesets
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 150.0) as u8,
                        (tint.g * 200.0) as u8,
                        (tint.b * 150.0) as u8,
                        180,
                    ),
                );
                // Draw mini grid
                let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100));
                let cell_w = preview_rect.width() / (*columns).min(4) as f32;
                let cell_h = preview_rect.height() / (*rows).min(4) as f32;
                for i in 0..=(*columns).min(4) {
                    let x = preview_rect.left() + cell_w * i as f32;
                    painter.line_segment([egui::pos2(x, preview_rect.top()), egui::pos2(x, preview_rect.bottom())], grid_stroke);
                }
                for i in 0..=(*rows).min(4) {
                    let y = preview_rect.top() + cell_h * i as f32;
                    painter.line_segment([egui::pos2(preview_rect.left(), y), egui::pos2(preview_rect.right(), y)], grid_stroke);
                }
                // Tile count label
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    format!("{}t", columns * rows),
                    egui::FontId::proportional(size * 0.2),
                    egui::Color32::WHITE,
                );
            }
            ObjectVisual::NineSlice { tint, .. } => {
                // Draw a 9-slice icon showing the 3x3 grid pattern
                let preview_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(size * 0.7, size * 0.7),
                );
                painter.rect_filled(
                    preview_rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(
                        (tint.r * 180.0) as u8,
                        (tint.g * 180.0) as u8,
                        (tint.b * 220.0) as u8,
                        180,
                    ),
                );
                // Draw 9-slice guides (2 vertical, 2 horizontal lines)
                let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150));
                let third_w = preview_rect.width() / 3.0;
                let third_h = preview_rect.height() / 3.0;
                // Vertical lines
                painter.line_segment([
                    egui::pos2(preview_rect.left() + third_w, preview_rect.top()),
                    egui::pos2(preview_rect.left() + third_w, preview_rect.bottom())
                ], grid_stroke);
                painter.line_segment([
                    egui::pos2(preview_rect.right() - third_w, preview_rect.top()),
                    egui::pos2(preview_rect.right() - third_w, preview_rect.bottom())
                ], grid_stroke);
                // Horizontal lines
                painter.line_segment([
                    egui::pos2(preview_rect.left(), preview_rect.top() + third_h),
                    egui::pos2(preview_rect.right(), preview_rect.top() + third_h)
                ], grid_stroke);
                painter.line_segment([
                    egui::pos2(preview_rect.left(), preview_rect.bottom() - third_h),
                    egui::pos2(preview_rect.right(), preview_rect.bottom() - third_h)
                ], grid_stroke);
                // Label
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "9",
                    egui::FontId::proportional(size * 0.3),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}
