//! Sprite rendering demo with procedurally generated texture atlas.
//!
//! This example demonstrates:
//! - Creating a texture atlas with auto-packing
//! - Rendering textured sprites with different tints
//! - Z-order sorting for layered rendering
//! - Sprite rotation and scaling
//!
//! Run with: cargo run --example sprite_demo --features textures

use graviplex::prelude::*;
use graviplex::advanced::{AtlasBuilder, SpritePipeline, TextureAtlas};

struct SpriteDemo {
    atlas: Option<TextureAtlas>,
    sprite_pipeline: Option<SpritePipeline>,
    time: f32,
}

impl SpriteDemo {
    fn new() -> Self {
        Self {
            atlas: None,
            sprite_pipeline: None,
            time: 0.0,
        }
    }

    /// Create procedural test images for the atlas
    fn create_test_images() -> AtlasBuilder {
        // Create a simple colored square with border
        fn make_square(size: u32, color: [u8; 4], border_color: [u8; 4]) -> image::RgbaImage {
            let mut img = image::RgbaImage::new(size, size);
            for y in 0..size {
                for x in 0..size {
                    let is_border = x == 0 || y == 0 || x == size - 1 || y == size - 1;
                    let pixel = if is_border { border_color } else { color };
                    img.put_pixel(x, y, image::Rgba(pixel));
                }
            }
            img
        }

        // Create a simple circle
        fn make_circle(size: u32, color: [u8; 4]) -> image::RgbaImage {
            let mut img = image::RgbaImage::new(size, size);
            let center = size as f32 / 2.0;
            let radius = center - 1.0;
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - center;
                    let dy = y as f32 - center;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= radius {
                        img.put_pixel(x, y, image::Rgba(color));
                    } else {
                        img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                    }
                }
            }
            img
        }

        // Create a star shape
        fn make_star(size: u32, color: [u8; 4]) -> image::RgbaImage {
            let mut img = image::RgbaImage::new(size, size);
            let center = size as f32 / 2.0;
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - center;
                    let dy = y as f32 - center;
                    let angle = dy.atan2(dx);
                    let dist = (dx * dx + dy * dy).sqrt();
                    // 5-pointed star
                    let star_radius = center * (0.5 + 0.5 * (5.0 * angle).cos().abs());
                    if dist <= star_radius {
                        img.put_pixel(x, y, image::Rgba(color));
                    } else {
                        img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                    }
                }
            }
            img
        }

        AtlasBuilder::new()
            .add_rgba_image("red_square", make_square(64, [255, 100, 100, 255], [200, 50, 50, 255]))
            .add_rgba_image("green_square", make_square(64, [100, 255, 100, 255], [50, 200, 50, 255]))
            .add_rgba_image("blue_square", make_square(64, [100, 100, 255, 255], [50, 50, 200, 255]))
            .add_rgba_image("yellow_circle", make_circle(48, [255, 255, 100, 255]))
            .add_rgba_image("cyan_circle", make_circle(48, [100, 255, 255, 255]))
            .add_rgba_image("magenta_circle", make_circle(48, [255, 100, 255, 255]))
            .add_rgba_image("white_star", make_star(64, [255, 255, 255, 255]))
            .add_rgba_image("orange_star", make_star(48, [255, 180, 50, 255]))
    }
}

impl GameLoop for SpriteDemo {
    fn init(&mut self, _world: &mut World, gfx: &Graphics) {
        // Build the texture atlas
        let atlas = Self::create_test_images()
            .build(&gfx.device, &gfx.queue, 512)
            .expect("Failed to build texture atlas");

        println!("Atlas created: {}x{}", atlas.width, atlas.height);
        println!("Regions: {:?}", atlas.region_names().collect::<Vec<_>>());

        // Create the sprite pipeline
        let camera = Camera2D::default();
        let format = gfx.config.as_ref().expect("Surface config required").format;
        let sprite_pipeline = SpritePipeline::new(&gfx.device, format, &camera, &atlas);

        self.atlas = Some(atlas);
        self.sprite_pipeline = Some(sprite_pipeline);
    }

    fn update(&mut self, _world: &mut World, res: &Resources) {
        self.time += res.time.delta();
    }

    fn render(&mut self, _world: &World, draw: &mut DrawContext) {
        let atlas = self.atlas.as_ref().unwrap();
        let pipeline = self.sprite_pipeline.as_mut().unwrap();

        // Background layer (z_order = -10)
        let bg_region = atlas.get("blue_square").unwrap();
        for i in 0..5 {
            for j in 0..5 {
                let x = (i as f32 - 2.0) * 150.0;
                let y = (j as f32 - 2.0) * 150.0;
                pipeline.draw(
                    [x, y],
                    [100.0, 100.0],
                    bg_region.uv_rect(),
                    [0.3, 0.3, 0.5, 0.5], // Dark blue tint, semi-transparent
                    0.0,
                    -10,
                );
            }
        }

        // Orbiting circles (z_order = 0)
        let circle_names = ["yellow_circle", "cyan_circle", "magenta_circle"];
        for (i, name) in circle_names.iter().enumerate() {
            let region = atlas.get(name).unwrap();
            let angle = self.time + (i as f32) * std::f32::consts::TAU / 3.0;
            let x = angle.cos() * 200.0;
            let y = angle.sin() * 200.0;
            pipeline.draw(
                [x, y],
                [60.0, 60.0],
                region.uv_rect(),
                [1.0, 1.0, 1.0, 1.0], // No tint
                0.0,
                0,
            );
        }

        // Rotating squares in corners (z_order = 5)
        let positions = [(-300.0, -200.0), (300.0, -200.0), (-300.0, 200.0), (300.0, 200.0)];
        let colors = ["red_square", "green_square", "blue_square", "red_square"];
        for (i, ((x, y), color)) in positions.iter().zip(colors.iter()).enumerate() {
            let region = atlas.get(*color).unwrap();
            let rotation = self.time * (1.0 + i as f32 * 0.5);
            pipeline.draw(
                [*x, *y],
                [80.0, 80.0],
                region.uv_rect(),
                [1.0, 1.0, 1.0, 1.0],
                rotation,
                5,
            );
        }

        // Central spinning star (z_order = 10, on top)
        let star_region = atlas.get("white_star").unwrap();
        let pulse = 1.0 + 0.3 * (self.time * 3.0).sin();
        let size = 120.0 * pulse;
        pipeline.draw(
            [0.0, 0.0],
            [size, size],
            star_region.uv_rect(),
            [1.0, 0.9, 0.5, 1.0], // Golden tint
            -self.time * 2.0,
            10,
        );

        // Small orbiting stars (z_order = 8)
        let small_star = atlas.get("orange_star").unwrap();
        for i in 0..8 {
            let angle = self.time * 1.5 + (i as f32) * std::f32::consts::TAU / 8.0;
            let radius = 100.0 + 20.0 * (self.time * 2.0 + i as f32).sin();
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;
            pipeline.draw(
                [x, y],
                [30.0, 30.0],
                small_star.uv_rect(),
                [1.0, 1.0, 1.0, 0.8],
                angle,
                8,
            );
        }

        // Flush sprites to GPU
        pipeline.flush(&draw.state(), atlas);

        // Also draw some regular circles for comparison
        draw.circle((Vec2::new(-400.0, 0.0), 30.0, Color::WHITE));
        draw.circle((Vec2::new(400.0, 0.0), 30.0, Color::WHITE));
    }
}

fn main() {
    App::build(SpriteDemo::new())
        .title("Graviplex Sprite Demo")
        .size(1200, 800)
        .camera(CameraConfig::centered().with_scale(1.0))
        .run()
        .unwrap();
}
