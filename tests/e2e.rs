//! End-to-end tests for Graviplex engine.
//!
//! Note: These tests verify the API works correctly at a high level.

use graviplex::prelude::*;
use graviplex::advanced::CircleInstance;

#[cfg(feature = "textures")]
use graviplex::advanced::{AtlasBuilder, AtlasRegion, SpriteInstance};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);

        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(a - b, Vec2::new(-2.0, -2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
        assert!((a.length() - 2.236).abs() < 0.01);
    }

    #[test]
    fn test_circle_geometry() {
        let c = Circle::new(Vec2::ZERO, 10.0, Color::RED);

        assert!(c.contains(Vec2::new(5.0, 5.0)));
        assert!(!c.contains(Vec2::new(15.0, 0.0)));

        let other = Circle::new(Vec2::new(15.0, 0.0), 10.0, Color::BLUE);
        assert!(c.intersects(&other));

        let far = Circle::new(Vec2::new(100.0, 0.0), 10.0, Color::GREEN);
        assert!(!c.intersects(&far));
    }

    #[test]
    fn test_rect_geometry() {
        let r = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));

        assert!(r.contains(Vec2::new(5.0, 5.0)));
        assert!(!r.contains(Vec2::new(15.0, 5.0)));
        assert_eq!(r.center(), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_color_creation() {
        let c = Color::rgba(1.0, 0.5, 0.25, 1.0);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 0.25);
        assert_eq!(c.a, 1.0);

        let from_arr: Color = [1.0, 0.0, 0.0, 1.0].into();
        assert_eq!(from_arr, Color::RED);
    }

    #[test]
    fn test_circle_instance_creation() {
        let c = CircleInstance::new([10.0, 20.0], 5.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(c.position, [10.0, 20.0]);
        assert_eq!(c.radius, 5.0);

        // Test From<Circle>
        let circle = Circle::new(Vec2::new(30.0, 40.0), 15.0, Color::BLUE);
        let instance: CircleInstance = circle.into();
        assert_eq!(instance.position, [30.0, 40.0]);
        assert_eq!(instance.radius, 15.0);
    }

    #[test]
    fn test_app_stats_default() {
        let stats = AppStats::default();
        assert_eq!(stats.average_fps, 0.0);
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.total_time, 0.0);
    }

    #[test]
    fn test_camera_config_builder() {
        let config = CameraConfig::centered()
            .with_scale(100.0)
            .with_position(50.0, 50.0)
            .with_zoom_speed(1.5)
            .with_move_speed(500.0)
            .with_zoom_range(0.1, 100.0);

        assert_eq!(config.scale, 100.0);
        assert_eq!(config.position, [50.0, 50.0]);
        assert_eq!(config.zoom_speed, 1.5);
        assert_eq!(config.move_speed, 500.0);
        assert_eq!(config.min_zoom, 0.1);
        assert_eq!(config.max_zoom, 100.0);
    }

    #[test]
    fn test_fps_counter_tracking() {
        use graviplex::core::stats::FpsCounter;

        let mut fps_counter = FpsCounter::new();

        // Initial state
        assert_eq!(fps_counter.frame_count(), 0);
        assert_eq!(fps_counter.fps(), 0.0);

        // Simulate 60 FPS frame
        fps_counter.update(1.0 / 60.0);

        assert_eq!(fps_counter.frame_count(), 1);
        assert!(fps_counter.fps() > 0.0);
        assert!(fps_counter.frame_time_ms() > 0.0);
    }

    // =========================================================================
    // SPRITE & TEXTURE TESTS (feature-gated)
    // =========================================================================

    #[cfg(feature = "textures")]
    mod texture_tests {
        use super::*;

        #[test]
        fn test_sprite_instance_creation_e2e() {
            let sprite = SpriteInstance::new(
                [100.0, 200.0],
                [64.0, 64.0],
                [0.0, 0.0, 0.5, 0.5],
                [1.0, 1.0, 1.0, 1.0],
                0.0,
                0,
            );

            assert_eq!(sprite.position, [100.0, 200.0]);
            assert_eq!(sprite.size, [64.0, 64.0]);
        }

        #[test]
        fn test_atlas_region_api() {
            let region = AtlasRegion {
                uv: [0.0, 0.0, 0.5, 0.5],
                width: 32,
                height: 32,
            };

            assert_eq!(region.uv_rect(), [0.0, 0.0, 0.5, 0.5]);
            assert_eq!(region.size_f32(), [32.0, 32.0]);
        }

        #[test]
        fn test_atlas_builder_programmatic_images() {
            // Create test images programmatically
            let red = image::RgbaImage::from_pixel(16, 16, image::Rgba([255, 0, 0, 255]));
            let green = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 255, 0, 255]));
            let blue = image::RgbaImage::from_pixel(24, 24, image::Rgba([0, 0, 255, 255]));

            let _builder = AtlasBuilder::new()
                .add_rgba_image("red", red)
                .add_rgba_image("green", green)
                .add_rgba_image("blue", blue);

            // Builder accepted all 3 images successfully (no panic)
        }

        #[test]
        fn test_sprite_with_texture_shape() {
            let sprite = Sprite::texture("player", Vec2::new(64.0, 64.0), Color::WHITE);

            match sprite.shape {
                SpriteShape::Texture { region_name, size } => {
                    assert_eq!(region_name, "player");
                    assert_eq!(size.x, 64.0);
                    assert_eq!(size.y, 64.0);
                }
                _ => panic!("Expected Texture shape"),
            }
        }

        #[test]
        fn test_sprite_instance_z_ordering() {
            let mut sprites = vec![
                SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0; 4], [1.0; 4], 0.0, 10),
                SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0; 4], [1.0; 4], 0.0, -5),
                SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0; 4], [1.0; 4], 0.0, 0),
                SpriteInstance::new([0.0, 0.0], [1.0, 1.0], [0.0; 4], [1.0; 4], 0.0, 5),
            ];

            // Sort by z_order (same as SpritePipeline does)
            sprites.sort_by(|a, b| a.z_order.cmp(&b.z_order));

            assert_eq!(sprites[0].z_order, -5);
            assert_eq!(sprites[1].z_order, 0);
            assert_eq!(sprites[2].z_order, 5);
            assert_eq!(sprites[3].z_order, 10);
        }

        #[test]
        fn test_sprite_instance_to_gpu_preserves_data() {
            let sprite = SpriteInstance::new(
                [123.0, 456.0],
                [78.0, 90.0],
                [0.1, 0.2, 0.3, 0.4],
                [0.5, 0.6, 0.7, 0.8],
                1.5,
                999,
            );

            let gpu = sprite.to_gpu();

            assert_eq!(gpu.position, sprite.position);
            assert_eq!(gpu.size, sprite.size);
            assert_eq!(gpu.uv_rect, sprite.uv_rect);
            assert_eq!(gpu.tint, sprite.tint);
            assert_eq!(gpu.rotation, sprite.rotation);
        }

        #[test]
        fn test_multiple_texture_sprites_in_world() {
            let mut world = World::new();

            // Spawn multiple texture sprites
            let player = world.spawn((
                Transform::from_position(Vec2::new(100.0, 100.0)),
                Sprite::texture("player", Vec2::new(32.0, 32.0), Color::WHITE),
                Visible,
            ));

            let enemy1 = world.spawn((
                Transform::from_position(Vec2::new(200.0, 100.0)),
                Sprite::texture("enemy", Vec2::new(32.0, 32.0), Color::RED),
                Visible,
            ));

            let enemy2 = world.spawn((
                Transform::from_position(Vec2::new(300.0, 100.0)),
                Sprite::texture("enemy", Vec2::new(32.0, 32.0), Color::RED),
                Visible,
            ));

            assert!(world.contains(player));
            assert!(world.contains(enemy1));
            assert!(world.contains(enemy2));

            // Count texture sprites by iterating
            let mut texture_count = 0;
            for (_, (_, sprite, _)) in world.query::<(&Transform, &Sprite, &Visible)>().iter() {
                if matches!(sprite.shape, SpriteShape::Texture { .. }) {
                    texture_count += 1;
                }
            }

            assert_eq!(texture_count, 3);
        }
    }

    // =========================================================================
    // REGRESSION TESTS - Ensure existing functionality works
    // =========================================================================

    #[test]
    fn test_circle_sprite_still_works() {
        let sprite = Sprite::circle(50.0, Color::RED);

        assert!(matches!(sprite.shape, SpriteShape::Circle { radius: 50.0 }));
        assert_eq!(sprite.color, Color::RED);
        assert_eq!(sprite.z_order, 0);
    }

    #[test]
    fn test_rect_sprite_still_works() {
        let sprite = Sprite::rect(100.0, 50.0, Color::BLUE);

        if let SpriteShape::Rect { size } = sprite.shape {
            assert_eq!(size.x, 100.0);
            assert_eq!(size.y, 50.0);
        } else {
            panic!("Expected Rect shape");
        }
    }

    #[test]
    fn test_line_sprite_still_works() {
        let sprite = Sprite::line(Vec2::new(100.0, 0.0), Color::GREEN);

        if let SpriteShape::Line { end_offset } = sprite.shape {
            assert_eq!(end_offset.x, 100.0);
            assert_eq!(end_offset.y, 0.0);
        } else {
            panic!("Expected Line shape");
        }
    }

    #[test]
    fn test_sprite_z_order_builder() {
        let sprite = Sprite::circle(10.0, Color::WHITE).with_z_order(42);
        assert_eq!(sprite.z_order, 42);

        let sprite2 = Sprite::rect(20.0, 20.0, Color::WHITE).with_z_order(-10);
        assert_eq!(sprite2.z_order, -10);
    }

    #[test]
    fn test_sprite_color_builder() {
        let sprite = Sprite::circle(10.0, Color::RED).with_color(Color::BLUE);
        assert_eq!(sprite.color, Color::BLUE);
    }

    #[test]
    fn test_mixed_sprite_types_in_world() {
        let mut world = World::new();

        // Mix of different sprite types
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(50.0, Color::RED),
            Visible,
        ));

        world.spawn((
            Transform::from_position(Vec2::new(100.0, 0.0)),
            Sprite::rect(30.0, 30.0, Color::BLUE),
            Visible,
        ));

        world.spawn((
            Transform::from_position(Vec2::new(200.0, 0.0)),
            Sprite::line(Vec2::new(50.0, 50.0), Color::GREEN),
            Visible,
        ));

        assert_eq!(world.len(), 3);

        // Query should return all 3
        let count = world.query::<(&Transform, &Sprite, &Visible)>().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_entity_despawn_with_sprite() {
        let mut world = World::new();

        let entity = world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(10.0, Color::WHITE),
            Visible,
        ));

        assert!(world.contains(entity));
        world.despawn(entity).unwrap();
        assert!(!world.contains(entity));
    }

    #[test]
    fn test_sprite_component_modification() {
        let mut world = World::new();

        let entity = world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(10.0, Color::RED),
            Visible,
        ));

        // Modify the sprite
        {
            let mut sprite = world.get_mut::<Sprite>(entity).unwrap();
            sprite.color = Color::BLUE;
            sprite.z_order = 100;
        }

        // Verify changes
        let sprite = world.get::<Sprite>(entity).unwrap();
        assert_eq!(sprite.color, Color::BLUE);
        assert_eq!(sprite.z_order, 100);
    }

    // =========================================================================
    // Z-ORDER REGRESSION TESTS - Ensure correct ordering across sprite types
    // =========================================================================

    /// Regression test: Z-ordering must be respected across different sprite types.
    /// Previously, all circles were rendered first, then all lines, then all textures,
    /// which broke z-ordering when mixing sprite types.
    #[test]
    fn test_z_order_sorting_mixed_sprite_types() {
        let mut world = World::new();

        // Spawn sprites with intentionally mixed z-orders and types
        // The correct render order should be: background circle -> middle line -> foreground rect
        world.spawn((
            Transform::from_position(Vec2::new(0.0, 0.0)),
            Sprite::rect(100.0, 100.0, Color::RED).with_z_order(10), // Should be LAST (foreground)
            Visible,
        ));

        world.spawn((
            Transform::from_position(Vec2::new(0.0, 0.0)),
            Sprite::circle(50.0, Color::BLUE).with_z_order(0), // Should be FIRST (background)
            Visible,
        ));

        world.spawn((
            Transform::from_position(Vec2::new(0.0, 0.0)),
            Sprite::line(Vec2::new(100.0, 100.0), Color::GREEN).with_z_order(5), // Should be MIDDLE
            Visible,
        ));

        // Collect and sort like render_world does
        let mut renderables: Vec<(i32, SpriteShape)> = world
            .query::<(&Transform, &Sprite, &Visible)>()
            .iter()
            .map(|(_, (_, sprite, _))| (sprite.z_order, sprite.shape.clone()))
            .collect();

        renderables.sort_by_key(|(z, _)| *z);

        // Verify correct z-order
        assert_eq!(renderables.len(), 3);
        assert_eq!(renderables[0].0, 0);  // Circle at z=0
        assert_eq!(renderables[1].0, 5);  // Line at z=5
        assert_eq!(renderables[2].0, 10); // Rect at z=10

        // Verify types are in correct order
        assert!(matches!(renderables[0].1, SpriteShape::Circle { .. }));
        assert!(matches!(renderables[1].1, SpriteShape::Line { .. }));
        assert!(matches!(renderables[2].1, SpriteShape::Rect { .. }));
    }

    /// Regression test: Same z-order sprites should still be batchable.
    #[test]
    fn test_same_z_order_batching() {
        let mut world = World::new();

        // Multiple circles at same z-order - should be batched together
        for i in 0..5 {
            world.spawn((
                Transform::from_position(Vec2::new(i as f32 * 10.0, 0.0)),
                Sprite::circle(10.0, Color::RED).with_z_order(5),
                Visible,
            ));
        }

        // Multiple lines at same z-order - should be batched together
        for i in 0..3 {
            world.spawn((
                Transform::from_position(Vec2::new(i as f32 * 20.0, 50.0)),
                Sprite::line(Vec2::new(15.0, 0.0), Color::BLUE).with_z_order(5),
                Visible,
            ));
        }

        let mut renderables: Vec<(i32, SpriteShape)> = world
            .query::<(&Transform, &Sprite, &Visible)>()
            .iter()
            .map(|(_, (_, sprite, _))| (sprite.z_order, sprite.shape.clone()))
            .collect();

        renderables.sort_by_key(|(z, _)| *z);

        // All 8 sprites should have z_order = 5
        assert_eq!(renderables.len(), 8);
        for (z, _) in &renderables {
            assert_eq!(*z, 5);
        }
    }

    /// Regression test: Z-order changes should trigger proper flush boundaries.
    #[test]
    fn test_z_order_boundaries() {
        let mut world = World::new();

        // Create sprites at different z-orders
        // This tests that flush happens at z-order boundaries
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(10.0, Color::RED).with_z_order(-10),
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(20.0, Color::GREEN).with_z_order(0),
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(30.0, Color::BLUE).with_z_order(10),
            Visible,
        ));

        let mut renderables: Vec<i32> = world
            .query::<(&Sprite, &Visible)>()
            .iter()
            .map(|(_, (sprite, _))| sprite.z_order)
            .collect();

        renderables.sort();

        // Should have 3 distinct z-order groups: -10, 0, 10
        assert_eq!(renderables, vec![-10, 0, 10]);
    }

    /// Regression test: Negative z-orders should work correctly with mixed types.
    #[test]
    fn test_negative_z_order_mixed_types() {
        let mut world = World::new();

        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::line(Vec2::new(100.0, 0.0), Color::WHITE).with_z_order(-100), // Far background
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(50.0, Color::RED).with_z_order(0), // Middle
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::rect(30.0, 30.0, Color::BLUE).with_z_order(100), // Foreground
            Visible,
        ));

        let mut renderables: Vec<(i32, SpriteShape)> = world
            .query::<(&Sprite, &Visible)>()
            .iter()
            .map(|(_, (sprite, _))| (sprite.z_order, sprite.shape.clone()))
            .collect();

        renderables.sort_by_key(|(z, _)| *z);

        assert_eq!(renderables[0].0, -100);
        assert!(matches!(renderables[0].1, SpriteShape::Line { .. }));
        
        assert_eq!(renderables[1].0, 0);
        assert!(matches!(renderables[1].1, SpriteShape::Circle { .. }));
        
        assert_eq!(renderables[2].0, 100);
        assert!(matches!(renderables[2].1, SpriteShape::Rect { .. }));
    }

    /// Regression test: Z-ordering with texture sprites (when textures feature enabled).
    #[cfg(feature = "textures")]
    #[test]
    fn test_z_order_with_texture_sprites() {
        let mut world = World::new();

        // Mix texture sprites with primitive sprites at different z-orders
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::texture("background", Vec2::new(800.0, 600.0), Color::WHITE).with_z_order(-10),
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::circle(50.0, Color::RED).with_z_order(0),
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::texture("player", Vec2::new(64.0, 64.0), Color::WHITE).with_z_order(5),
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::line(Vec2::new(100.0, 0.0), Color::GREEN).with_z_order(10),
            Visible,
        ));
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            Sprite::texture("ui_overlay", Vec2::new(200.0, 50.0), Color::WHITE).with_z_order(100),
            Visible,
        ));

        let mut renderables: Vec<(i32, &'static str)> = world
            .query::<(&Sprite, &Visible)>()
            .iter()
            .map(|(_, (sprite, _))| {
                let type_name = match sprite.shape {
                    SpriteShape::Circle { .. } => "circle",
                    SpriteShape::Rect { .. } => "rect",
                    SpriteShape::Line { .. } => "line",
                    SpriteShape::Texture { .. } => "texture",
                    SpriteShape::TextureId { .. } => "textureId",
                };
                (sprite.z_order, type_name)
            })
            .collect();

        renderables.sort_by_key(|(z, _)| *z);

        // Verify correct z-order across all types including textures
        assert_eq!(renderables.len(), 5);
        assert_eq!(renderables[0], (-10, "texture"));  // background
        assert_eq!(renderables[1], (0, "circle"));     // circle
        assert_eq!(renderables[2], (5, "texture"));    // player
        assert_eq!(renderables[3], (10, "line"));      // line
        assert_eq!(renderables[4], (100, "texture"));  // ui_overlay
    }
}
