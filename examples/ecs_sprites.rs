//! ECS-integrated sprite rendering example.
//!
//! This example demonstrates:
//! - Using texture sprites with the ECS world
//! - Spawning entities with texture-based Sprite components
//! - Movement and rotation systems with textured sprites
//! - Mixed rendering (texture sprites + primitive shapes)
//!
//! Run with: cargo run --example ecs_sprites --features textures

use graviplex::prelude::*;
use graviplex::advanced::{AtlasBuilder, SpritePipeline, TextureAtlas};

// Custom component for spin behavior
#[derive(Clone, Copy)]
struct Spin(f32); // radians per second

// Custom component for orbit behavior
#[derive(Clone, Copy)]
struct Orbit {
    center: Vec2,
    radius: f32,
    speed: f32,
    angle: f32,
}

struct EcsSpritesGame {
    atlas: Option<TextureAtlas>,
    sprite_pipeline: Option<SpritePipeline>,
}

impl EcsSpritesGame {
    fn new() -> Self {
        Self {
            atlas: None,
            sprite_pipeline: None,
        }
    }

    fn create_atlas() -> Result<AtlasBuilder, graviplex::advanced::AtlasError> {
        // Try to load images from the assets folder
        // Place your PNG/JPG images in: assets/sprites/
        //   - player.png (64x64 recommended)
        //   - enemy.png (48x48 recommended)
        //   - pickup.png (32x32 recommended)
        //   - ring.png (64x64 recommended)
        //   - bullet.png (16x16 recommended)
        
        let builder = AtlasBuilder::new()
            .add_image("player", "assets/sprites/player.png")?
            .add_image("enemy", "assets/sprites/enemy.png")?
            .add_image("pickup", "assets/sprites/pickup.png")?
            .add_image("ring", "assets/sprites/ring.png")?
            .add_image("bullet", "assets/sprites/bullet.png")?;
        
        Ok(builder)
    }

    /// Fallback to procedural textures if image files aren't found
    fn create_fallback_atlas() -> AtlasBuilder {
        fn make_gradient_square(size: u32, color1: [u8; 4], color2: [u8; 4]) -> image::RgbaImage {
            let mut img = image::RgbaImage::new(size, size);
            for y in 0..size {
                for x in 0..size {
                    let t = x as f32 / size as f32;
                    let r = (color1[0] as f32 * (1.0 - t) + color2[0] as f32 * t) as u8;
                    let g = (color1[1] as f32 * (1.0 - t) + color2[1] as f32 * t) as u8;
                    let b = (color1[2] as f32 * (1.0 - t) + color2[2] as f32 * t) as u8;
                    img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                }
            }
            img
        }

        fn make_checkerboard(size: u32, color1: [u8; 4], color2: [u8; 4]) -> image::RgbaImage {
            let mut img = image::RgbaImage::new(size, size);
            let cell = size / 4;
            for y in 0..size {
                for x in 0..size {
                    let checker = ((x / cell) + (y / cell)) % 2 == 0;
                    let color = if checker { color1 } else { color2 };
                    img.put_pixel(x, y, image::Rgba(color));
                }
            }
            img
        }

        fn make_ring(size: u32, color: [u8; 4]) -> image::RgbaImage {
            let mut img = image::RgbaImage::new(size, size);
            let center = size as f32 / 2.0;
            let outer = center - 2.0;
            let inner = center * 0.5;
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - center;
                    let dy = y as f32 - center;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= outer && dist >= inner {
                        img.put_pixel(x, y, image::Rgba(color));
                    } else {
                        img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                    }
                }
            }
            img
        }

        AtlasBuilder::new()
            .add_rgba_image("player", make_gradient_square(64, [50, 150, 255, 255], [150, 50, 255, 255]))
            .add_rgba_image("enemy", make_gradient_square(48, [255, 100, 100, 255], [255, 50, 50, 255]))
            .add_rgba_image("pickup", make_checkerboard(32, [255, 255, 100, 255], [255, 200, 50, 255]))
            .add_rgba_image("ring", make_ring(64, [100, 255, 200, 255]))
            .add_rgba_image("bullet", make_gradient_square(16, [255, 255, 255, 255], [200, 200, 255, 255]))
    }
}

impl GameLoop for EcsSpritesGame {
    fn init(&mut self, world: &mut World, gfx: &Graphics) {
        // Try to build atlas from image files, fall back to procedural textures
        let atlas_builder = match Self::create_atlas() {
            Ok(builder) => {
                println!("Loaded sprite images from assets/sprites/");
                builder
            }
            Err(e) => {
                println!("Could not load images ({e}), using procedural textures");
                println!("To use real images, create: assets/sprites/{{player,enemy,pickup,ring,bullet}}.png");
                Self::create_fallback_atlas()
            }
        };

        let atlas = atlas_builder
            .build(&gfx.device, &gfx.queue, 2048)
            .expect("Failed to build atlas");

        let camera = Camera2D::default();
        let format = gfx.config.as_ref().expect("Surface config required").format;
        let sprite_pipeline = SpritePipeline::new(&gfx.device, format, &camera, &atlas);

        // Spawn player entity (center)
        world.spawn((
            Transform::from_position(Vec2::ZERO).with_uniform_scale(1.5),
            Sprite::texture("player", Vec2::new(64.0, 64.0), Color::WHITE).with_z_order(10),
            Spin(1.0),
            Visible,
        ));

        // Spawn orbiting enemies
        for i in 0..6 {
            let angle = (i as f32) * std::f32::consts::TAU / 6.0;
            world.spawn((
                Transform::from_position(Vec2::new(angle.cos() * 200.0, angle.sin() * 200.0)),
                Sprite::texture("enemy", Vec2::new(48.0, 48.0), Color::WHITE).with_z_order(5),
                Orbit {
                    center: Vec2::ZERO,
                    radius: 200.0,
                    speed: 0.5 + (i as f32) * 0.1,
                    angle,
                },
                Visible,
            ));
        }

        // Spawn pickups in a grid
        for x in -2..=2 {
            for y in -1..=1 {
                if x == 0 && y == 0 {
                    continue; // Skip center
                }
                world.spawn((
                    Transform::from_xy(x as f32 * 100.0, y as f32 * 150.0 + 50.0),
                    Sprite::texture("pickup", Vec2::new(32.0, 32.0), Color::WHITE).with_z_order(3),
                    Spin(3.0),
                    Visible,
                ));
            }
        }

        // Spawn decorative rings
        for i in 0..4 {
            let angle = (i as f32) * std::f32::consts::TAU / 4.0 + std::f32::consts::FRAC_PI_4;
            world.spawn((
                Transform::from_position(Vec2::new(angle.cos() * 350.0, angle.sin() * 250.0)),
                Sprite::texture("ring", Vec2::new(80.0, 80.0), Color::rgba(1.0, 1.0, 1.0, 0.7)).with_z_order(1),
                Spin(-0.5),
                Visible,
            ));
        }

        self.atlas = Some(atlas);
        self.sprite_pipeline = Some(sprite_pipeline);

        println!("Spawned {} entities", world.len());
    }

    fn update(&mut self, world: &mut World, res: &Resources) {
        let dt = res.time.delta();

        // Update spin rotation
        for (_, (transform, spin)) in world.query::<(&mut Transform, &Spin)>().iter() {
            transform.rotation += spin.0 * dt;
        }

        // Update orbit positions
        for (_, (transform, orbit)) in world.query::<(&mut Transform, &mut Orbit)>().iter() {
            orbit.angle += orbit.speed * dt;
            transform.position = orbit.center + Vec2::new(
                orbit.angle.cos() * orbit.radius,
                orbit.angle.sin() * orbit.radius,
            );
        }
    }

    fn render(&mut self, world: &World, draw: &mut DrawContext) {
        let atlas = self.atlas.as_ref().unwrap();
        let pipeline = self.sprite_pipeline.as_mut().unwrap();

        // Collect and render texture sprites from ECS
        let mut sprites_to_render: Vec<(Transform, Sprite)> = Vec::new();

        for (_, (transform, sprite, _)) in world.query::<(&Transform, &Sprite, &Visible)>().iter() {
            if let SpriteShape::Texture { .. } = sprite.shape {
                sprites_to_render.push((*transform, *sprite));
            }
        }

        // Sort by z_order
        sprites_to_render.sort_by_key(|(_, s)| s.z_order);

        // Render each texture sprite
        for (transform, sprite) in sprites_to_render {
            if let SpriteShape::Texture { region_name, size } = sprite.shape {
                if let Some(region) = atlas.get(region_name) {
                    let scaled_size = [
                        size.x * transform.scale.x,
                        size.y * transform.scale.y,
                    ];
                    pipeline.draw(
                        [transform.position.x, transform.position.y],
                        scaled_size,
                        region.uv_rect(),
                        sprite.color.into(),
                        transform.rotation,
                        sprite.z_order,
                    );
                }
            }
        }

        pipeline.flush(&draw.state(), atlas);

        // Draw some lines connecting enemies to center
        for (_, (transform, _, _)) in world.query::<(&Transform, &Orbit, &Visible)>().iter() {
            draw.line((
                Vec2::ZERO,
                transform.position,
                Color::rgba(0.5, 0.5, 0.5, 0.3),
            ));
        }

        // Draw a circle around the player
        draw.circle((Vec2::ZERO, 45.0, Color::rgba(0.3, 0.6, 1.0, 0.3)));
    }
}

fn main() {
    App::build(EcsSpritesGame::new())
        .title("Graviplex ECS Sprites Demo")
        .size(1200, 800)
        .camera(CameraConfig::centered().with_scale(1.0))
        .run()
        .unwrap();
}
