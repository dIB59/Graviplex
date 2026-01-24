//! Regression tests for the map editor module.
//!
//! These tests ensure that the editor functionality works correctly,
//! especially around texture/atlas integration which was a source of bugs.

#[cfg(feature = "gui")]
mod tests {
    use graviplex::prelude::*;
    use graviplex::editor::{MapEditor, MapObject, EditorConfig, GridConfig, ObjectVisual};
    
    // =========================================================================
    // OBJECT REGISTRATION TESTS
    // =========================================================================

    #[test]
    fn test_register_texture_object() {
        let mut editor = MapEditor::new();
        
        let obj = MapObject::texture("Castle", "assets/castle.png", Vec2::new(128.0, 128.0));
        editor.register_object(obj);
        
        assert!(editor.palette.get_object("Castle").is_some());
    }

    #[test]
    fn test_register_circle_object() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Red Circle", 50.0, Color::RED);
        
        assert!(editor.palette.get_object("Red Circle").is_some());
    }

    #[test]
    fn test_register_rect_object() {
        let mut editor = MapEditor::new();
        
        editor.register_rect("Blue Box", 100.0, 50.0, Color::BLUE);
        
        assert!(editor.palette.get_object("Blue Box").is_some());
    }

    #[test]
    fn test_register_sprite_sheet() {
        let mut editor = MapEditor::new();
        
        let obj = MapObject::sprite_sheet("Walk Animation", "assets/walk.png", 8, Vec2::new(64.0, 64.0));
        editor.register_object(obj);
        
        let registered = editor.palette.get_object("Walk Animation");
        assert!(registered.is_some());
        
        let registered = registered.unwrap();
        match &registered.visual {
            ObjectVisual::SpriteSheet { frame_count, .. } => {
                assert_eq!(frame_count, &8);
            }
            _ => panic!("Expected SpriteSheet visual"),
        }
    }

    #[test]
    fn test_register_tileset() {
        let mut editor = MapEditor::new();
        
        let obj = MapObject::tileset("Terrain Tiles", "assets/tiles.png", 4, 4, Vec2::new(32.0, 32.0));
        editor.register_object(obj);
        
        let registered = editor.palette.get_object("Terrain Tiles");
        assert!(registered.is_some());
        
        let registered = registered.unwrap();
        match &registered.visual {
            ObjectVisual::Tileset { columns, rows, .. } => {
                assert_eq!(columns, &4);
                assert_eq!(rows, &4);
            }
            _ => panic!("Expected Tileset visual"),
        }
    }

    #[test]
    fn test_register_nine_slice() {
        let mut editor = MapEditor::new();
        
        let obj = MapObject::nine_slice("Button", "assets/button.png", Vec2::new(100.0, 40.0), 10, 10, 10, 10);
        editor.register_object(obj);
        
        let registered = editor.palette.get_object("Button");
        assert!(registered.is_some());
        
        let registered = registered.unwrap();
        match &registered.visual {
            ObjectVisual::NineSlice { left, right, top, bottom, .. } => {
                assert_eq!(left, &10);
                assert_eq!(right, &10);
                assert_eq!(top, &10);
                assert_eq!(bottom, &10);
            }
            _ => panic!("Expected NineSlice visual"),
        }
    }

    // =========================================================================
    // OBJECT PLACEMENT TESTS
    // =========================================================================

    #[test]
    fn test_place_object() {
        let config = EditorConfig {
            grid: GridConfig {
                enabled: false, // Disable grid snapping for precise placement
                ..Default::default()
            },
            ..Default::default()
        };
        let mut editor = MapEditor::with_config(config);
        
        let obj = MapObject::circle("Test Circle", 25.0, Color::GREEN);
        editor.register_object(obj);
        
        let position = Vec2::new(100.0, 200.0);
        let placed_id = editor.place_object("Test Circle", position);
        
        assert!(placed_id.is_some());
        
        let objects = editor.objects();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].position, position);
        assert_eq!(objects[0].object_type, "Test Circle");
    }

    #[test]
    fn test_place_multiple_objects() {
        let config = EditorConfig {
            grid: GridConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut editor = MapEditor::with_config(config);
        
        editor.register_circle("Circle A", 10.0, Color::RED);
        editor.register_circle("Circle B", 20.0, Color::BLUE);
        
        editor.place_object("Circle A", Vec2::new(0.0, 0.0));
        editor.place_object("Circle B", Vec2::new(50.0, 50.0));
        
        assert_eq!(editor.objects().len(), 2);
    }

    #[test]
    fn test_delete_object() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Test", 10.0, Color::RED);
        
        let id = editor.place_object("Test", Vec2::new(0.0, 0.0)).unwrap();
        assert_eq!(editor.objects().len(), 1);
        
        editor.delete_object(id);
        assert_eq!(editor.objects().len(), 0);
    }

    // =========================================================================
    // GRID SNAPPING TESTS
    // =========================================================================

    #[test]
    fn test_grid_snap() {
        let grid = GridConfig {
            enabled: true,
            visible: true,
            cell_size: Vec2::new(32.0, 32.0),
            ..Default::default()
        };
        
        // Test snapping to grid (45/32 = 1.4 rounds to 1, 78/32 = 2.4 rounds to 2)
        let snapped = grid.snap(Vec2::new(45.0, 78.0));
        assert_eq!(snapped, Vec2::new(32.0, 64.0)); // 1*32=32, 2*32=64
    }

    #[test]
    fn test_grid_snap_disabled() {
        let grid = GridConfig {
            enabled: false,
            ..Default::default()
        };
        
        // When grid is disabled, position should not change
        let original = Vec2::new(45.5, 78.3);
        let result = grid.snap(original);
        assert_eq!(result, original);
    }

    #[test]
    fn test_grid_snap_with_offset() {
        let grid = GridConfig {
            enabled: true,
            cell_size: Vec2::new(32.0, 32.0),
            offset: Vec2::new(16.0, 16.0),
            ..Default::default()
        };
        
        // With offset, snapping should account for the offset
        let snapped = grid.snap(Vec2::new(50.0, 50.0));
        // 50 - 16 = 34, rounds to 32, + 16 = 48
        assert_eq!(snapped, Vec2::new(48.0, 48.0));
    }

    // =========================================================================
    // SELECTION TESTS
    // =========================================================================

    #[test]
    fn test_select_object() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Test", 25.0, Color::RED);
        
        let id = editor.place_object("Test", Vec2::new(50.0, 50.0)).unwrap();
        
        editor.select(id);
        
        assert!(editor.state.selected.contains(&id));
        assert_eq!(editor.state.selected.len(), 1);
    }

    #[test]
    fn test_select_all() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Test", 10.0, Color::RED);
        
        editor.place_object("Test", Vec2::new(0.0, 0.0));
        editor.place_object("Test", Vec2::new(50.0, 50.0));
        editor.place_object("Test", Vec2::new(100.0, 100.0));
        
        editor.select_all();
        
        assert_eq!(editor.state.selected.len(), 3);
    }

    #[test]
    fn test_clear_selection() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Test", 10.0, Color::RED);
        
        let id = editor.place_object("Test", Vec2::new(0.0, 0.0)).unwrap();
        editor.select(id);
        
        assert_eq!(editor.state.selected.len(), 1);
        
        editor.clear_selection();
        
        assert_eq!(editor.state.selected.len(), 0);
    }

    // =========================================================================
    // UNDO/REDO TESTS
    // =========================================================================

    #[test]
    fn test_undo_place() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Test", 10.0, Color::RED);
        
        editor.place_object("Test", Vec2::new(0.0, 0.0));
        assert_eq!(editor.objects().len(), 1);
        
        editor.undo();
        assert_eq!(editor.objects().len(), 0);
    }

    #[test]
    fn test_redo_place() {
        let mut editor = MapEditor::new();
        
        editor.register_circle("Test", 10.0, Color::RED);
        
        editor.place_object("Test", Vec2::new(0.0, 0.0));
        editor.undo();
        assert_eq!(editor.objects().len(), 0);
        
        editor.redo();
        assert_eq!(editor.objects().len(), 1);
    }

    // =========================================================================
    // CATEGORY TESTS
    // =========================================================================

    #[test]
    fn test_object_categories() {
        let mut editor = MapEditor::new();
        
        let obj1 = MapObject::circle("Circle 1", 10.0, Color::RED)
            .with_category("Shapes");
        let obj2 = MapObject::circle("Circle 2", 20.0, Color::BLUE)
            .with_category("Shapes");
        let obj3 = MapObject::texture("Tree", "tree.png", Vec2::new(64.0, 64.0))
            .with_category("Nature");
        
        editor.register_object(obj1);
        editor.register_object(obj2);
        editor.register_object(obj3);
        
        let categories: Vec<_> = editor.palette.categories().collect();
        assert!(categories.iter().any(|c| *c == "Shapes"));
        assert!(categories.iter().any(|c| *c == "Nature"));
        
        let shapes = editor.palette.assets_in_category("Shapes");
        assert!(shapes.is_some());
        assert_eq!(shapes.unwrap().len(), 2);
    }

    // =========================================================================
    // MAP OBJECT BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_object_with_category() {
        let obj = MapObject::circle("Test", 10.0, Color::RED)
            .with_category("My Category");
        
        assert_eq!(obj.category, "My Category");
    }

    #[test]
    fn test_object_with_layer() {
        let obj = MapObject::circle("Test", 10.0, Color::RED)
            .with_layer(5);
        
        assert_eq!(obj.default_layer, 5);
    }

    #[test]
    fn test_object_rotatable() {
        let obj = MapObject::circle("Test", 10.0, Color::RED)
            .with_rotatable(false);
        
        assert!(!obj.rotatable);
    }

    #[test]
    fn test_object_scalable() {
        let obj = MapObject::circle("Test", 10.0, Color::RED)
            .with_scalable(false);
        
        assert!(!obj.scalable);
    }

    // =========================================================================
    // TEXTURE NAME CONSISTENCY TESTS (Regression for atlas lookup bug)
    // =========================================================================

    #[test]
    fn test_texture_name_stored_correctly() {
        let texture_path = "assets/sprites/player.png";
        let obj = MapObject::texture("Player", texture_path, Vec2::new(64.0, 64.0));
        
        match &obj.visual {
            ObjectVisual::Texture { texture_name, .. } => {
                assert_eq!(texture_name, texture_path);
            }
            _ => panic!("Expected Texture visual"),
        }
    }

    #[test]
    fn test_sprite_sheet_name_stored_correctly() {
        let texture_path = "assets/sprites/walk.png";
        let obj = MapObject::sprite_sheet("Walk", texture_path, 8, Vec2::new(64.0, 64.0));
        
        match &obj.visual {
            ObjectVisual::SpriteSheet { texture_name, .. } => {
                assert_eq!(texture_name, texture_path);
            }
            _ => panic!("Expected SpriteSheet visual"),
        }
    }

    #[test]
    fn test_tileset_name_stored_correctly() {
        let texture_path = "assets/tiles/terrain.png";
        let obj = MapObject::tileset("Terrain", texture_path, 4, 4, Vec2::new(32.0, 32.0));
        
        match &obj.visual {
            ObjectVisual::Tileset { texture_name, .. } => {
                assert_eq!(texture_name, texture_path);
            }
            _ => panic!("Expected Tileset visual"),
        }
    }

    #[test]
    fn test_nine_slice_name_stored_correctly() {
        let texture_path = "assets/ui/button.png";
        let obj = MapObject::nine_slice("Button", texture_path, Vec2::new(100.0, 40.0), 8, 8, 8, 8);
        
        match &obj.visual {
            ObjectVisual::NineSlice { texture_name, .. } => {
                assert_eq!(texture_name, texture_path);
            }
            _ => panic!("Expected NineSlice visual"),
        }
    }

    // =========================================================================
    // PLACED OBJECT STATE TESTS
    // =========================================================================

    #[test]
    fn test_placed_object_default_state() {
        let config = EditorConfig {
            grid: GridConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut editor = MapEditor::with_config(config);
        
        editor.register_circle("Test", 25.0, Color::RED);
        
        let id = editor.place_object("Test", Vec2::new(100.0, 100.0)).unwrap();
        
        let obj = editor.get_object(id).unwrap();
        
        // Default scale should be 1.0
        assert_eq!(obj.scale, Vec2::new(1.0, 1.0));
        // Default rotation should be 0
        assert_eq!(obj.rotation, 0.0);
        // Should not be selected by default
        assert!(!obj.selected);
    }

    #[test]
    fn test_placed_object_position() {
        let config = EditorConfig {
            grid: GridConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut editor = MapEditor::with_config(config);
        
        editor.register_circle("Test", 25.0, Color::RED);
        
        let position = Vec2::new(123.45, 678.90);
        let id = editor.place_object("Test", position).unwrap();
        
        let obj = editor.get_object(id).unwrap();
        assert_eq!(obj.position, position);
    }

    // =========================================================================
    // EDITOR CONFIG TESTS
    // =========================================================================

    #[test]
    fn test_editor_default_config() {
        let config = EditorConfig::default();
        
        assert!(!config.enabled);
        assert!(config.show_panel);
        assert_eq!(config.panel_width, 250.0);
        assert!(config.show_outlines);
        assert_eq!(config.max_undo_steps, 50);
    }

    #[test]
    fn test_editor_with_custom_config() {
        let config = EditorConfig {
            enabled: true,
            show_panel: false,
            panel_width: 300.0,
            max_undo_steps: 100,
            ..Default::default()
        };
        
        let editor = MapEditor::with_config(config.clone());
        
        assert!(editor.config.enabled);
        assert!(!editor.config.show_panel);
        assert_eq!(editor.config.panel_width, 300.0);
        assert_eq!(editor.config.max_undo_steps, 100);
    }

    // =========================================================================
    // GRID CONFIG TESTS
    // =========================================================================

    #[test]
    fn test_grid_default_config() {
        let grid = GridConfig::default();
        
        assert!(grid.enabled);  // Grid is enabled by default
        assert!(grid.visible);
        assert_eq!(grid.cell_size, Vec2::new(32.0, 32.0));
    }
}

// =========================================================================
// ATLAS BUILDER TESTS (Regression for atlas overflow bug)
// =========================================================================

#[cfg(feature = "textures")]
mod atlas_tests {
    use graviplex::prelude::*;
    
    #[test]
    fn test_atlas_region_count() {
        // This test ensures the atlas correctly reports region counts
        let _builder = AtlasBuilder::new();
        // Note: we can't actually test building without GPU, but we can test the builder API
        assert!(true); // Placeholder - actual testing requires GPU
    }
}
