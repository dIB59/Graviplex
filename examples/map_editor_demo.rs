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
    let max_texture_size = 512; // Skip textures larger than this (likely sprite sheets)
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

#[cfg(feature = "gui")]
struct EditorDemo {
    editor_plugin: MapEditorPlugin,
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

        Self { editor_plugin }
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

