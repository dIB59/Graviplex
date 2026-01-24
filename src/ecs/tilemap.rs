//! Tilemap system for pixel-art games like Stardew Valley and Moonlighter.
//!
//! This module provides:
//! - [`Tilemap`] - Component representing a grid-based tile map
//! - [`TileLayer`] - Individual layer within a tilemap (ground, objects, etc.)
//! - [`Tile`] - Single tile with texture, collision, and animation support
//! - [`TilesetConfig`] - Configuration for tileset properties
//!
//! # Example
//!
//! ```ignore
//! use graviplex::prelude::*;
//!
//! // Create a simple tilemap
//! let mut tilemap = Tilemap::new(32, 32, 16, 16); // 32x32 tiles, 16x16 pixels each
//!
//! // Add a ground layer
//! let ground_layer = TileLayer::new("ground", 32, 32)
//!     .fill(Tile::new("grass"))
//!     .with_z_order(0);
//! tilemap.add_layer(ground_layer);
//!
//! // Add an objects layer (trees, rocks, etc.)
//! let objects_layer = TileLayer::new("objects", 32, 32)
//!     .with_z_order(1);
//! tilemap.add_layer(objects_layer);
//!
//! // Spawn the tilemap entity
//! world.spawn((Transform::from_position(Vec2::ZERO), tilemap, Visible));
//! ```
//!
//! # Architecture
//!
//! Tilemaps support multiple layers rendered in z-order:
//! - **Ground layer** (z=0): Grass, dirt, water, paths
//! - **Decoration layer** (z=1): Flowers, small rocks
//! - **Object layer** (z=2): Trees, buildings (can have collision)
//! - **Overlay layer** (z=10): Effects rendered on top
//!
//! Each tile can have:
//! - A texture region from the atlas
//! - Collision flags (solid, water, etc.)
//! - Animation frames with timing
//! - Auto-tile rules for seamless terrain transitions

use crate::core::math::Vec2;

// =============================================================================
// TILE
// =============================================================================

/// A single tile in a tilemap layer.
///
/// Tiles reference texture regions by name and can have collision and animation.
#[derive(Clone, Debug)]
pub struct Tile {
    /// Name of the texture region in the atlas.
    /// Empty string means no tile (transparent).
    pub region_name: String,
    /// Collision flags for this tile.
    pub collision: TileCollision,
    /// Animation data (if animated).
    pub animation: Option<TileAnimation>,
    /// Flip/rotation flags for tile variations.
    pub flip: TileFlip,
}

impl Tile {
    /// Creates a new tile with the given texture region.
    pub fn new(region_name: impl Into<String>) -> Self {
        Self {
            region_name: region_name.into(),
            collision: TileCollision::None,
            animation: None,
            flip: TileFlip::None,
        }
    }

    /// Creates an empty (transparent) tile.
    pub fn empty() -> Self {
        Self {
            region_name: String::new(),
            collision: TileCollision::None,
            animation: None,
            flip: TileFlip::None,
        }
    }

    /// Returns true if this tile has no texture.
    pub fn is_empty(&self) -> bool {
        self.region_name.is_empty()
    }

    /// Sets the collision type.
    pub fn with_collision(mut self, collision: TileCollision) -> Self {
        self.collision = collision;
        self
    }

    /// Makes this tile solid (blocks movement).
    pub fn solid(mut self) -> Self {
        self.collision = TileCollision::Solid;
        self
    }

    /// Sets the animation for this tile.
    pub fn with_animation(mut self, animation: TileAnimation) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Sets the flip flags.
    pub fn with_flip(mut self, flip: TileFlip) -> Self {
        self.flip = flip;
        self
    }

    /// Flips the tile horizontally.
    pub fn flip_h(mut self) -> Self {
        self.flip = match self.flip {
            TileFlip::None => TileFlip::Horizontal,
            TileFlip::Horizontal => TileFlip::None,
            TileFlip::Vertical => TileFlip::Both,
            TileFlip::Both => TileFlip::Vertical,
        };
        self
    }

    /// Flips the tile vertically.
    pub fn flip_v(mut self) -> Self {
        self.flip = match self.flip {
            TileFlip::None => TileFlip::Vertical,
            TileFlip::Vertical => TileFlip::None,
            TileFlip::Horizontal => TileFlip::Both,
            TileFlip::Both => TileFlip::Horizontal,
        };
        self
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::empty()
    }
}

// =============================================================================
// TILE COLLISION
// =============================================================================

/// Collision types for tiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileCollision {
    /// No collision - can walk through.
    #[default]
    None,
    /// Solid - blocks all movement.
    Solid,
    /// Water - blocks walking but allows swimming/boats.
    Water,
    /// Platform - solid from above, passable from below.
    Platform,
    /// Trigger - no collision but fires events when entered.
    Trigger,
    /// Custom collision with a user-defined ID.
    Custom(u8),
}

impl TileCollision {
    /// Returns true if this tile blocks movement.
    pub fn is_solid(&self) -> bool {
        matches!(self, TileCollision::Solid | TileCollision::Platform)
    }

    /// Returns true if this tile is water.
    pub fn is_water(&self) -> bool {
        matches!(self, TileCollision::Water)
    }
}

// =============================================================================
// TILE FLIP
// =============================================================================

/// Flip/mirror flags for tile rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileFlip {
    /// No flip.
    #[default]
    None,
    /// Flip horizontally.
    Horizontal,
    /// Flip vertically.
    Vertical,
    /// Flip both horizontally and vertically (180° rotation).
    Both,
}

impl TileFlip {
    /// Returns UV coordinate adjustments (u_flip, v_flip).
    /// Used to flip texture coordinates in the shader.
    pub fn uv_multipliers(&self) -> (f32, f32) {
        match self {
            TileFlip::None => (1.0, 1.0),
            TileFlip::Horizontal => (-1.0, 1.0),
            TileFlip::Vertical => (1.0, -1.0),
            TileFlip::Both => (-1.0, -1.0),
        }
    }
}

// =============================================================================
// TILE ANIMATION
// =============================================================================

/// Animation data for animated tiles.
///
/// # Example
///
/// ```ignore
/// // Water animation: 4 frames, 0.25s each
/// let water_anim = TileAnimation::new(vec!["water_1", "water_2", "water_3", "water_4"], 0.25);
///
/// let water_tile = Tile::new("water_1").with_animation(water_anim);
/// ```
#[derive(Clone, Debug)]
pub struct TileAnimation {
    /// Texture region names for each frame.
    pub frames: Vec<String>,
    /// Duration of each frame in seconds.
    pub frame_duration: f32,
    /// Current animation time (updated by animation system).
    pub current_time: f32,
}

impl TileAnimation {
    /// Creates a new tile animation.
    pub fn new(frames: Vec<impl Into<String>>, frame_duration: f32) -> Self {
        Self {
            frames: frames.into_iter().map(|f| f.into()).collect(),
            frame_duration,
            current_time: 0.0,
        }
    }

    /// Gets the current frame index.
    pub fn current_frame(&self) -> usize {
        if self.frames.is_empty() || self.frame_duration <= 0.0 {
            return 0;
        }
        let total_duration = self.frame_duration * self.frames.len() as f32;
        let time_in_cycle = self.current_time % total_duration;
        (time_in_cycle / self.frame_duration) as usize
    }

    /// Gets the current frame's region name.
    pub fn current_region(&self) -> Option<&str> {
        self.frames.get(self.current_frame()).map(|s| s.as_str())
    }

    /// Advances the animation by delta time.
    pub fn tick(&mut self, dt: f32) {
        self.current_time += dt;
    }
}

// =============================================================================
// TILE LAYER
// =============================================================================

/// A single layer in a tilemap.
///
/// Layers are rendered in z-order, allowing for ground, objects, and overlays.
///
/// # Example
///
/// ```ignore
/// let ground = TileLayer::new("ground", 32, 32)
///     .fill(Tile::new("grass"))
///     .with_z_order(0);
///
/// let objects = TileLayer::new("objects", 32, 32)
///     .with_z_order(1);
/// ```
#[derive(Clone, Debug)]
pub struct TileLayer {
    /// Layer name (for lookup).
    pub name: String,
    /// Width in tiles.
    pub width: u32,
    /// Height in tiles.
    pub height: u32,
    /// Z-order for rendering (lower = behind).
    pub z_order: i32,
    /// Opacity (0.0 = invisible, 1.0 = fully visible).
    pub opacity: f32,
    /// Whether this layer is visible.
    pub visible: bool,
    /// The tile data (row-major: index = y * width + x).
    tiles: Vec<Tile>,
}

impl TileLayer {
    /// Creates a new empty tile layer.
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            name: name.into(),
            width,
            height,
            z_order: 0,
            opacity: 1.0,
            visible: true,
            tiles: vec![Tile::empty(); size],
        }
    }

    /// Sets the z-order for this layer.
    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Sets the opacity for this layer.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Fills the entire layer with the given tile.
    pub fn fill(mut self, tile: Tile) -> Self {
        self.tiles = vec![tile; (self.width * self.height) as usize];
        self
    }

    /// Gets a tile at the given position.
    pub fn get(&self, x: u32, y: u32) -> Option<&Tile> {
        if x < self.width && y < self.height {
            Some(&self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Gets a mutable tile at the given position.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x < self.width && y < self.height {
            Some(&mut self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Sets a tile at the given position.
    pub fn set(&mut self, x: u32, y: u32, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
        }
    }

    /// Sets a tile by region name at the given position.
    pub fn set_region(&mut self, x: u32, y: u32, region_name: impl Into<String>) {
        self.set(x, y, Tile::new(region_name));
    }

    /// Fills a rectangular region with the given tile.
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, tile: Tile) {
        for ty in y..(y + height).min(self.height) {
            for tx in x..(x + width).min(self.width) {
                self.set(tx, ty, tile.clone());
            }
        }
    }

    /// Returns an iterator over all tiles with their positions.
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, &Tile)> {
        self.tiles.iter().enumerate().map(|(i, tile)| {
            let x = (i as u32) % self.width;
            let y = (i as u32) / self.width;
            (x, y, tile)
        })
    }

    /// Returns a mutable iterator over all tiles with their positions.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, u32, &mut Tile)> {
        let width = self.width;
        self.tiles.iter_mut().enumerate().map(move |(i, tile)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            (x, y, tile)
        })
    }

    /// Returns the number of non-empty tiles.
    pub fn tile_count(&self) -> usize {
        self.tiles.iter().filter(|t| !t.is_empty()).count()
    }
}

// =============================================================================
// TILEMAP COMPONENT
// =============================================================================

/// A tilemap component representing a grid-based tile world.
///
/// Tilemaps are composed of multiple layers rendered in z-order.
/// They support collision detection, animated tiles, and auto-tiling.
///
/// # Example
///
/// ```ignore
/// // Create a 64x64 tilemap with 16x16 pixel tiles
/// let mut tilemap = Tilemap::new(64, 64, 16, 16);
///
/// // Add layers
/// tilemap.add_layer(TileLayer::new("ground", 64, 64).fill(Tile::new("grass")).with_z_order(0));
/// tilemap.add_layer(TileLayer::new("objects", 64, 64).with_z_order(1));
///
/// // Spawn as an entity
/// world.spawn((Transform::from_position(Vec2::ZERO), tilemap, Visible));
/// ```
#[derive(Clone, Debug)]
pub struct Tilemap {
    /// Width of the tilemap in tiles.
    pub width: u32,
    /// Height of the tilemap in tiles.
    pub height: u32,
    /// Width of each tile in pixels.
    pub tile_width: u32,
    /// Height of each tile in pixels.
    pub tile_height: u32,
    /// Layers in this tilemap (sorted by z-order during rendering).
    layers: Vec<TileLayer>,
    /// Animation timer for animated tiles.
    pub animation_time: f32,
}

impl Tilemap {
    /// Creates a new tilemap with the given dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in tiles
    /// * `height` - Height in tiles
    /// * `tile_width` - Width of each tile in pixels
    /// * `tile_height` - Height of each tile in pixels
    pub fn new(width: u32, height: u32, tile_width: u32, tile_height: u32) -> Self {
        Self {
            width,
            height,
            tile_width,
            tile_height,
            layers: Vec::new(),
            animation_time: 0.0,
        }
    }

    /// Creates a square tilemap with square tiles.
    pub fn square(tiles: u32, tile_size: u32) -> Self {
        Self::new(tiles, tiles, tile_size, tile_size)
    }

    /// Returns the total width of the tilemap in pixels.
    pub fn pixel_width(&self) -> f32 {
        (self.width * self.tile_width) as f32
    }

    /// Returns the total height of the tilemap in pixels.
    pub fn pixel_height(&self) -> f32 {
        (self.height * self.tile_height) as f32
    }

    /// Returns the tile size as a Vec2.
    pub fn tile_size(&self) -> Vec2 {
        Vec2::new(self.tile_width as f32, self.tile_height as f32)
    }

    /// Converts world coordinates to tile coordinates.
    ///
    /// Returns None if the position is outside the tilemap bounds.
    pub fn world_to_tile(&self, world_pos: Vec2, tilemap_pos: Vec2) -> Option<(u32, u32)> {
        let local_pos = world_pos - tilemap_pos;
        let tx = (local_pos.x / self.tile_width as f32).floor();
        let ty = (local_pos.y / self.tile_height as f32).floor();

        if tx >= 0.0 && ty >= 0.0 && tx < self.width as f32 && ty < self.height as f32 {
            Some((tx as u32, ty as u32))
        } else {
            None
        }
    }

    /// Converts tile coordinates to world coordinates (tile center).
    pub fn tile_to_world(&self, tile_x: u32, tile_y: u32, tilemap_pos: Vec2) -> Vec2 {
        Vec2::new(
            tilemap_pos.x + (tile_x as f32 + 0.5) * self.tile_width as f32,
            tilemap_pos.y + (tile_y as f32 + 0.5) * self.tile_height as f32,
        )
    }

    /// Converts tile coordinates to world coordinates (tile top-left corner).
    pub fn tile_to_world_corner(&self, tile_x: u32, tile_y: u32, tilemap_pos: Vec2) -> Vec2 {
        Vec2::new(
            tilemap_pos.x + tile_x as f32 * self.tile_width as f32,
            tilemap_pos.y + tile_y as f32 * self.tile_height as f32,
        )
    }

    /// Adds a layer to the tilemap.
    pub fn add_layer(&mut self, layer: TileLayer) {
        self.layers.push(layer);
    }

    /// Gets a layer by name.
    pub fn layer(&self, name: &str) -> Option<&TileLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Gets a mutable layer by name.
    pub fn layer_mut(&mut self, name: &str) -> Option<&mut TileLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Gets a layer by index.
    pub fn layer_at(&self, index: usize) -> Option<&TileLayer> {
        self.layers.get(index)
    }

    /// Gets a mutable layer by index.
    pub fn layer_at_mut(&mut self, index: usize) -> Option<&mut TileLayer> {
        self.layers.get_mut(index)
    }

    /// Returns the number of layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Returns an iterator over all layers sorted by z-order.
    pub fn layers_sorted(&self) -> impl Iterator<Item = &TileLayer> {
        let mut sorted: Vec<_> = self.layers.iter().collect();
        sorted.sort_by_key(|l| l.z_order);
        sorted.into_iter()
    }

    /// Returns an iterator over all layers.
    pub fn layers(&self) -> impl Iterator<Item = &TileLayer> {
        self.layers.iter()
    }

    /// Returns a mutable iterator over all layers.
    pub fn layers_mut(&mut self) -> impl Iterator<Item = &mut TileLayer> {
        self.layers.iter_mut()
    }

    /// Checks if a world position collides with a solid tile.
    ///
    /// Checks all layers and returns the first collision found.
    pub fn check_collision(&self, world_pos: Vec2, tilemap_pos: Vec2) -> Option<TileCollision> {
        let (tx, ty) = self.world_to_tile(world_pos, tilemap_pos)?;
        
        for layer in &self.layers {
            if let Some(tile) = layer.get(tx, ty) {
                if tile.collision != TileCollision::None {
                    return Some(tile.collision);
                }
            }
        }
        None
    }

    /// Checks if a world position is blocked (solid collision).
    pub fn is_blocked(&self, world_pos: Vec2, tilemap_pos: Vec2) -> bool {
        self.check_collision(world_pos, tilemap_pos)
            .map(|c| c.is_solid())
            .unwrap_or(false)
    }

    /// Updates tile animations.
    ///
    /// Call this in your update loop to advance animated tiles.
    pub fn update_animations(&mut self, dt: f32) {
        self.animation_time += dt;
        
        for layer in &mut self.layers {
            for tile in layer.tiles.iter_mut() {
                if let Some(anim) = &mut tile.animation {
                    anim.tick(dt);
                }
            }
        }
    }
}

// =============================================================================
// AUTO-TILE SUPPORT (15-TILE SYSTEM)
// =============================================================================

/// Neighbor flags for auto-tiling (4-connected).
///
/// Used as a bitmask to determine which tile variant to use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NeighborFlags(pub u8);

impl NeighborFlags {
    pub const NONE: u8 = 0;
    pub const TOP: u8 = 1 << 0;    // bit 0
    pub const RIGHT: u8 = 1 << 1;  // bit 1
    pub const BOTTOM: u8 = 1 << 2; // bit 2
    pub const LEFT: u8 = 1 << 3;   // bit 3

    pub fn new() -> Self {
        Self(0)
    }

    pub fn with_top(mut self) -> Self {
        self.0 |= Self::TOP;
        self
    }

    pub fn with_right(mut self) -> Self {
        self.0 |= Self::RIGHT;
        self
    }

    pub fn with_bottom(mut self) -> Self {
        self.0 |= Self::BOTTOM;
        self
    }

    pub fn with_left(mut self) -> Self {
        self.0 |= Self::LEFT;
        self
    }

    pub fn has_top(&self) -> bool {
        self.0 & Self::TOP != 0
    }

    pub fn has_right(&self) -> bool {
        self.0 & Self::RIGHT != 0
    }

    pub fn has_bottom(&self) -> bool {
        self.0 & Self::BOTTOM != 0
    }

    pub fn has_left(&self) -> bool {
        self.0 & Self::LEFT != 0
    }

    /// Returns the tile index (0-15) for the 16-tile autotile system.
    pub fn tile_index(&self) -> u8 {
        self.0
    }
}

/// Auto-tile configuration for the 15-tile system (16 variants including isolated).
///
/// The 15-tile system uses 4 cardinal neighbors (top, right, bottom, left) to select
/// from 16 tile variants. This is the system used by RPG Maker and Stardew Valley.
///
/// # Tile Index Layout
///
/// The tile index is a 4-bit bitmask:
/// - Bit 0 (1): Top neighbor
/// - Bit 1 (2): Right neighbor  
/// - Bit 2 (4): Bottom neighbor
/// - Bit 3 (8): Left neighbor
///
/// Index | Binary | Neighbors        | Description
/// ------|--------|------------------|------------------
///   0   | 0000   | None             | Isolated tile
///   1   | 0001   | Top              | Bottom edge
///   2   | 0010   | Right            | Left edge
///   3   | 0011   | Top+Right        | Bottom-left corner
///   4   | 0100   | Bottom           | Top edge
///   5   | 0101   | Top+Bottom       | Vertical corridor
///   6   | 0110   | Right+Bottom     | Top-left corner
///   7   | 0111   | Top+Right+Bottom | Left edge (3-way)
///   8   | 1000   | Left             | Right edge
///   9   | 1001   | Top+Left         | Bottom-right corner
///  10   | 1010   | Right+Left       | Horizontal corridor
///  11   | 1011   | Top+Right+Left   | Bottom edge (3-way)
///  12   | 1100   | Bottom+Left      | Top-right corner
///  13   | 1101   | Top+Bottom+Left  | Right edge (3-way)
///  14   | 1110   | Right+Bottom+Left| Top edge (3-way)
///  15   | 1111   | All              | Center/interior
///
/// # Example
///
/// ```ignore
/// // Create autotile config with region prefix
/// let water = AutoTileConfig::new("water_"); // expects water_0 through water_15
///
/// // Or with explicit mapping
/// let grass = AutoTileConfig::with_regions([
///     "grass_isolated",    // 0: no neighbors
///     "grass_bottom",      // 1: top neighbor only
///     "grass_left",        // 2: right neighbor only
///     // ... etc
/// ]);
/// ```
#[derive(Clone, Debug)]
pub struct AutoTileConfig {
    /// Prefix for auto-generated region names (e.g., "grass_" -> "grass_0", "grass_1", etc.)
    pub prefix: String,
    /// Optional explicit region names for each of the 16 variants.
    /// If Some, overrides prefix-based naming.
    pub regions: Option<[String; 16]>,
}

impl AutoTileConfig {
    /// Creates a new auto-tile configuration with a region name prefix.
    ///
    /// The prefix is combined with the tile index (0-15) to form region names.
    /// For example, prefix "water_" produces regions "water_0" through "water_15".
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            regions: None,
        }
    }

    /// Creates an auto-tile configuration with explicit region names for all 16 variants.
    pub fn with_regions(regions: [impl Into<String>; 16]) -> Self {
        Self {
            prefix: String::new(),
            regions: Some(regions.map(|r| r.into())),
        }
    }

    /// Gets the region name for the given neighbor flags.
    pub fn get_region(&self, flags: NeighborFlags) -> String {
        let index = flags.tile_index() as usize;
        if let Some(ref regions) = self.regions {
            regions[index].clone()
        } else {
            format!("{}{}", self.prefix, index)
        }
    }

    /// Gets the region name for the given tile index (0-15).
    pub fn get_region_by_index(&self, index: u8) -> String {
        let index = (index & 0x0F) as usize; // Ensure 0-15 range
        if let Some(ref regions) = self.regions {
            regions[index].clone()
        } else {
            format!("{}{}", self.prefix, index)
        }
    }
}

impl TileLayer {
    /// Applies auto-tiling to the layer using the given configuration.
    ///
    /// This method examines each tile and its neighbors, then updates the
    /// tile's region name based on the auto-tile rules.
    ///
    /// # Arguments
    ///
    /// * `config` - The auto-tile configuration
    /// * `match_fn` - A function that returns true if a tile should be considered
    ///                a "matching" neighbor for auto-tiling purposes
    ///
    /// # Example
    ///
    /// ```ignore
    /// let water_config = AutoTileConfig::new("water_");
    ///
    /// // Auto-tile all water tiles
    /// layer.apply_autotile(&water_config, |tile| {
    ///     tile.region_name.starts_with("water")
    /// });
    /// ```
    pub fn apply_autotile<F>(&mut self, config: &AutoTileConfig, match_fn: F)
    where
        F: Fn(&Tile) -> bool,
    {
        // First pass: compute neighbor flags for each tile
        let mut updates: Vec<(u32, u32, String)> = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let tile = match self.get(x, y) {
                    Some(t) if match_fn(t) => t,
                    _ => continue,
                };

                // Skip empty tiles
                if tile.is_empty() {
                    continue;
                }

                let mut flags = NeighborFlags::new();

                // Check top neighbor (y - 1)
                if y > 0 {
                    if let Some(neighbor) = self.get(x, y - 1) {
                        if match_fn(neighbor) {
                            flags = flags.with_top();
                        }
                    }
                }

                // Check right neighbor (x + 1)
                if x + 1 < self.width {
                    if let Some(neighbor) = self.get(x + 1, y) {
                        if match_fn(neighbor) {
                            flags = flags.with_right();
                        }
                    }
                }

                // Check bottom neighbor (y + 1)
                if y + 1 < self.height {
                    if let Some(neighbor) = self.get(x, y + 1) {
                        if match_fn(neighbor) {
                            flags = flags.with_bottom();
                        }
                    }
                }

                // Check left neighbor (x - 1)
                if x > 0 {
                    if let Some(neighbor) = self.get(x - 1, y) {
                        if match_fn(neighbor) {
                            flags = flags.with_left();
                        }
                    }
                }

                let new_region = config.get_region(flags);
                updates.push((x, y, new_region));
            }
        }

        // Second pass: apply updates
        for (x, y, region) in updates {
            if let Some(tile) = self.get_mut(x, y) {
                tile.region_name = region;
            }
        }
    }

    /// Fills a region with auto-tiled tiles.
    ///
    /// This is a convenience method that fills a rectangular region with tiles
    /// and automatically applies auto-tiling.
    pub fn fill_autotile(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        config: &AutoTileConfig,
    ) {
        // Fill with base tiles first
        let base_region = config.get_region_by_index(0);
        for ty in y..(y + height).min(self.height) {
            for tx in x..(x + width).min(self.width) {
                self.set(tx, ty, Tile::new(base_region.clone()));
            }
        }

        // Apply auto-tiling
        self.apply_autotile(config, |tile| {
            // Match any tile that uses this autotile config
            if let Some(ref regions) = config.regions {
                regions.contains(&tile.region_name)
            } else {
                tile.region_name.starts_with(&config.prefix)
            }
        });
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_creation() {
        let tile = Tile::new("grass");
        assert_eq!(tile.region_name, "grass");
        assert_eq!(tile.collision, TileCollision::None);
        assert!(tile.animation.is_none());
    }

    #[test]
    fn test_tile_empty() {
        let tile = Tile::empty();
        assert!(tile.is_empty());
        assert_eq!(tile.region_name, "");
    }

    #[test]
    fn test_tile_with_collision() {
        let tile = Tile::new("wall").solid();
        assert_eq!(tile.collision, TileCollision::Solid);
        assert!(tile.collision.is_solid());
    }

    #[test]
    fn test_tile_flip() {
        let tile = Tile::new("grass").flip_h();
        assert_eq!(tile.flip, TileFlip::Horizontal);
        
        let tile = tile.flip_v();
        assert_eq!(tile.flip, TileFlip::Both);
    }

    #[test]
    fn test_tile_animation() {
        let mut anim = TileAnimation::new(vec!["water_1", "water_2", "water_3", "water_4"], 0.25);
        assert_eq!(anim.current_frame(), 0);
        assert_eq!(anim.current_region(), Some("water_1"));

        anim.tick(0.3);
        assert_eq!(anim.current_frame(), 1);
        assert_eq!(anim.current_region(), Some("water_2"));

        anim.tick(0.5);
        assert_eq!(anim.current_frame(), 3);
    }

    #[test]
    fn test_tile_layer_creation() {
        let layer = TileLayer::new("ground", 10, 10).with_z_order(0);
        assert_eq!(layer.name, "ground");
        assert_eq!(layer.width, 10);
        assert_eq!(layer.height, 10);
        assert_eq!(layer.z_order, 0);
    }

    #[test]
    fn test_tile_layer_fill() {
        let layer = TileLayer::new("ground", 10, 10).fill(Tile::new("grass"));
        assert_eq!(layer.tile_count(), 100);
        assert_eq!(layer.get(5, 5).unwrap().region_name, "grass");
    }

    #[test]
    fn test_tile_layer_set_get() {
        let mut layer = TileLayer::new("ground", 10, 10);
        layer.set(5, 5, Tile::new("water").with_collision(TileCollision::Water));
        
        let tile = layer.get(5, 5).unwrap();
        assert_eq!(tile.region_name, "water");
        assert!(tile.collision.is_water());
    }

    #[test]
    fn test_tile_layer_bounds() {
        let layer = TileLayer::new("ground", 10, 10);
        assert!(layer.get(9, 9).is_some());
        assert!(layer.get(10, 10).is_none());
        assert!(layer.get(100, 100).is_none());
    }

    #[test]
    fn test_tilemap_creation() {
        let tilemap = Tilemap::new(32, 32, 16, 16);
        assert_eq!(tilemap.width, 32);
        assert_eq!(tilemap.height, 32);
        assert_eq!(tilemap.tile_width, 16);
        assert_eq!(tilemap.tile_height, 16);
        assert_eq!(tilemap.pixel_width(), 512.0);
        assert_eq!(tilemap.pixel_height(), 512.0);
    }

    #[test]
    fn test_tilemap_world_to_tile() {
        let tilemap = Tilemap::new(10, 10, 16, 16);
        let tilemap_pos = Vec2::ZERO;

        assert_eq!(tilemap.world_to_tile(Vec2::new(8.0, 8.0), tilemap_pos), Some((0, 0)));
        assert_eq!(tilemap.world_to_tile(Vec2::new(24.0, 8.0), tilemap_pos), Some((1, 0)));
        assert_eq!(tilemap.world_to_tile(Vec2::new(159.0, 159.0), tilemap_pos), Some((9, 9)));
        assert_eq!(tilemap.world_to_tile(Vec2::new(160.0, 160.0), tilemap_pos), None);
        assert_eq!(tilemap.world_to_tile(Vec2::new(-1.0, 0.0), tilemap_pos), None);
    }

    #[test]
    fn test_tilemap_tile_to_world() {
        let tilemap = Tilemap::new(10, 10, 16, 16);
        let tilemap_pos = Vec2::ZERO;

        // Tile center
        let world = tilemap.tile_to_world(0, 0, tilemap_pos);
        assert_eq!(world, Vec2::new(8.0, 8.0));

        let world = tilemap.tile_to_world(1, 1, tilemap_pos);
        assert_eq!(world, Vec2::new(24.0, 24.0));
    }

    #[test]
    fn test_tilemap_layers() {
        let mut tilemap = Tilemap::new(10, 10, 16, 16);
        tilemap.add_layer(TileLayer::new("ground", 10, 10).with_z_order(0));
        tilemap.add_layer(TileLayer::new("objects", 10, 10).with_z_order(1));

        assert_eq!(tilemap.layer_count(), 2);
        assert!(tilemap.layer("ground").is_some());
        assert!(tilemap.layer("objects").is_some());
        assert!(tilemap.layer("nonexistent").is_none());
    }

    #[test]
    fn test_tilemap_collision() {
        let mut tilemap = Tilemap::new(10, 10, 16, 16);
        let mut layer = TileLayer::new("ground", 10, 10);
        layer.set(5, 5, Tile::new("wall").solid());
        tilemap.add_layer(layer);

        let tilemap_pos = Vec2::ZERO;
        
        // Center of tile (5, 5) = (88, 88)
        assert!(tilemap.is_blocked(Vec2::new(88.0, 88.0), tilemap_pos));
        // Adjacent tile should not be blocked
        assert!(!tilemap.is_blocked(Vec2::new(72.0, 72.0), tilemap_pos));
    }

    // =========================================================================
    // 15-TILE AUTOTILE TESTS
    // =========================================================================

    #[test]
    fn test_neighbor_flags_basic() {
        let flags = NeighborFlags::new();
        assert_eq!(flags.tile_index(), 0);
        assert!(!flags.has_top());
        assert!(!flags.has_right());
        assert!(!flags.has_bottom());
        assert!(!flags.has_left());
    }

    #[test]
    fn test_neighbor_flags_individual() {
        let top = NeighborFlags::new().with_top();
        assert_eq!(top.tile_index(), 1);
        assert!(top.has_top());

        let right = NeighborFlags::new().with_right();
        assert_eq!(right.tile_index(), 2);
        assert!(right.has_right());

        let bottom = NeighborFlags::new().with_bottom();
        assert_eq!(bottom.tile_index(), 4);
        assert!(bottom.has_bottom());

        let left = NeighborFlags::new().with_left();
        assert_eq!(left.tile_index(), 8);
        assert!(left.has_left());
    }

    #[test]
    fn test_neighbor_flags_combinations() {
        // Top + Right = 3 (bottom-left corner)
        let flags = NeighborFlags::new().with_top().with_right();
        assert_eq!(flags.tile_index(), 3);

        // Top + Bottom = 5 (vertical corridor)
        let flags = NeighborFlags::new().with_top().with_bottom();
        assert_eq!(flags.tile_index(), 5);

        // Right + Left = 10 (horizontal corridor)
        let flags = NeighborFlags::new().with_right().with_left();
        assert_eq!(flags.tile_index(), 10);

        // All neighbors = 15 (center/interior)
        let flags = NeighborFlags::new().with_top().with_right().with_bottom().with_left();
        assert_eq!(flags.tile_index(), 15);
    }

    #[test]
    fn test_autotile_config_prefix() {
        let config = AutoTileConfig::new("water_");
        
        assert_eq!(config.get_region_by_index(0), "water_0");
        assert_eq!(config.get_region_by_index(5), "water_5");
        assert_eq!(config.get_region_by_index(15), "water_15");

        let flags = NeighborFlags::new().with_top().with_bottom();
        assert_eq!(config.get_region(flags), "water_5");
    }

    #[test]
    fn test_autotile_config_explicit_regions() {
        let config = AutoTileConfig::with_regions([
            "isolated", "top_only", "right_only", "top_right",
            "bottom_only", "vertical", "right_bottom", "top_right_bottom",
            "left_only", "top_left", "horizontal", "top_right_left",
            "bottom_left", "top_bottom_left", "right_bottom_left", "center",
        ]);

        assert_eq!(config.get_region_by_index(0), "isolated");
        assert_eq!(config.get_region_by_index(5), "vertical");
        assert_eq!(config.get_region_by_index(10), "horizontal");
        assert_eq!(config.get_region_by_index(15), "center");
    }

    #[test]
    fn test_apply_autotile_isolated() {
        let mut layer = TileLayer::new("test", 3, 3);
        // Place a single tile in the center
        layer.set(1, 1, Tile::new("water_0"));

        let config = AutoTileConfig::new("water_");
        layer.apply_autotile(&config, |t| t.region_name.starts_with("water_"));

        // Single isolated tile should be index 0
        assert_eq!(layer.get(1, 1).unwrap().region_name, "water_0");
    }

    #[test]
    fn test_apply_autotile_horizontal_line() {
        let mut layer = TileLayer::new("test", 5, 3);
        // Place a horizontal line of tiles
        layer.set(1, 1, Tile::new("water_0"));
        layer.set(2, 1, Tile::new("water_0"));
        layer.set(3, 1, Tile::new("water_0"));

        let config = AutoTileConfig::new("water_");
        layer.apply_autotile(&config, |t| t.region_name.starts_with("water_"));

        // Left tile: has right neighbor only = 2
        assert_eq!(layer.get(1, 1).unwrap().region_name, "water_2");
        // Middle tile: has left and right = 10
        assert_eq!(layer.get(2, 1).unwrap().region_name, "water_10");
        // Right tile: has left neighbor only = 8
        assert_eq!(layer.get(3, 1).unwrap().region_name, "water_8");
    }

    #[test]
    fn test_apply_autotile_vertical_line() {
        let mut layer = TileLayer::new("test", 3, 5);
        // Place a vertical line of tiles
        layer.set(1, 1, Tile::new("water_0"));
        layer.set(1, 2, Tile::new("water_0"));
        layer.set(1, 3, Tile::new("water_0"));

        let config = AutoTileConfig::new("water_");
        layer.apply_autotile(&config, |t| t.region_name.starts_with("water_"));

        // Top tile: has bottom neighbor only = 4
        assert_eq!(layer.get(1, 1).unwrap().region_name, "water_4");
        // Middle tile: has top and bottom = 5
        assert_eq!(layer.get(1, 2).unwrap().region_name, "water_5");
        // Bottom tile: has top neighbor only = 1
        assert_eq!(layer.get(1, 3).unwrap().region_name, "water_1");
    }

    #[test]
    fn test_apply_autotile_2x2_block() {
        let mut layer = TileLayer::new("test", 4, 4);
        // Place a 2x2 block
        layer.set(1, 1, Tile::new("water_0"));
        layer.set(2, 1, Tile::new("water_0"));
        layer.set(1, 2, Tile::new("water_0"));
        layer.set(2, 2, Tile::new("water_0"));

        let config = AutoTileConfig::new("water_");
        layer.apply_autotile(&config, |t| t.region_name.starts_with("water_"));

        // Top-left: right + bottom = 6 (top-left corner appearance)
        assert_eq!(layer.get(1, 1).unwrap().region_name, "water_6");
        // Top-right: left + bottom = 12 (top-right corner appearance)
        assert_eq!(layer.get(2, 1).unwrap().region_name, "water_12");
        // Bottom-left: top + right = 3 (bottom-left corner appearance)
        assert_eq!(layer.get(1, 2).unwrap().region_name, "water_3");
        // Bottom-right: top + left = 9 (bottom-right corner appearance)
        assert_eq!(layer.get(2, 2).unwrap().region_name, "water_9");
    }

    #[test]
    fn test_apply_autotile_3x3_filled() {
        let mut layer = TileLayer::new("test", 5, 5);
        // Fill a 3x3 area
        for y in 1..=3 {
            for x in 1..=3 {
                layer.set(x, y, Tile::new("water_0"));
            }
        }

        let config = AutoTileConfig::new("water_");
        layer.apply_autotile(&config, |t| t.region_name.starts_with("water_"));

        // Center tile: all neighbors = 15
        assert_eq!(layer.get(2, 2).unwrap().region_name, "water_15");

        // Corners
        assert_eq!(layer.get(1, 1).unwrap().region_name, "water_6");  // top-left
        assert_eq!(layer.get(3, 1).unwrap().region_name, "water_12"); // top-right
        assert_eq!(layer.get(1, 3).unwrap().region_name, "water_3");  // bottom-left
        assert_eq!(layer.get(3, 3).unwrap().region_name, "water_9");  // bottom-right

        // Edges
        assert_eq!(layer.get(2, 1).unwrap().region_name, "water_14"); // top edge
        assert_eq!(layer.get(2, 3).unwrap().region_name, "water_11"); // bottom edge
        assert_eq!(layer.get(1, 2).unwrap().region_name, "water_7");  // left edge
        assert_eq!(layer.get(3, 2).unwrap().region_name, "water_13"); // right edge
    }

    #[test]
    fn test_fill_autotile() {
        let mut layer = TileLayer::new("test", 5, 5);
        let config = AutoTileConfig::new("grass_");
        
        layer.fill_autotile(1, 1, 3, 3, &config);

        // Center should be fully surrounded
        assert_eq!(layer.get(2, 2).unwrap().region_name, "grass_15");
        
        // Corners should have correct variants
        assert_eq!(layer.get(1, 1).unwrap().region_name, "grass_6");  // top-left
        assert_eq!(layer.get(3, 3).unwrap().region_name, "grass_9");  // bottom-right
    }
}
