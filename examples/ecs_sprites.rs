//! ECS-integrated sprite rendering example.
//!
//! This example demonstrates:
//! - Using texture sprites with the ECS world
//! - Spawning entities with texture-based Sprite components
//! - Movement and rotation systems with textured sprites
//! - Mixed rendering (texture sprites + primitive shapes)
//!
//! The engine automatically handles all rendering - just call `draw.render_world(world)`!
//!
//! Run with: cargo run --example ecs_sprites

use graviplex::prelude::*;

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

struct EcsSpritesGame;

impl EcsSpritesGame {
    fn new() -> Self {
        Self
    }
}

impl GameLoop for EcsSpritesGame {
    fn init(&mut self, world: &mut World, _gfx: &Graphics) {
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
        // Render ALL entities automatically (circles, lines, AND texture sprites)
        draw.render_world(world);

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

/// Create the atlas with procedural textures (or load from files if available)
fn create_atlas() -> AtlasBuilder {
    // Try to load images from files first
    if let Ok(builder) = AtlasBuilder::new()
        .add_image("player", "assets/sprites/player.png")
        .and_then(|b| b.add_image("enemy", "assets/sprites/enemy.png"))
        .and_then(|b| b.add_image("pickup", "assets/sprites/pickup.png"))
        .and_then(|b| b.add_image("ring", "assets/sprites/ring.png"))
    {
        println!("Loaded sprite images from assets/sprites/");
        return builder;
    }

    // Fall back to procedural textures
    println!("Using procedural textures (place PNGs in assets/sprites/ for real images)");
    AtlasBuilder::new()
        .add_gradient("player", 64, 64, [50, 150, 255, 255], [150, 50, 255, 255])
        .add_gradient("enemy", 48, 48, [255, 100, 100, 255], [255, 50, 50, 255])
        .add_checkerboard("pickup", 32, 32, 8, [255, 255, 100, 255], [255, 200, 50, 255])
        .add_ring("ring", 64, 12, [100, 255, 200, 255])
}

fn main() {
    // Register the atlas with the app - the engine handles everything else!
    App::build(EcsSpritesGame::new())
        .title("Graviplex ECS Sprites Demo")
        .size(1200, 800)
        .camera(CameraConfig::centered().with_scale(1.0))
        .atlas(create_atlas())
        .run()
        .unwrap();
}
