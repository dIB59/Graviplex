//! Built-in components for common game functionality.
//!
//! These components cover the most common use cases for 2D games:
//! - [`Transform`] - Position, rotation, and scale
//! - [`Velocity`] - Linear velocity for movement
//! - [`Sprite`] - Visual representation (shape + color)
//! - [`Visible`] - Marker for entities that should be rendered
//! - [`Lifetime`] - Auto-despawn after duration
//! - [`Despawn`] - Marker for entities to remove this frame
//!
//! # GPU Compatibility
//!
//! Components marked with `#[repr(C)]` and implementing `bytemuck::NoUninit`
//! can be efficiently uploaded to GPU buffers for instanced rendering.

use crate::core::color::Color;
use crate::core::math::Vec2;

// =============================================================================
// TRANSFORM
// =============================================================================

/// Position, rotation, and scale of an entity in world space.
///
/// This is the fundamental spatial component - most entities that exist
/// in the game world will have a Transform.
///
/// # Example
///
/// ```ignore
/// // Create at position
/// let transform = Transform::from_position(Vec2::new(100.0, 200.0));
///
/// // Full construction
/// let transform = Transform {
///     position: Vec2::new(100.0, 200.0),
///     rotation: 0.0,
///     scale: Vec2::ONE,
/// };
///
/// // With builder pattern
/// let transform = Transform::from_position(Vec2::ZERO)
///     .with_rotation(std::f32::consts::PI / 4.0)
///     .with_scale(Vec2::new(2.0, 2.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// World position in pixels.
    pub position: Vec2,
    /// Rotation in radians (counter-clockwise from +X axis).
    pub rotation: f32,
    /// Scale multiplier (1.0 = original size).
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    /// Creates a transform at the given position with default rotation and scale.
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Creates a transform at the given x, y coordinates.
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_position(Vec2::new(x, y))
    }

    /// Sets the rotation (in radians).
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale.
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Sets uniform scale (same for x and y).
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec2::new(scale, scale);
        self
    }

    /// Translates the transform by the given offset.
    pub fn translate(&mut self, offset: Vec2) {
        self.position += offset;
    }

    /// Rotates the transform by the given angle (in radians).
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }
}

// =============================================================================
// VELOCITY
// =============================================================================

/// Linear velocity component for entity movement.
///
/// Used by the [`movement_system`](super::movement_system) to update
/// [`Transform`] positions each frame.
///
/// # Example
///
/// ```ignore
/// // Entity moving right at 100 pixels/second
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity(Vec2::new(100.0, 0.0)),
/// ));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity(pub Vec2);

impl Velocity {
    /// Creates a new velocity.
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Creates a velocity from a direction and speed.
    pub fn from_direction(direction: Vec2, speed: f32) -> Self {
        Self(direction.normalize() * speed)
    }

    /// Returns the speed (magnitude of velocity).
    pub fn speed(&self) -> f32 {
        self.0.length()
    }

    /// Returns the direction (normalized velocity).
    pub fn direction(&self) -> Vec2 {
        self.0.normalize()
    }
}

impl From<Vec2> for Velocity {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

// =============================================================================
// ACCELERATION
// =============================================================================

/// Linear acceleration component for physics-based movement.
///
/// Used by the [`physics_system`] to update [`Velocity`] each frame.
///
/// # Example
///
/// ```ignore
/// // Entity with acceleration (e.g., car accelerating)
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity::new(0.0, 0.0),
///     Acceleration::new(10.0, 0.0), // Accelerate right
/// ));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Acceleration(pub Vec2);

impl Acceleration {
    /// Creates a new acceleration.
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Creates acceleration pointing in a direction.
    pub fn from_direction(direction: Vec2, magnitude: f32) -> Self {
        Self(direction.normalize() * magnitude)
    }

    /// Returns the magnitude of the acceleration.
    pub fn magnitude(&self) -> f32 {
        self.0.length()
    }
}

impl From<Vec2> for Acceleration {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

// =============================================================================
// GRAVITY
// =============================================================================

/// Gravity component for entities affected by gravity.
///
/// This is separate from global gravity so individual entities can
/// have different gravity effects (e.g., floating objects, underwater physics).
///
/// # Example
///
/// ```ignore
/// // Standard downward gravity
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity::new(0.0, 0.0),
///     Gravity::default(), // Standard gravity
/// ));
///
/// // Custom gravity (e.g., moon physics)
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity::new(0.0, 0.0),
///     Gravity::new(0.0, -163.2), // Moon gravity
/// ));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gravity(pub Vec2);

impl Gravity {
    /// Standard Earth-like gravity (downward).
    pub const STANDARD: Self = Self(Vec2 { x: 0.0, y: -980.0 });
    
    /// No gravity.
    pub const NONE: Self = Self(Vec2::ZERO);

    /// Creates a new gravity vector.
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Creates gravity with the standard downward direction but custom magnitude.
    pub fn with_strength(strength: f32) -> Self {
        Self(Vec2::new(0.0, -strength))
    }
}

impl Default for Gravity {
    fn default() -> Self {
        Self::STANDARD
    }
}

impl From<Vec2> for Gravity {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

// =============================================================================
// RIGIDBODY
// =============================================================================

/// Physics body type determining how an entity responds to physics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BodyType {
    /// Dynamic bodies are affected by forces, gravity, and collisions.
    #[default]
    Dynamic,
    /// Kinematic bodies move via velocity but aren't affected by forces.
    /// Good for platforms, elevators, player controllers.
    Kinematic,
    /// Static bodies don't move but can be collided with.
    /// Good for walls, floors, obstacles.
    Static,
}

/// RigidBody component for physics simulation.
///
/// Controls how an entity interacts with the physics system.
///
/// # Example
///
/// ```ignore
/// // Player character (kinematic - controlled directly)
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity::new(0.0, 0.0),
///     RigidBody::kinematic(),
/// ));
///
/// // Falling crate (dynamic - affected by physics)
/// world.spawn((
///     Transform::from_position(Vec2::new(0.0, 100.0)),
///     Velocity::new(0.0, 0.0),
///     Gravity::default(),
///     RigidBody::dynamic().with_mass(5.0),
/// ));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidBody {
    /// Type of physics body.
    pub body_type: BodyType,
    /// Mass in arbitrary units (affects force response).
    pub mass: f32,
    /// Linear drag coefficient (0 = no drag, 1 = heavy drag).
    pub drag: f32,
    /// Angular drag coefficient.
    pub angular_drag: f32,
    /// Bounciness (0 = no bounce, 1 = perfect bounce).
    pub restitution: f32,
    /// Friction coefficient for surface contacts.
    pub friction: f32,
    /// Whether gravity affects this body.
    pub gravity_scale: f32,
    /// Whether this body can rotate.
    pub freeze_rotation: bool,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            mass: 1.0,
            drag: 0.0,
            angular_drag: 0.05,
            restitution: 0.0,
            friction: 0.4,
            gravity_scale: 1.0,
            freeze_rotation: false,
        }
    }
}

impl RigidBody {
    /// Creates a dynamic rigid body.
    pub fn dynamic() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            ..Default::default()
        }
    }

    /// Creates a kinematic rigid body.
    pub fn kinematic() -> Self {
        Self {
            body_type: BodyType::Kinematic,
            gravity_scale: 0.0,
            ..Default::default()
        }
    }

    /// Creates a static rigid body.
    pub fn static_body() -> Self {
        Self {
            body_type: BodyType::Static,
            mass: f32::INFINITY,
            gravity_scale: 0.0,
            ..Default::default()
        }
    }

    /// Sets the mass.
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Sets the drag coefficient.
    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }

    /// Sets the bounciness.
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Sets the friction.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Sets the gravity scale.
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Freezes rotation.
    pub fn freeze_rotation(mut self) -> Self {
        self.freeze_rotation = true;
        self
    }

    /// Checks if this is a dynamic body.
    pub fn is_dynamic(&self) -> bool {
        self.body_type == BodyType::Dynamic
    }

    /// Checks if this is a kinematic body.
    pub fn is_kinematic(&self) -> bool {
        self.body_type == BodyType::Kinematic
    }

    /// Checks if this is a static body.
    pub fn is_static(&self) -> bool {
        self.body_type == BodyType::Static
    }
}

// =============================================================================
// SPRITE
// =============================================================================

/// Visual representation of an entity.
///
/// Defines how an entity should be rendered - its shape, color, and draw order.
///
/// # Example
///
/// ```ignore
/// // Red circle
/// let sprite = Sprite::circle(50.0, Color::RED);
///
/// // Blue rectangle with z-order
/// let sprite = Sprite::rect(100.0, 50.0, Color::BLUE).with_z_order(10);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    /// The shape to render.
    pub shape: SpriteShape,
    /// The color/tint.
    pub color: Color,
    /// Draw order (higher = rendered on top).
    pub z_order: i32,
}

impl Sprite {
    /// Creates a circle sprite.
    pub fn circle(radius: f32, color: Color) -> Self {
        Self {
            shape: SpriteShape::Circle { radius },
            color,
            z_order: 0,
        }
    }

    /// Creates a rectangle sprite.
    pub fn rect(width: f32, height: f32, color: Color) -> Self {
        Self {
            shape: SpriteShape::Rect {
                size: Vec2::new(width, height),
            },
            color,
            z_order: 0,
        }
    }

    /// Creates a line sprite (from transform position to offset).
    pub fn line(end_offset: Vec2, color: Color) -> Self {
        Self {
            shape: SpriteShape::Line { end_offset },
            color,
            z_order: 0,
        }
    }

    /// Creates a textured sprite from an atlas region.
    ///
    /// The `region_name` must match a region in the texture atlas.
    /// The `size` is the display size in world units.
    /// The `tint` color is multiplied with the texture color (use WHITE for no tint).
    pub fn texture(region_name: impl Into<String>, size: Vec2, tint: Color) -> Self {
        Self {
            shape: SpriteShape::Texture { region_name: region_name.into(), size },
            color: tint,
            z_order: 0,
        }
    }

    /// Sets the z-order (draw order).
    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Sets the color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::circle(10.0, Color::WHITE)
    }
}

/// Shape types for sprites.
#[derive(Clone, Debug, PartialEq)]
pub enum SpriteShape {
    /// Circle with given radius.
    Circle { radius: f32 },
    /// Rectangle with given size (width, height).
    Rect { size: Vec2 },
    /// Line from entity position to position + end_offset.
    Line { end_offset: Vec2 },
    /// Textured sprite from an atlas region.
    ///
    /// The `region_name` maps to an [`AtlasRegion`] in the texture atlas.
    /// Use with [`TextureAtlas`] for efficient batched rendering.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Create a textured sprite using the convenience constructor
    /// let sprite = Sprite::texture("player", Vec2::new(32.0, 32.0), Color::WHITE);
    /// ```
    Texture {
        /// Name of the region in the texture atlas.
        region_name: String,
        /// Display size in world units.
        size: Vec2,
    },
}

// =============================================================================
// MARKER COMPONENTS
// =============================================================================

/// Marker component for entities that should be rendered.
///
/// Only entities with both a [`Transform`] and [`Visible`] component
/// will be rendered by [`DrawContext::render_world`].
///
/// # Example
///
/// ```ignore
/// // Visible entity
/// world.spawn((Transform::default(), Sprite::circle(50.0, Color::RED), Visible));
///
/// // Invisible entity (won't be rendered)
/// world.spawn((Transform::default(), Sprite::circle(50.0, Color::RED)));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Visible;

/// Marker component for entities that should be despawned.
///
/// The [`despawn_system`](super::despawn_system) removes all entities
/// with this component at the end of each frame.
///
/// # Example
///
/// ```ignore
/// // Mark entity for despawn
/// world.insert(entity, Despawn)?;
///
/// // Later, in update:
/// despawn_system(&mut world);  // Entity is removed
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Despawn;

// =============================================================================
// LIFETIME
// =============================================================================

/// Component that tracks remaining lifetime before auto-despawn.
///
/// The [`lifetime_system`](super::lifetime_system) decrements this each frame
/// and marks entities for despawn when it reaches zero.
///
/// # Example
///
/// ```ignore
/// // Particle that lives for 2 seconds
/// world.spawn((
///     Transform::from_position(pos),
///     Velocity(Vec2::new(0.0, -50.0)),
///     Sprite::circle(5.0, Color::YELLOW),
///     Visible,
///     Lifetime(2.0),
/// ));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lifetime(pub f32);

impl Lifetime {
    /// Creates a lifetime component with the given duration in seconds.
    pub fn new(seconds: f32) -> Self {
        Self(seconds)
    }

    /// Returns true if the lifetime has expired.
    pub fn is_expired(&self) -> bool {
        self.0 <= 0.0
    }

    /// Decrements the lifetime by delta time.
    pub fn tick(&mut self, dt: f32) {
        self.0 -= dt;
    }
}

impl From<f32> for Lifetime {
    fn from(seconds: f32) -> Self {
        Self(seconds)
    }
}

// =============================================================================
// PLAYER CONTROLLER
// =============================================================================

/// Component for input-driven character movement.
///
/// Attach this to an entity with [`Transform`] and [`Velocity`] to enable
/// WASD/Arrow key movement via [`player_input_system`](super::player_input_system).
///
/// # Example
///
/// ```ignore
/// world.spawn((
///     Transform::from_position(Vec2::new(400.0, 300.0)),
///     Velocity::new(0.0, 0.0),
///     Sprite::texture("player", Vec2::new(64.0, 64.0), Color::WHITE),
///     PlayerController::new(200.0),  // 200 pixels/second
///     Visible,
/// ));
///
/// // In update:
/// systems::player_input(world, &res.input);
/// systems::movement(world, res.time.delta());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerController {
    /// Movement speed in units per second
    pub speed: f32,
    /// Whether diagonal movement should be normalized (same speed in all directions)
    pub normalize_diagonal: bool,
}

impl PlayerController {
    /// Create a new player controller with the given speed.
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            normalize_diagonal: true,
        }
    }

    /// Set whether to normalize diagonal movement.
    ///
    /// When true (default), diagonal movement has the same speed as cardinal movement.
    /// When false, diagonal movement is ~1.41x faster.
    pub fn with_diagonal_normalized(mut self, normalize: bool) -> Self {
        self.normalize_diagonal = normalize;
        self
    }
}

impl Default for PlayerController {
    fn default() -> Self {
        Self::new(200.0)
    }
}

// =============================================================================
// SPRITE ANIMATION
// =============================================================================

/// Component for animating sprites through a sequence of frames.
///
/// Works with sprite sheets that have been sliced using
/// [`AtlasBuilder::add_sprite_sheet`]. The animation cycles through
/// frames named `{prefix}_0`, `{prefix}_1`, etc.
///
/// Use [`animation_system`](super::animation_system) to update animations each frame.
///
/// # Example
///
/// ```ignore
/// // Load sprite sheet with 6 frames
/// let atlas = AtlasBuilder::new()
///     .add_sprite_sheet("player_walk", "assets/walk.png", 64, 64)?
///     .build(&gfx, 2048)?;
///
/// // Spawn animated sprite
/// world.spawn((
///     Transform::from_position(pos),
///     Sprite::texture("player_walk_0", Vec2::new(64.0, 64.0), Color::WHITE),
///     SpriteAnimation::new("player_walk", 6).with_fps(12.0),
///     Visible,
/// ));
///
/// // In update:
/// systems::animation(world, res.time.delta());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteAnimation {
    /// Prefix for frame names (frames are named "{prefix}_0", "{prefix}_1", etc.)
    pub prefix: String,
    /// Total number of frames in the animation
    pub frame_count: u32,
    /// Current frame index (0-based)
    pub current_frame: u32,
    /// Time accumulated since last frame change
    pub elapsed: f32,
    /// Seconds per frame (1.0 / fps)
    pub frame_duration: f32,
    /// Whether the animation should loop
    pub looping: bool,
    /// Whether the animation is playing
    pub playing: bool,
}

impl SpriteAnimation {
    /// Create a new animation with the given prefix and frame count.
    ///
    /// Default settings: 10 FPS, looping, playing.
    pub fn new(prefix: impl Into<String>, frame_count: u32) -> Self {
        Self {
            prefix: prefix.into(),
            frame_count,
            current_frame: 0,
            elapsed: 0.0,
            frame_duration: 0.1, // 10 FPS default
            looping: true,
            playing: true,
        }
    }

    /// Set the animation speed in frames per second.
    pub fn with_fps(mut self, fps: f32) -> Self {
        self.frame_duration = 1.0 / fps.max(0.001);
        self
    }

    /// Set the frame duration directly (seconds per frame).
    pub fn with_frame_duration(mut self, duration: f32) -> Self {
        self.frame_duration = duration;
        self
    }

    /// Set whether the animation loops.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Start playing the animation from the beginning.
    pub fn play(&mut self) {
        self.playing = true;
        self.current_frame = 0;
        self.elapsed = 0.0;
    }

    /// Stop the animation.
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Pause the animation (keeps current frame).
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resume a paused animation.
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// Get the current frame's region name (e.g., "player_walk_3").
    pub fn current_region_name(&self) -> String {
        format!("{}_{}", self.prefix, self.current_frame)
    }

    /// Check if the animation has finished (only relevant for non-looping animations).
    pub fn is_finished(&self) -> bool {
        !self.looping && self.current_frame >= self.frame_count - 1
    }

    /// Update the animation state. Called by [`animation_system`](super::animation_system).
    pub fn tick(&mut self, dt: f32) {
        if !self.playing {
            return;
        }

        self.elapsed += dt;

        while self.elapsed >= self.frame_duration {
            self.elapsed -= self.frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.frame_count {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frame_count - 1;
                    self.playing = false;
                }
            }
        }
    }
}

// =============================================================================
// TAGS
// =============================================================================

/// A set of string tags for categorizing entities.
///
/// Tags allow flexible entity categorization without creating new component types.
/// Useful for identifying entity types (Player, Enemy, Projectile), groups (Team1, Team2),
/// or states (Invincible, Stunned).
///
/// # Example
///
/// ```ignore
/// // Create entity with tags
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Tags::new().with("player").with("controllable"),
/// ));
///
/// // Query entities by tag
/// for (entity, (transform, tags)) in world.query::<(&Transform, &Tags)>().iter() {
///     if tags.has("enemy") {
///         // Handle enemy
///     }
/// }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tags {
    tags: std::collections::HashSet<String>,
}

impl Tags {
    /// Create empty tags.
    pub fn new() -> Self {
        Self {
            tags: std::collections::HashSet::new(),
        }
    }

    /// Create tags from a slice of tag names.
    pub fn from_slice(tags: &[&str]) -> Self {
        Self {
            tags: tags.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    /// Add a tag (builder pattern).
    pub fn with(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Add a tag.
    pub fn add(&mut self, tag: impl Into<String>) {
        self.tags.insert(tag.into());
    }

    /// Remove a tag.
    pub fn remove(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    /// Check if a tag exists.
    pub fn has(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Check if all given tags exist.
    pub fn has_all(&self, tags: &[&str]) -> bool {
        tags.iter().all(|t| self.tags.contains(*t))
    }

    /// Check if any of the given tags exist.
    pub fn has_any(&self, tags: &[&str]) -> bool {
        tags.iter().any(|t| self.tags.contains(*t))
    }

    /// Get the number of tags.
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// Check if there are no tags.
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Clear all tags.
    pub fn clear(&mut self) {
        self.tags.clear();
    }

    /// Iterate over all tags.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.tags.iter()
    }
}

// =============================================================================
// NAME
// =============================================================================

/// A name component for identifying entities.
///
/// Useful for debugging and for finding specific entities by name.
///
/// # Example
///
/// ```ignore
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Name::new("Player 1"),
/// ));
///
/// // Find entity by name
/// for (entity, name) in world.query::<&Name>().iter() {
///     if name.as_str() == "Player 1" {
///         // Found the player
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    /// Create a new name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =============================================================================
// HEALTH
// =============================================================================

/// Health component for entities that can take damage and die.
///
/// # Example
///
/// ```ignore
/// // Player with 100 HP
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Health::new(100.0),
///     Tags::new().with("player"),
/// ));
///
/// // In damage system
/// for (entity, health) in world.query::<&mut Health>().iter() {
///     if health.is_dead() {
///         // Handle death
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Health {
    /// Current health points.
    pub current: f32,
    /// Maximum health points.
    pub max: f32,
}

impl Health {
    /// Create a new health component.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Create with specific current and max values.
    pub fn with_current(current: f32, max: f32) -> Self {
        Self {
            current: current.min(max),
            max,
        }
    }

    /// Deal damage to this health.
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    /// Heal this health.
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Set current health to max.
    pub fn restore_full(&mut self) {
        self.current = self.max;
    }

    /// Check if dead (health <= 0).
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Check if at full health.
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Get health as a percentage (0.0 - 1.0).
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_builders() {
        let t = Transform::from_xy(10.0, 20.0)
            .with_rotation(1.0)
            .with_uniform_scale(2.0);

        assert_eq!(t.position, Vec2::new(10.0, 20.0));
        assert_eq!(t.rotation, 1.0);
        assert_eq!(t.scale, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn test_transform_translate() {
        let mut t = Transform::from_position(Vec2::new(10.0, 10.0));
        t.translate(Vec2::new(5.0, -3.0));
        assert_eq!(t.position, Vec2::new(15.0, 7.0));
    }

    #[test]
    fn test_velocity_from_direction() {
        let v = Velocity::from_direction(Vec2::new(1.0, 0.0), 100.0);
        assert!((v.speed() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_sprite_shapes() {
        let circle = Sprite::circle(50.0, Color::RED);
        assert!(matches!(circle.shape, SpriteShape::Circle { radius: 50.0 }));

        let rect = Sprite::rect(100.0, 50.0, Color::BLUE);
        assert!(matches!(rect.shape, SpriteShape::Rect { .. }));
    }

    #[test]
    fn test_sprite_texture_shape() {
        let texture_sprite = Sprite::texture("player", Vec2::new(64.0, 64.0), Color::WHITE);
        if let SpriteShape::Texture { region_name, size } = &texture_sprite.shape {
            assert_eq!(region_name, "player");
            assert_eq!(*size, Vec2::new(64.0, 64.0));
        } else {
            panic!("Expected Texture shape");
        }
        assert_eq!(texture_sprite.color, Color::WHITE);
        assert_eq!(texture_sprite.z_order, 0);
    }

    #[test]
    fn test_sprite_texture_with_tint() {
        let tinted = Sprite::texture("enemy", Vec2::new(32.0, 32.0), Color::RED);

        assert_eq!(tinted.color, Color::RED);
        if let SpriteShape::Texture { region_name, size } = tinted.shape {
            assert_eq!(region_name, "enemy");
            assert_eq!(size, Vec2::new(32.0, 32.0));
        } else {
            panic!("Expected Texture shape");
        }
    }

    #[test]
    fn test_sprite_texture_with_z_order() {
        let sprite = Sprite::texture("bullet", Vec2::new(8.0, 8.0), Color::rgb(1.0, 1.0, 0.0)).with_z_order(100);

        assert_eq!(sprite.z_order, 100);
    }

    #[test]
    fn test_sprite_shape_all_variants() {
        // Ensure all variants can be constructed and matched
        let shapes = vec![
            SpriteShape::Circle { radius: 10.0 },
            SpriteShape::Rect {
                size: Vec2::new(20.0, 30.0),
            },
            SpriteShape::Line {
                end_offset: Vec2::new(50.0, 0.0),
            },
            SpriteShape::Texture {
                region_name: "test".to_string(),
                size: Vec2::new(32.0, 32.0),
            },
        ];

        assert_eq!(shapes.len(), 4);

        for shape in shapes {
            match shape {
                SpriteShape::Circle { radius } => assert!(radius > 0.0),
                SpriteShape::Rect { size } => assert!(size.x > 0.0 && size.y > 0.0),
                SpriteShape::Line { end_offset } => assert!(end_offset.length() > 0.0),
                SpriteShape::Texture { region_name, size } => {
                    assert!(!region_name.is_empty());
                    assert!(size.x > 0.0 && size.y > 0.0);
                }
            }
        }
    }

    #[test]
    fn test_sprite_shape_partial_eq() {
        let shape1 = SpriteShape::Circle { radius: 10.0 };
        let shape2 = SpriteShape::Circle { radius: 10.0 };
        let shape3 = SpriteShape::Circle { radius: 20.0 };

        assert_eq!(shape1, shape2);
        assert_ne!(shape1, shape3);

        let tex1 = SpriteShape::Texture {
            region_name: "player".to_string(),
            size: Vec2::new(32.0, 32.0),
        };
        let tex2 = SpriteShape::Texture {
            region_name: "player".to_string(),
            size: Vec2::new(32.0, 32.0),
        };
        let tex3 = SpriteShape::Texture {
            region_name: "enemy".to_string(),
            size: Vec2::new(32.0, 32.0),
        };

        assert_eq!(tex1, tex2);
        assert_ne!(tex1, tex3);
    }

    #[test]
    fn test_lifetime() {
        let mut lt = Lifetime::new(1.0);
        assert!(!lt.is_expired());

        lt.tick(0.5);
        assert!(!lt.is_expired());
        assert_eq!(lt.0, 0.5);

        lt.tick(0.6);
        assert!(lt.is_expired());
    }
}
