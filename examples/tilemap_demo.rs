//! Tilemap Demo - Shows how to create and render tilemaps like Stardew Valley/Moonlighter
//!
//! This example demonstrates:
//! - Creating multi-layer tilemaps (ground, objects, overlay)
//! - Using the 15-tile autotile system for seamless terrain
//! - Tile collision detection
//! - Player movement with PlayerController component
//! - Camera following with smooth lag
//!
//! Uses the "Tiny Swords" free asset pack for visuals.
//!
//! Run with: cargo run --example tilemap_demo --features textures

use graviplex::prelude::*;
use graviplex::advanced::{AtlasBuilder, SpritePipeline, TextureAtlas};

const ASSETS: &str = "assets/Tiny Swords (Free Pack)";

/// Simple player marker component
struct Player;

/// Game state
struct TilemapGame {
    atlas: Option<TextureAtlas>,
    sprite_pipeline: Option<SpritePipeline>,
}

impl TilemapGame {
    fn new() -> Self {
        Self {
            atlas: None,
            sprite_pipeline: None,
        }
    }

    /// Build the texture atlas from Tiny Swords assets
    fn build_atlas() -> Result<AtlasBuilder, Box<dyn std::error::Error>> {
        let builder = AtlasBuilder::new()
            // Terrain tiles - we'll use solid colors to simulate the tilemap
            // (The actual Tiny Swords tileset is a single spritesheet that needs slicing)
            .add_solid_color("grass", 64, 64, [86, 125, 70, 255])
            .add_solid_color("dirt", 64, 64, [139, 90, 43, 255])
            .add_solid_color("water_0", 64, 64, [64, 164, 223, 255])  // isolated
            .add_solid_color("water_1", 64, 64, [74, 174, 233, 255])  // top
            .add_solid_color("water_2", 64, 64, [74, 174, 233, 255])  // right
            .add_solid_color("water_3", 64, 64, [84, 184, 243, 255])  // top+right
            .add_solid_color("water_4", 64, 64, [74, 174, 233, 255])  // bottom
            .add_solid_color("water_5", 64, 64, [84, 184, 243, 255])  // vertical
            .add_solid_color("water_6", 64, 64, [84, 184, 243, 255])  // right+bottom
            .add_solid_color("water_7", 64, 64, [94, 194, 253, 255])  // 3-way
            .add_solid_color("water_8", 64, 64, [74, 174, 233, 255])  // left
            .add_solid_color("water_9", 64, 64, [84, 184, 243, 255])  // top+left
            .add_solid_color("water_10", 64, 64, [84, 184, 243, 255]) // horizontal
            .add_solid_color("water_11", 64, 64, [94, 194, 253, 255]) // 3-way
            .add_solid_color("water_12", 64, 64, [84, 184, 243, 255]) // bottom+left
            .add_solid_color("water_13", 64, 64, [94, 194, 253, 255]) // 3-way
            .add_solid_color("water_14", 64, 64, [94, 194, 253, 255]) // 3-way
            .add_solid_color("water_15", 64, 64, [104, 204, 255, 255]) // center
            // Decorations from Tiny Swords
            .add_image("rock1", format!("{}/Terrain/Decorations/Rocks/Rock1.png", ASSETS))?
            .add_image("rock2", format!("{}/Terrain/Decorations/Rocks/Rock2.png", ASSETS))?
            .add_image("rock3", format!("{}/Terrain/Decorations/Rocks/Rock3.png", ASSETS))?
            .add_image("bush1", format!("{}/Terrain/Decorations/Bushes/Bushe1.png", ASSETS))?
            .add_image("bush2", format!("{}/Terrain/Decorations/Bushes/Bushe2.png", ASSETS))?
            // Buildings
            .add_image("house1", format!("{}/Buildings/Blue Buildings/House1.png", ASSETS))?
            .add_image("house2", format!("{}/Buildings/Blue Buildings/House2.png", ASSETS))?
            .add_image("tower", format!("{}/Buildings/Blue Buildings/Tower.png", ASSETS))?
            // Player character
            .add_image("player_idle", format!("{}/Units/Blue Units/Pawn/Pawn_Idle.png", ASSETS))?;
        
        Ok(builder)
    }
}

impl GameLoop for TilemapGame {
    fn init(&mut self, world: &mut World, gfx: &Graphics) {
        // Build the texture atlas
        let atlas_builder = match Self::build_atlas() {
            Ok(builder) => builder,
            Err(e) => {
                log::error!("Failed to load assets: {}", e);
                log::info!("Make sure the 'Tiny Swords (Free Pack)' assets are in the assets folder");
                // Fall back to procedural tiles only
                AtlasBuilder::new()
                    .add_solid_color("grass", 64, 64, [86, 125, 70, 255])
                    .add_solid_color("dirt", 64, 64, [139, 90, 43, 255])
                    .add_solid_color("water_0", 64, 64, [64, 164, 223, 255])
                    .add_solid_color("water_15", 64, 64, [104, 204, 255, 255])
                    .add_solid_color("rock1", 32, 32, [128, 128, 128, 255])
                    .add_solid_color("bush1", 32, 32, [50, 150, 50, 255])
                    .add_solid_color("house1", 96, 96, [100, 80, 60, 255])
                    .add_solid_color("player_idle", 32, 32, [100, 150, 255, 255])
            }
        };

        let atlas = gfx
            .build_atlas(atlas_builder, 2048)
            .expect("Failed to build texture atlas");

        println!("Atlas created: {}x{}", atlas.width, atlas.height);
        println!("Regions: {:?}", atlas.region_names().collect::<Vec<_>>());

        let sprite_pipeline = gfx.create_sprite_pipeline(&atlas);
        self.sprite_pipeline = Some(sprite_pipeline);

        // Create a 20x15 tilemap with 64x64 pixel tiles
        let tile_size = 64;
        let mut tilemap = Tilemap::new(20, 15, tile_size, tile_size);

        // =====================================================================
        // GROUND LAYER (z=0)
        // =====================================================================
        let mut ground = TileLayer::new("ground", 20, 15)
            .fill(Tile::new("grass"))
            .with_z_order(0);

        // Add some dirt path tiles
        for x in 5..15 {
            ground.set(x, 7, Tile::new("dirt"));
            ground.set(x, 8, Tile::new("dirt"));
        }
        // Vertical path
        for y in 3..12 {
            ground.set(10, y, Tile::new("dirt"));
        }

        // Add water pond - place tiles then autotile
        for y in 2..6 {
            for x in 2..7 {
                ground.set(x, y, Tile::new("water_0").with_collision(TileCollision::Water));
            }
        }
        
        // Apply autotiling to water
        let water_config = AutoTileConfig::new("water_");
        ground.apply_autotile(&water_config, |tile| {
            tile.region_name.starts_with("water_")
        });

        tilemap.add_layer(ground);

        // =====================================================================
        // OBJECTS LAYER (z=1) - Trees, rocks, buildings
        // =====================================================================
        let mut objects = TileLayer::new("objects", 20, 15).with_z_order(1);

        // Add rocks (solid collision)
        objects.set(8, 3, Tile::new("rock1").solid());
        objects.set(15, 5, Tile::new("rock2").solid());
        objects.set(17, 10, Tile::new("rock3").solid());
        objects.set(3, 12, Tile::new("rock1").solid());

        // Add bushes
        objects.set(12, 2, Tile::new("bush1").solid());
        objects.set(14, 4, Tile::new("bush2").solid());
        objects.set(6, 11, Tile::new("bush1").solid());

        // Add a house (larger sprite, uses single tile position)
        objects.set(16, 12, Tile::new("house1").solid());

        tilemap.add_layer(objects);

        // Spawn the tilemap entity
        world.spawn((
            Transform::from_position(Vec2::ZERO),
            tilemap,
            Visible,
        ));

        // =====================================================================
        // PLAYER - Using Tiny Swords pawn sprite with PlayerController
        // =====================================================================
        let player_start = Vec2::new(10.0 * tile_size as f32, 10.0 * tile_size as f32);
        world.spawn((
            Transform::from_position(player_start),
            Sprite::texture("player_idle", Vec2::new(192.0, 192.0), Color::WHITE).with_z_order(5),
            Velocity::new(0.0, 0.0),
            PlayerController::new(150.0),  // 150 pixels/second movement speed
            Visible,
            Player,
        ));

        self.atlas = Some(atlas);

        log::info!("Tilemap Demo initialized!");
        log::info!("  - Map size: 20x15 tiles ({}x{} pixels)", 20 * tile_size, 15 * tile_size);
        log::info!("  - Controls: WASD to move player");
    }

    fn update(&mut self, world: &mut World, res: &Resources) {
        let dt = res.time.delta();

        // Run player input system (reads WASD, sets velocity)
        player_input_system(world, &res.input);

        // Get tilemap for collision detection
        let tilemap_data: Option<(Vec2, Tilemap)> = {
            let mut query = world.query::<(&Transform, &Tilemap)>();
            let result = query.iter().next().map(|(_, (t, tm))| (t.position, tm.clone()));
            result
        };

        // Apply movement with tilemap collision
        for (_, (transform, velocity)) in world.query::<(&mut Transform, &Velocity)>().iter() {
            // Calculate new position
            let new_pos = transform.position + velocity.0 * dt;

            // Check collision with tilemap
            if let Some((tilemap_pos, tilemap)) = &tilemap_data {
                if !tilemap.is_blocked(new_pos, *tilemap_pos) {
                    transform.position = new_pos;
                } else {
                    // Try sliding along walls
                    let slide_x = Vec2::new(new_pos.x, transform.position.y);
                    let slide_y = Vec2::new(transform.position.x, new_pos.y);

                    if !tilemap.is_blocked(slide_x, *tilemap_pos) {
                        transform.position.x = new_pos.x;
                    }
                    if !tilemap.is_blocked(slide_y, *tilemap_pos) {
                        transform.position.y = new_pos.y;
                    }
                }
            } else {
                transform.position = new_pos;
            }
        }
    }

    fn camera_target(&self, world: &World) -> Option<[f32; 2]> {
        // Follow the player
        for (_, (transform, _)) in world.query::<(&Transform, &Player)>().iter() {
            return Some([transform.position.x, transform.position.y]);
        }
        None
    }

    fn camera_follow_config(&self) -> CameraFollow {
        CameraFollow::new()
            .with_smoothing(4.0)       // Lower = more lag behind player (shows speed)
            .with_max_offset(150.0)    // Camera can lag up to 150 pixels behind player
            .with_deadzone(2.0)        // Ignore sub-pixel movements
    }

    fn render(&mut self, world: &World, draw: &mut DrawContext) {
        let atlas = self.atlas.as_ref().unwrap();
        let pipeline = self.sprite_pipeline.as_mut().unwrap();

        let mut tile_count = 0;

        // Render tilemap manually since we have our own atlas/pipeline
        for (_, (transform, tilemap, _)) in world.query::<(&Transform, &Tilemap, &Visible)>().iter() {
            let tile_size = tilemap.tile_size();
            let position = transform.position;

            // Render all layers sorted by z-order
            for layer in tilemap.layers_sorted() {
                if !layer.visible || layer.opacity <= 0.0 {
                    continue;
                }

                let tint_alpha = layer.opacity;

                // Render all tiles in the layer
                for (tx, ty, tile) in layer.iter() {
                    if tile.is_empty() {
                        continue;
                    }

                    // Get region from atlas
                    if let Some(region) = atlas.get(&tile.region_name) {
                        // Calculate world position (tile center)
                        let world_x = position.x + (tx as f32 + 0.5) * tile_size.x;
                        let world_y = position.y + (ty as f32 + 0.5) * tile_size.y;

                        pipeline.draw(
                            [world_x, world_y],
                            [tile_size.x, tile_size.y],
                            region.uv_rect(),
                            [1.0, 1.0, 1.0, tint_alpha],
                            0.0,
                            layer.z_order,
                        );
                        tile_count += 1;
                    }
                }
            }
        }

        // Render player sprite
        for (_, (transform, sprite, _)) in world.query::<(&Transform, &Sprite, &Visible)>().iter() {
            if let SpriteShape::Texture { region_name, size } = sprite.shape {
                if let Some(region) = atlas.get(region_name) {
                    pipeline.draw(
                        [transform.position.x, transform.position.y],
                        [size.x * transform.scale.x, size.y * transform.scale.y],
                        region.uv_rect(),
                        sprite.color.into(),
                        transform.rotation,
                        sprite.z_order,
                    );
                }
            }
        }

        // Debug: draw a circle at origin to verify rendering works
        draw.circle((Vec2::new(640.0, 480.0), 50.0, Color::RED));

        // Flush all sprites to GPU
        pipeline.flush(&draw.state(), atlas);

        // Log once
        static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOGGED.swap(true, std::sync::atomic::Ordering::SeqCst) {
            println!("Rendered {} tiles", tile_count);
        }
    }
}

fn main() {
    // Camera will follow the player, so no need to set initial position
    App::build(TilemapGame::new())
        .title("Graviplex Tilemap Demo - Tiny Swords")
        .size(1280, 960)
        .camera(CameraConfig::centered().with_scale(1.0))
        .run()
        .unwrap();
}
