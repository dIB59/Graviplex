//! Map Editor Demo
//!
//! This example demonstrates the in-game map editor plugin for placing assets.
//!
//! Controls:
//! - F1 or `: Toggle editor on/off
//! - V: Select tool
//! - P: Place tool
//! - E: Erase tool
//! - B: Paint (brush) tool
//! - G: Toggle grid snapping
//! - Delete/Backspace: Delete selected objects
//! - Ctrl+Z: Undo
//! - Ctrl+Shift+Z: Redo
//! - Ctrl+A: Select all

use graviplex::prelude::*;
use std::collections::HashSet;

#[cfg(feature = "gui")]
use graviplex::editor::{MapEditorPlugin, EditorConfig, GridConfig, MapObject};

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("This example requires the 'gui' feature. Run with:");
    eprintln!("  cargo run --example map_editor_demo --features gui");
}

#[cfg(feature = "gui")]
fn main() {
    // Build atlas with sprites from assets folder
    #[cfg(feature = "textures")]
    let atlas = build_atlas_from_assets();
    
    let mut app = App::build(EditorDemo::new())
        .title("Map Editor Demo - Press F1 or ` to toggle editor")
        .size(1280, 720);
    
    #[cfg(feature = "textures")]
    {
        // Use 4096 atlas - should be enough for filtered textures
        app = app.atlas(atlas).atlas_size(4096);
    }
    
    app.run().unwrap();
}

#[cfg(all(feature = "gui", feature = "textures"))]
fn build_atlas_from_assets() -> graviplex::prelude::AtlasBuilder {
    use graviplex::prelude::AtlasBuilder;
    use image::GenericImageView;
    use std::fs;
    use std::path::Path;
    
    // Recursively collect all image paths
    fn collect_images(path: &Path, images: &mut Vec<(String, std::path::PathBuf)>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    collect_images(&entry_path, images);
                } else if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp") {
                        // Use the full path as the texture name (same as plugin does)
                        let texture_name = entry_path.to_string_lossy().to_string();
                        images.push((texture_name, entry_path));
                    }
                }
            }
        }
    }
    
    let mut images = Vec::new();
    collect_images(Path::new("assets"), &mut images);
    
    // Filter to only valid images and collect their sizes
    // Skip large sprite sheets that won't fit in atlas
    let max_texture_size = 1024; // Skip textures larger than this (likely sprite sheets)
    let valid_images: Vec<_> = images.into_iter()
        .filter_map(|(name, path)| {
            // Try to open the image to verify it's valid
            match image::open(&path) {
                Ok(img) => {
                    let (w, h) = img.dimensions();
                    // Skip very large textures (likely sprite sheets)
                    if w > max_texture_size || h > max_texture_size {
                        return None;
                    }
                    Some((name, path))
                }
                Err(e) => {
                    eprintln!("Skipping invalid texture {}: {}", name, e);
                    None
                }
            }
        })
        .collect();
    
    println!("[Atlas] Loading {} textures (filtered to max {}x{})...", valid_images.len(), max_texture_size, max_texture_size);
    
    // Build the atlas with all valid images
    // Since we pre-validated with image::open(), add_image should never fail
    let mut builder = AtlasBuilder::new();
    for (name, path) in valid_images {
        builder = builder.add_image(&name, &path)
            .expect("Pre-validated image should load successfully");
    }
    
    builder
}

/// Asset type selection for the configuration dialog
#[cfg(feature = "gui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetType {
    SingleTexture,
    SpriteSheet,
    Tileset,
    NineSlice,
}

#[cfg(feature = "gui")]
struct EditorDemo {
    editor_plugin: MapEditorPlugin,
    /// Index of current sprite sheet being configured in dialog
    sprite_sheet_dialog_index: Option<usize>,
    /// Selected asset type in dialog
    dialog_asset_type: AssetType,
    /// Frame count value for sprite sheets
    sprite_sheet_frame_count: u32,
    /// Tileset columns
    tileset_columns: u32,
    /// Tileset rows
    tileset_rows: u32,
    /// 9-slice border margins
    nine_slice_left: u32,
    nine_slice_right: u32,
    nine_slice_top: u32,
    nine_slice_bottom: u32,
    /// Cached preview texture for the dialog
    preview_texture: Option<egui::TextureHandle>,
    /// Path of the currently loaded preview texture
    preview_texture_path: Option<String>,
    /// Set of ignored tile indices for tileset configuration
    tileset_ignored_tiles: HashSet<u32>,
}

#[cfg(feature = "gui")]
impl EditorDemo {
    fn new() -> Self {
        // Configure the editor
        let config = EditorConfig {
            enabled: true, // Start with editor enabled
            show_panel: true,
            panel_width: 280.0,
            grid: GridConfig {
                enabled: true,
                visible: true,
                cell_size: Vec2::new(32.0, 32.0),
                ..Default::default()
            },
            ..Default::default()
        };

        // Create plugin with asset directory scanning
        let editor_plugin = MapEditorPlugin::with_config(config)
            .with_asset_dir("assets")
            .enabled();

        Self { 
            editor_plugin,
            sprite_sheet_dialog_index: None,
            dialog_asset_type: AssetType::SpriteSheet,
            sprite_sheet_frame_count: 1,
            tileset_columns: 4,
            tileset_rows: 4,
            nine_slice_left: 8,
            nine_slice_right: 8,
            nine_slice_top: 8,
            nine_slice_bottom: 8,
            preview_texture: None,
            preview_texture_path: None,
            tileset_ignored_tiles: HashSet::new(),
        }
    }
    
    /// Show dialog to configure pending sprite sheets
    fn show_sprite_sheet_dialog(&mut self, ctx: &egui::Context) {
        // Get the next pending sprite sheet to configure
        if self.editor_plugin.pending_sprite_sheets.is_empty() {
            self.sprite_sheet_dialog_index = None;
            self.preview_texture = None;
            self.preview_texture_path = None;
            return;
        }
        
        // Extract data from the first pending sprite sheet to avoid borrow conflicts
        let pending_info = {
            let pending = &self.editor_plugin.pending_sprite_sheets[0];
            (
                pending.display_name.clone(),
                pending.file_path.clone(),
                pending.width,
                pending.height,
                pending.suggested_frame_count,
            )
        };
        let (display_name, file_path, width, height, suggested_frame_count) = pending_info;
        let remaining_count = self.editor_plugin.pending_sprite_sheets.len().saturating_sub(1);
        
        // Load preview texture if needed
        if self.preview_texture_path.as_ref() != Some(&file_path) {
            // Load the image
            if let Ok(img) = image::open(&file_path) {
                let rgba = img.to_rgba8();
                let (img_width, img_height) = (rgba.width(), rgba.height());
                
                // Resize if too large for egui (max 2048 on any side)
                const MAX_TEXTURE_SIZE: u32 = 2048;
                let (final_image, size) = if img_width > MAX_TEXTURE_SIZE || img_height > MAX_TEXTURE_SIZE {
                    // Calculate scale to fit within limits
                    let scale = (MAX_TEXTURE_SIZE as f32 / img_width as f32)
                        .min(MAX_TEXTURE_SIZE as f32 / img_height as f32);
                    let new_width = (img_width as f32 * scale) as u32;
                    let new_height = (img_height as f32 * scale) as u32;
                    
                    let resized = image::imageops::resize(
                        &rgba,
                        new_width,
                        new_height,
                        image::imageops::FilterType::Triangle,
                    );
                    let size = [resized.width() as usize, resized.height() as usize];
                    (resized.into_raw(), size)
                } else {
                    let size = [img_width as usize, img_height as usize];
                    (rgba.into_raw(), size)
                };
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &final_image);
                self.preview_texture = Some(ctx.load_texture(
                    &file_path,
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                self.preview_texture_path = Some(file_path.clone());
            }
        }
        
        // Initialize suggestions if dialog just opened
        if self.sprite_sheet_dialog_index.is_none() {
            self.sprite_sheet_dialog_index = Some(0);
            self.sprite_sheet_frame_count = suggested_frame_count;
            self.dialog_asset_type = AssetType::SpriteSheet;
            self.tileset_ignored_tiles.clear();
            // Guess tileset dimensions based on aspect ratio
            let aspect = width as f32 / height as f32;
            if aspect > 0.8 && aspect < 1.2 {
                // Square-ish image - likely a tileset
                self.tileset_columns = 4;
                self.tileset_rows = 4;
            } else if aspect > 1.0 {
                self.tileset_columns = (aspect.round() as u32).max(2);
                self.tileset_rows = 1;
            } else {
                self.tileset_columns = 1;
                self.tileset_rows = (1.0 / aspect).round() as u32;
            }
        }
        
        let title = format!("🎞️ Configure: {}", display_name);
        let mut close_dialog = false;
        let mut skip = false;
        
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(350.0);
                
                ui.heading("Configure Asset Type");
                ui.add_space(8.0);
                
                // Show image info
                ui.horizontal(|ui| {
                    ui.label("📄 File:");
                    ui.monospace(&file_path);
                });
                ui.horizontal(|ui| {
                    ui.label("📐 Size:");
                    ui.label(format!("{}×{} pixels", width, height));
                });
                
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                // Asset type selection with radio buttons
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.selectable_value(&mut self.dialog_asset_type, AssetType::SingleTexture, "📷 Single");
                    ui.selectable_value(&mut self.dialog_asset_type, AssetType::SpriteSheet, "🎬 Animation");
                    ui.selectable_value(&mut self.dialog_asset_type, AssetType::Tileset, "🧱 Tileset");
                    ui.selectable_value(&mut self.dialog_asset_type, AssetType::NineSlice, "🖼️ 9-Slice");
                });
                
                ui.add_space(8.0);
                
                // Show options based on selected type
                match self.dialog_asset_type {
                    AssetType::SingleTexture => {
                        ui.label("Use the entire image as a single texture.");
                    }
                    AssetType::SpriteSheet => {
                        ui.horizontal(|ui| {
                            ui.label("🎬 Frames (horizontal):");
                            ui.add(egui::DragValue::new(&mut self.sprite_sheet_frame_count)
                                .range(1..=64)
                                .speed(0.1));
                        });
                        
                        if self.sprite_sheet_frame_count > 0 {
                            let frame_width = width / self.sprite_sheet_frame_count;
                            ui.label(format!("Frame size: {}×{} px", frame_width, height));
                            
                            if width % self.sprite_sheet_frame_count != 0 {
                                ui.colored_label(egui::Color32::YELLOW, 
                                    "⚠️ Width doesn't divide evenly");
                            }
                        }
                    }
                    AssetType::Tileset => {
                        ui.horizontal(|ui| {
                            ui.label("Columns:");
                            ui.add(egui::DragValue::new(&mut self.tileset_columns)
                                .range(1..=32)
                                .speed(0.1));
                            ui.label("Rows:");
                            ui.add(egui::DragValue::new(&mut self.tileset_rows)
                                .range(1..=32)
                                .speed(0.1));
                        });
                        
                        let tile_width = width / self.tileset_columns;
                        let tile_height = height / self.tileset_rows;
                        let tile_count = self.tileset_columns * self.tileset_rows;
                        ui.label(format!("Tile size: {}×{} px ({} tiles)", tile_width, tile_height, tile_count));
                        
                        if width % self.tileset_columns != 0 || height % self.tileset_rows != 0 {
                            ui.colored_label(egui::Color32::YELLOW, 
                                "⚠️ Dimensions don't divide evenly");
                        }
                    }
                    AssetType::NineSlice => {
                        ui.label("9-slice UI element - corners stay fixed, edges stretch.");
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Left:");
                            ui.add(egui::DragValue::new(&mut self.nine_slice_left)
                                .range(1..=width/2)
                                .speed(0.5));
                            ui.label("Right:");
                            ui.add(egui::DragValue::new(&mut self.nine_slice_right)
                                .range(1..=width/2)
                                .speed(0.5));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Top:");
                            ui.add(egui::DragValue::new(&mut self.nine_slice_top)
                                .range(1..=height/2)
                                .speed(0.5));
                            ui.label("Bottom:");
                            ui.add(egui::DragValue::new(&mut self.nine_slice_bottom)
                                .range(1..=height/2)
                                .speed(0.5));
                        });
                        
                        // Show warnings if margins are too large
                        if self.nine_slice_left + self.nine_slice_right >= width {
                            ui.colored_label(egui::Color32::RED, 
                                "⚠️ Left + Right margins exceed width");
                        }
                        if self.nine_slice_top + self.nine_slice_bottom >= height {
                            ui.colored_label(egui::Color32::RED, 
                                "⚠️ Top + Bottom margins exceed height");
                        }
                    }
                }
                
                ui.add_space(12.0);
                
                // Visual preview
                ui.separator();
                ui.label("Preview:");
                
                let preview_max_size = 250.0;
                let scale = (preview_max_size / width as f32).min(preview_max_size / height as f32);
                let preview_width = width as f32 * scale;
                let preview_height = height as f32 * scale;
                
                // Use click sense for tileset, hover for others
                let sense = if self.dialog_asset_type == AssetType::Tileset {
                    egui::Sense::click()
                } else {
                    egui::Sense::hover()
                };
                
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(preview_width, preview_height),
                    sense,
                );
                
                // Handle tile clicking for tileset mode
                if self.dialog_asset_type == AssetType::Tileset {
                    if response.clicked() {
                        if let Some(pointer_pos) = response.interact_pointer_pos() {
                            let tile_width = preview_width / self.tileset_columns as f32;
                            let tile_height = preview_height / self.tileset_rows as f32;
                            
                            let rel_x = pointer_pos.x - rect.left();
                            let rel_y = pointer_pos.y - rect.top();
                            
                            let col = (rel_x / tile_width).floor() as u32;
                            let row = (rel_y / tile_height).floor() as u32;
                            
                            if col < self.tileset_columns && row < self.tileset_rows {
                                let tile_idx = row * self.tileset_columns + col;
                                // Toggle ignored status
                                if self.tileset_ignored_tiles.contains(&tile_idx) {
                                    self.tileset_ignored_tiles.remove(&tile_idx);
                                } else {
                                    self.tileset_ignored_tiles.insert(tile_idx);
                                }
                            }
                        }
                    }
                }
                
                if ui.is_rect_visible(rect) {
                    let painter = ui.painter();
                    
                    // Draw the actual image if loaded
                    if let Some(tex) = &self.preview_texture {
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        painter.image(tex.id(), rect, uv, egui::Color32::WHITE);
                    } else {
                        // Fallback background if image not loaded
                        painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 50));
                    }
                    
                    // Draw grid lines based on selected type
                    let line_color = egui::Color32::from_rgb(255, 100, 100);
                    let stroke = egui::Stroke::new(2.0, line_color);
                    
                    match self.dialog_asset_type {
                        AssetType::SingleTexture => {
                            // Just draw a border
                            painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
                        }
                        AssetType::SpriteSheet => {
                            // Draw border
                            painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
                            
                            // Draw vertical lines for frames
                            let frame_width = preview_width / self.sprite_sheet_frame_count as f32;
                            for i in 1..self.sprite_sheet_frame_count {
                                let x = rect.left() + frame_width * i as f32;
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    stroke,
                                );
                            }
                            
                            // Show frame numbers
                            for i in 0..self.sprite_sheet_frame_count.min(10) {
                                let x = rect.left() + frame_width * (i as f32 + 0.5);
                                painter.text(
                                    egui::pos2(x, rect.center().y),
                                    egui::Align2::CENTER_CENTER,
                                    format!("{}", i),
                                    egui::FontId::proportional(12.0),
                                    egui::Color32::WHITE,
                                );
                            }
                        }
                        AssetType::Tileset => {
                            // Draw border
                            painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
                            
                            // Draw grid for tileset
                            let tile_width = preview_width / self.tileset_columns as f32;
                            let tile_height = preview_height / self.tileset_rows as f32;
                            
                            // Draw ignored tile overlays
                            for &tile_idx in &self.tileset_ignored_tiles {
                                let col = tile_idx % self.tileset_columns;
                                let row = tile_idx / self.tileset_columns;
                                if row < self.tileset_rows {
                                    let tile_rect = egui::Rect::from_min_size(
                                        egui::pos2(
                                            rect.left() + col as f32 * tile_width,
                                            rect.top() + row as f32 * tile_height,
                                        ),
                                        egui::vec2(tile_width, tile_height),
                                    );
                                    // Semi-transparent red overlay for ignored tiles
                                    painter.rect_filled(tile_rect, 0.0, egui::Color32::from_rgba_unmultiplied(200, 50, 50, 150));
                                    // X mark
                                    let x_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
                                    let margin = 4.0;
                                    painter.line_segment(
                                        [
                                            egui::pos2(tile_rect.left() + margin, tile_rect.top() + margin),
                                            egui::pos2(tile_rect.right() - margin, tile_rect.bottom() - margin),
                                        ],
                                        x_stroke,
                                    );
                                    painter.line_segment(
                                        [
                                            egui::pos2(tile_rect.right() - margin, tile_rect.top() + margin),
                                            egui::pos2(tile_rect.left() + margin, tile_rect.bottom() - margin),
                                        ],
                                        x_stroke,
                                    );
                                }
                            }
                            
                            // Vertical lines
                            for i in 1..self.tileset_columns {
                                let x = rect.left() + tile_width * i as f32;
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    stroke,
                                );
                            }
                            
                            // Horizontal lines
                            for i in 1..self.tileset_rows {
                                let y = rect.top() + tile_height * i as f32;
                                painter.line_segment(
                                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                                    stroke,
                                );
                            }
                        }
                        AssetType::NineSlice => {
                            // Draw border
                            painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
                            
                            // Draw 9-slice guides
                            let left_x = rect.left() + self.nine_slice_left as f32 * scale;
                            let right_x = rect.right() - self.nine_slice_right as f32 * scale;
                            let top_y = rect.top() + self.nine_slice_top as f32 * scale;
                            let bottom_y = rect.bottom() - self.nine_slice_bottom as f32 * scale;
                            
                            // Vertical lines
                            painter.line_segment(
                                [egui::pos2(left_x, rect.top()), egui::pos2(left_x, rect.bottom())],
                                stroke,
                            );
                            painter.line_segment(
                                [egui::pos2(right_x, rect.top()), egui::pos2(right_x, rect.bottom())],
                                stroke,
                            );
                            
                            // Horizontal lines
                            painter.line_segment(
                                [egui::pos2(rect.left(), top_y), egui::pos2(rect.right(), top_y)],
                                stroke,
                            );
                            painter.line_segment(
                                [egui::pos2(rect.left(), bottom_y), egui::pos2(rect.right(), bottom_y)],
                                stroke,
                            );
                            
                            // Label the 9 sections
                            let label_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180);
                            let font = egui::FontId::proportional(10.0);
                            
                            // Corner labels
                            painter.text(egui::pos2(rect.left() + 4.0, rect.top() + 4.0), 
                                egui::Align2::LEFT_TOP, "TL", font.clone(), label_color);
                            painter.text(egui::pos2(rect.right() - 4.0, rect.top() + 4.0), 
                                egui::Align2::RIGHT_TOP, "TR", font.clone(), label_color);
                            painter.text(egui::pos2(rect.left() + 4.0, rect.bottom() - 4.0), 
                                egui::Align2::LEFT_BOTTOM, "BL", font.clone(), label_color);
                            painter.text(egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0), 
                                egui::Align2::RIGHT_BOTTOM, "BR", font.clone(), label_color);
                            
                            // Center label
                            let center_x = (left_x + right_x) / 2.0;
                            let center_y = (top_y + bottom_y) / 2.0;
                            painter.text(egui::pos2(center_x, center_y), 
                                egui::Align2::CENTER_CENTER, "CENTER", font, label_color);
                        }
                    }
                }
                
                // Show instruction for tileset mode
                if self.dialog_asset_type == AssetType::Tileset {
                    ui.small("💡 Click tiles to mark as ignored/empty");
                    if !self.tileset_ignored_tiles.is_empty() {
                        ui.small(format!("Ignored: {} tiles", self.tileset_ignored_tiles.len()));
                    }
                }
                
                ui.add_space(12.0);
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("✅ Confirm").clicked() {
                        close_dialog = true;
                    }
                    
                    if ui.button("⏭️ Skip").clicked() {
                        skip = true;
                    }
                });
                
                ui.add_space(4.0);
                ui.label(format!("({} more to configure)", remaining_count));
            });
        
        // Handle dialog actions outside of the closure
        if skip {
            // Skip this asset and save that choice
            self.editor_plugin.skip_sprite_sheet(0);
            self.sprite_sheet_dialog_index = None;
        } else if close_dialog {
            match self.dialog_asset_type {
                AssetType::SingleTexture => {
                    self.editor_plugin.resolve_sprite_sheet(0, None);
                }
                AssetType::SpriteSheet => {
                    self.editor_plugin.resolve_sprite_sheet(0, Some(self.sprite_sheet_frame_count));
                }
                AssetType::Tileset => {
                    let ignored: Vec<u32> = self.tileset_ignored_tiles.iter().copied().collect();
                    self.editor_plugin.resolve_tileset(0, self.tileset_columns, self.tileset_rows, ignored);
                    self.tileset_ignored_tiles.clear();
                }
                AssetType::NineSlice => {
                    self.editor_plugin.resolve_nine_slice(
                        0, 
                        self.nine_slice_left, 
                        self.nine_slice_right, 
                        self.nine_slice_top, 
                        self.nine_slice_bottom
                    );
                }
            }
            self.sprite_sheet_dialog_index = None;
        }
    }
}

#[cfg(feature = "gui")]
impl GameLoop for EditorDemo {
    fn init(&mut self, world: &mut World, _gfx: &Graphics) {
        // Initialize the plugin (scans assets directory)
        self.editor_plugin.init(world);
        
        // You can also register custom objects programmatically
        self.editor_plugin.editor_mut().register_object(
            MapObject::circle("Custom Marker", 16.0, Color::rgb(1.0, 0.5, 0.0))
                .with_category("Custom")
        );
        
        // Place a few default objects to demonstrate
        self.editor_plugin.editor_mut().place_object("Platform", Vec2::new(0.0, -150.0));
        self.editor_plugin.editor_mut().place_object("Player Spawn", Vec2::new(-200.0, 0.0));
    }

    fn update(&mut self, _world: &mut World, _res: &Resources) {
        // The plugin handles its own state through handle_input
    }

    fn handle_input(&mut self, world: &mut World, input: &InputState, camera: &Camera2D) -> bool {
        // Let the editor plugin handle input (includes F1/` toggle)
        self.editor_plugin.handle_input(world, input, camera)
    }

    fn render(&mut self, _world: &World, draw: &mut DrawContext) {
        // Render editor overlays (grid, objects, preview)
        self.editor_plugin.render_editor(draw);
    }

    #[cfg(feature = "gui")]
    fn gui(&mut self, ctx: &egui::Context) {
        // Render the editor UI panel
        let world = World::new();
        self.editor_plugin.editor.ui(ctx, &world);
        
        // Show sprite sheet configuration dialog if needed
        self.show_sprite_sheet_dialog(ctx);
        
        // File menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("📁 File", |ui| {
                    if ui.button("💾 Save Map...").clicked() {
                        if let Err(e) = self.editor_plugin.editor_mut().save("map.json") {
                            eprintln!("Failed to save map: {}", e);
                        } else {
                            println!("Map saved to map.json");
                        }
                        ui.close();
                    }
                    if ui.button("📂 Load Map...").clicked() {
                        if let Err(e) = self.editor_plugin.editor_mut().load("map.json") {
                            eprintln!("Failed to load map: {}", e);
                        } else {
                            println!("Map loaded from map.json");
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("🗑 Clear Map").clicked() {
                        self.editor_plugin.editor_mut().clear();
                        ui.close();
                    }
                });
                
                ui.menu_button("⚙️ Assets", |ui| {
                    if ui.button("🔄 Reset Asset Config").clicked() {
                        // Delete the asset config file to re-ask about all assets
                        let _ = std::fs::remove_file("assets/.asset_config.json");
                        println!("Asset config cleared. Restart to re-configure assets.");
                        ui.close();
                    }
                    ui.separator();
                    let pending_count = self.editor_plugin.pending_sprite_sheets.len();
                    ui.label(format!("Pending: {} assets", pending_count));
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.editor_plugin.editor().is_dirty() {
                        ui.label("● Unsaved changes");
                    }
                });
            });
        });
        
        // Show help window
        egui::Window::new("📖 Help")
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .collapsible(true)
            .default_open(false)
            .show(ctx, |ui| {
                if ui.ui_contains_pointer() {
                    self.editor_plugin.editor.state.panel_focused = true;
                }
                
                ui.heading("Map Editor Controls");
                ui.separator();
                
                ui.label("🔧 Toggle Editor:");
                ui.label("  F1 or ` (backtick)");
                ui.add_space(8.0);
                
                ui.label("🛠 Tools:");
                ui.label("  V - Select");
                ui.label("  P - Place");
                ui.label("  E - Erase");
                ui.label("  B - Paint/Brush");
                ui.add_space(8.0);
                
                ui.label("📐 Grid:");
                ui.label("  G - Toggle snap");
                ui.add_space(8.0);
                
                ui.label("🖱 Mouse:");
                ui.label("  Click - Place/Select");
                ui.label("  Shift+Click - Multi-select");
                ui.label("  Drag - Move selected");
                ui.add_space(8.0);
                
                ui.label("⌨ Keyboard:");
                ui.label("  Del - Delete selected");
                ui.label("  Ctrl+Z - Undo");
                ui.label("  Ctrl+Shift+Z - Redo");
                ui.label("  Ctrl+A - Select all");
            });
    }
}

