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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub fn texture(region_name: &'static str, size: Vec2, tint: Color) -> Self {
        Self {
            shape: SpriteShape::Texture { region_name, size },
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpriteShape {
    /// Circle with given radius.
    Circle { radius: f32 },
    /// Rectangle with given size (width, height).
    Rect { size: Vec2 },
    /// Line from entity position to position + end_offset.
    Line { end_offset: Vec2 },
    /// Textured sprite from an atlas region.
    ///
    /// The `region_name` is an index or hash that maps to an [`AtlasRegion`].
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
        region_name: &'static str,
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

        assert!(matches!(
            texture_sprite.shape,
            SpriteShape::Texture {
                region_name: "player",
                ..
            }
        ));
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
                region_name: "test",
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
            region_name: "player",
            size: Vec2::new(32.0, 32.0),
        };
        let tex2 = SpriteShape::Texture {
            region_name: "player",
            size: Vec2::new(32.0, 32.0),
        };
        let tex3 = SpriteShape::Texture {
            region_name: "enemy",
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
