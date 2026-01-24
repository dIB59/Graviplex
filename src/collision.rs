//! Collision detection and response for 2D games.
//!
//! This module provides:
//! - [`Collider`] component for defining collision shapes
//! - [`CollisionLayer`] for filtering collisions
//! - [`CollisionEvent`] for collision notifications
//! - Collision detection systems
//!
//! # Example
//!
//! ```ignore
//! use graviplex::prelude::*;
//! use graviplex::collision::{Collider, CollisionLayer, collision_system};
//!
//! // Define collision layers
//! const PLAYER: CollisionLayer = CollisionLayer::new(0);
//! const ENEMY: CollisionLayer = CollisionLayer::new(1);
//! const PROJECTILE: CollisionLayer = CollisionLayer::new(2);
//!
//! // Spawn player with collider
//! world.spawn((
//!     Transform::from_position(Vec2::ZERO),
//!     Collider::circle(32.0)
//!         .with_layer(PLAYER)
//!         .collides_with(ENEMY | PROJECTILE),
//!     Sprite::circle(32.0, Color::BLUE),
//!     Visible,
//! ));
//!
//! // In update:
//! let events = collision_system(world);
//! for event in events {
//!     println!("Collision between {:?} and {:?}", event.entity_a, event.entity_b);
//! }
//! ```

use crate::core::math::Vec2;
use crate::ecs::{Entity, Transform, World};
use std::ops::{BitAnd, BitOr};

// =============================================================================
// COLLISION LAYER
// =============================================================================

/// A collision layer mask for filtering which entities can collide.
///
/// Use bitwise OR to combine layers: `PLAYER | ENEMY`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CollisionLayer(pub u32);

impl CollisionLayer {
    /// No collision layer (collides with nothing).
    pub const NONE: Self = Self(0);
    
    /// All collision layers.
    pub const ALL: Self = Self(u32::MAX);

    /// Create a collision layer from a bit index (0-31).
    #[inline]
    pub const fn new(bit: u32) -> Self {
        debug_assert!(bit < 32, "Collision layer bit must be 0-31");
        Self(1 << bit)
    }

    /// Check if this layer overlaps with another layer mask.
    #[inline]
    pub fn overlaps(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Check if this layer contains a specific layer.
    #[inline]
    pub fn contains(self, layer: Self) -> bool {
        (self.0 & layer.0) == layer.0
    }
}

impl BitOr for CollisionLayer {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for CollisionLayer {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

// =============================================================================
// COLLISION SHAPES
// =============================================================================

/// Shape used for collision detection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColliderShape {
    /// Circle collider with radius.
    Circle { radius: f32 },
    
    /// Axis-aligned bounding box (rectangle).
    AABB { half_extents: Vec2 },
    
    /// Point collider (useful for bullets, etc).
    Point,
}

impl ColliderShape {
    /// Create a circle shape.
    #[inline]
    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    /// Create an AABB shape.
    #[inline]
    pub fn aabb(width: f32, height: f32) -> Self {
        Self::AABB {
            half_extents: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    /// Create a point shape.
    #[inline]
    pub fn point() -> Self {
        Self::Point
    }
}

// =============================================================================
// COLLIDER COMPONENT
// =============================================================================

/// Component that enables collision detection for an entity.
///
/// # Example
///
/// ```ignore
/// // Circle collider
/// Collider::circle(32.0)
///
/// // Rectangle collider
/// Collider::aabb(64.0, 32.0)
///
/// // With layer configuration
/// Collider::circle(32.0)
///     .with_layer(PLAYER_LAYER)
///     .collides_with(ENEMY_LAYER | PICKUP_LAYER)
///     .as_trigger()  // No physics response, just detection
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collider {
    /// The collision shape.
    pub shape: ColliderShape,
    
    /// The layer this collider belongs to.
    pub layer: CollisionLayer,
    
    /// Which layers this collider can collide with.
    pub mask: CollisionLayer,
    
    /// Offset from the entity's transform position.
    pub offset: Vec2,
    
    /// Whether this is a trigger (no physics response, just detection).
    pub is_trigger: bool,
    
    /// Whether this collider is currently enabled.
    pub enabled: bool,
}

impl Collider {
    /// Create a circle collider.
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape::circle(radius),
            layer: CollisionLayer::ALL,
            mask: CollisionLayer::ALL,
            offset: Vec2::ZERO,
            is_trigger: false,
            enabled: true,
        }
    }

    /// Create an AABB (rectangle) collider.
    pub fn aabb(width: f32, height: f32) -> Self {
        Self {
            shape: ColliderShape::aabb(width, height),
            layer: CollisionLayer::ALL,
            mask: CollisionLayer::ALL,
            offset: Vec2::ZERO,
            is_trigger: false,
            enabled: true,
        }
    }

    /// Create a point collider.
    pub fn point() -> Self {
        Self {
            shape: ColliderShape::point(),
            layer: CollisionLayer::ALL,
            mask: CollisionLayer::ALL,
            offset: Vec2::ZERO,
            is_trigger: false,
            enabled: true,
        }
    }

    /// Set the collision layer this collider belongs to.
    pub fn with_layer(mut self, layer: CollisionLayer) -> Self {
        self.layer = layer;
        self
    }

    /// Set which layers this collider can interact with.
    pub fn collides_with(mut self, mask: CollisionLayer) -> Self {
        self.mask = mask;
        self
    }

    /// Set the offset from the entity's transform position.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Make this collider a trigger (detection only, no physics response).
    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self
    }

    /// Enable or disable the collider.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if this collider can interact with another based on layers.
    pub fn can_collide_with(&self, other: &Collider) -> bool {
        self.enabled
            && other.enabled
            && self.layer.overlaps(other.mask)
            && other.layer.overlaps(self.mask)
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self::circle(1.0)
    }
}

// =============================================================================
// COLLISION EVENT
// =============================================================================

/// Information about a collision between two entities.
#[derive(Clone, Copy, Debug)]
pub struct CollisionEvent {
    /// First entity in the collision.
    pub entity_a: Entity,
    /// Second entity in the collision.
    pub entity_b: Entity,
    /// Penetration depth (how much they overlap).
    pub penetration: f32,
    /// Normal vector from A to B.
    pub normal: Vec2,
    /// Contact point in world space.
    pub contact: Vec2,
}

impl CollisionEvent {
    /// Check if a specific entity is involved in this collision.
    pub fn involves(&self, entity: Entity) -> bool {
        self.entity_a == entity || self.entity_b == entity
    }

    /// Get the "other" entity given one of the collision participants.
    pub fn other(&self, entity: Entity) -> Option<Entity> {
        if self.entity_a == entity {
            Some(self.entity_b)
        } else if self.entity_b == entity {
            Some(self.entity_a)
        } else {
            None
        }
    }
}

// =============================================================================
// COLLISION DETECTION
// =============================================================================

/// Check for collision between two circles.
fn circle_vs_circle(
    pos_a: Vec2,
    radius_a: f32,
    pos_b: Vec2,
    radius_b: f32,
) -> Option<(f32, Vec2)> {
    let diff = pos_b - pos_a;
    let dist_sq = diff.length_squared();
    let min_dist = radius_a + radius_b;

    if dist_sq < min_dist * min_dist {
        let dist = dist_sq.sqrt();
        let penetration = min_dist - dist;
        let normal = if dist > 0.0001 {
            diff / dist
        } else {
            Vec2::UP // Arbitrary normal for perfectly overlapping circles
        };
        Some((penetration, normal))
    } else {
        None
    }
}

/// Check for collision between two AABBs.
fn aabb_vs_aabb(
    pos_a: Vec2,
    half_a: Vec2,
    pos_b: Vec2,
    half_b: Vec2,
) -> Option<(f32, Vec2)> {
    let diff = pos_b - pos_a;
    
    let overlap_x = (half_a.x + half_b.x) - diff.x.abs();
    let overlap_y = (half_a.y + half_b.y) - diff.y.abs();

    if overlap_x > 0.0 && overlap_y > 0.0 {
        // Choose the axis with smallest overlap
        if overlap_x < overlap_y {
            let normal = Vec2::new(if diff.x > 0.0 { 1.0 } else { -1.0 }, 0.0);
            Some((overlap_x, normal))
        } else {
            let normal = Vec2::new(0.0, if diff.y > 0.0 { 1.0 } else { -1.0 });
            Some((overlap_y, normal))
        }
    } else {
        None
    }
}

/// Check for collision between a circle and an AABB.
fn circle_vs_aabb(
    circle_pos: Vec2,
    radius: f32,
    aabb_pos: Vec2,
    half_extents: Vec2,
) -> Option<(f32, Vec2)> {
    // Find the closest point on the AABB to the circle center
    let closest = Vec2::new(
        circle_pos.x.clamp(aabb_pos.x - half_extents.x, aabb_pos.x + half_extents.x),
        circle_pos.y.clamp(aabb_pos.y - half_extents.y, aabb_pos.y + half_extents.y),
    );

    let diff = circle_pos - closest;
    let dist_sq = diff.length_squared();

    if dist_sq < radius * radius {
        let dist = dist_sq.sqrt();
        let penetration = radius - dist;
        let normal = if dist > 0.0001 {
            diff / dist
        } else {
            // Circle center is inside AABB, find closest edge
            let dx = half_extents.x - (circle_pos.x - aabb_pos.x).abs();
            let dy = half_extents.y - (circle_pos.y - aabb_pos.y).abs();
            if dx < dy {
                Vec2::new(if circle_pos.x > aabb_pos.x { 1.0 } else { -1.0 }, 0.0)
            } else {
                Vec2::new(0.0, if circle_pos.y > aabb_pos.y { 1.0 } else { -1.0 })
            }
        };
        Some((penetration, normal))
    } else {
        None
    }
}

/// Check collision between two colliders at given positions.
pub fn check_collision(
    collider_a: &Collider,
    pos_a: Vec2,
    collider_b: &Collider,
    pos_b: Vec2,
) -> Option<(f32, Vec2, Vec2)> {
    if !collider_a.can_collide_with(collider_b) {
        return None;
    }

    let actual_pos_a = pos_a + collider_a.offset;
    let actual_pos_b = pos_b + collider_b.offset;

    let result = match (&collider_a.shape, &collider_b.shape) {
        (ColliderShape::Circle { radius: r_a }, ColliderShape::Circle { radius: r_b }) => {
            circle_vs_circle(actual_pos_a, *r_a, actual_pos_b, *r_b)
        }

        (ColliderShape::AABB { half_extents: h_a }, ColliderShape::AABB { half_extents: h_b }) => {
            aabb_vs_aabb(actual_pos_a, *h_a, actual_pos_b, *h_b)
        }

        (ColliderShape::Circle { radius }, ColliderShape::AABB { half_extents }) => {
            circle_vs_aabb(actual_pos_a, *radius, actual_pos_b, *half_extents)
        }

        (ColliderShape::AABB { half_extents }, ColliderShape::Circle { radius }) => {
            circle_vs_aabb(actual_pos_b, *radius, actual_pos_a, *half_extents)
                .map(|(p, n)| (p, -n))
        }

        (ColliderShape::Point, ColliderShape::Circle { radius }) => {
            let dist = actual_pos_a.distance(actual_pos_b);
            if dist < *radius {
                let normal = (actual_pos_a - actual_pos_b).normalize();
                Some((radius - dist, normal))
            } else {
                None
            }
        }

        (ColliderShape::Circle { radius }, ColliderShape::Point) => {
            let dist = actual_pos_a.distance(actual_pos_b);
            if dist < *radius {
                let normal = (actual_pos_b - actual_pos_a).normalize();
                Some((radius - dist, normal))
            } else {
                None
            }
        }

        (ColliderShape::Point, ColliderShape::AABB { half_extents }) => {
            let inside_x = (actual_pos_a.x - actual_pos_b.x).abs() < half_extents.x;
            let inside_y = (actual_pos_a.y - actual_pos_b.y).abs() < half_extents.y;
            if inside_x && inside_y {
                // Simple penetration for point in AABB
                Some((0.001, Vec2::UP))
            } else {
                None
            }
        }

        (ColliderShape::AABB { half_extents }, ColliderShape::Point) => {
            let inside_x = (actual_pos_b.x - actual_pos_a.x).abs() < half_extents.x;
            let inside_y = (actual_pos_b.y - actual_pos_a.y).abs() < half_extents.y;
            if inside_x && inside_y {
                Some((0.001, Vec2::DOWN))
            } else {
                None
            }
        }

        (ColliderShape::Point, ColliderShape::Point) => {
            // Points don't collide
            None
        }
    };

    result.map(|(penetration, normal)| {
        let contact = actual_pos_a + normal * (penetration / 2.0);
        (penetration, normal, contact)
    })
}

// =============================================================================
// COLLISION SYSTEM
// =============================================================================

/// Run collision detection on all entities with `Transform` and `Collider` components.
///
/// Returns a list of collision events that occurred this frame.
///
/// # Performance
///
/// This is a simple O(n²) broad phase. For games with many colliders (100+),
/// consider using spatial partitioning (grid, quadtree) with `collision_system_spatial`.
pub fn collision_system(world: &World) -> Vec<CollisionEvent> {
    let mut events = Vec::new();

    // Collect all collidable entities
    let entities: Vec<(Entity, Vec2, Collider)> = world
        .query::<(&Transform, &Collider)>()
        .iter()
        .map(|(entity, (transform, collider))| (entity, transform.position, *collider))
        .collect();

    // Check all pairs
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (entity_a, pos_a, collider_a) = &entities[i];
            let (entity_b, pos_b, collider_b) = &entities[j];

            if let Some((penetration, normal, contact)) =
                check_collision(collider_a, *pos_a, collider_b, *pos_b)
            {
                events.push(CollisionEvent {
                    entity_a: *entity_a,
                    entity_b: *entity_b,
                    penetration,
                    normal,
                    contact,
                });
            }
        }
    }

    events
}

/// Check if a point is inside any collider.
///
/// Useful for mouse picking and point queries.
pub fn point_query(world: &World, point: Vec2) -> Vec<Entity> {
    let mut hits = Vec::new();
    let point_collider = Collider::point();

    for (entity, (transform, collider)) in world.query::<(&Transform, &Collider)>().iter() {
        if check_collision(&point_collider, point, collider, transform.position).is_some() {
            hits.push(entity);
        }
    }

    hits
}

/// Cast a ray and find the first entity hit.
///
/// Returns the entity, hit point, and normal.
pub fn raycast(
    world: &World,
    origin: Vec2,
    direction: Vec2,
    max_distance: f32,
    layer_mask: CollisionLayer,
) -> Option<(Entity, Vec2, Vec2)> {
    let dir = direction.normalize();
    let mut closest: Option<(Entity, f32, Vec2, Vec2)> = None;

    for (entity, (transform, collider)) in world.query::<(&Transform, &Collider)>().iter() {
        if !layer_mask.overlaps(collider.layer) || !collider.enabled {
            continue;
        }

        // Simple circle raycast for now
        if let ColliderShape::Circle { radius } = collider.shape {
            let pos = transform.position + collider.offset;
            let to_center = pos - origin;
            let t = to_center.dot(dir);
            
            if t < 0.0 || t > max_distance {
                continue;
            }

            let closest_point = origin + dir * t;
            let dist_to_center = closest_point.distance(pos);

            if dist_to_center <= radius {
                // Hit! Calculate actual intersection point
                let half_chord = (radius * radius - dist_to_center * dist_to_center).sqrt();
                let hit_t = t - half_chord;

                if hit_t > 0.0 && hit_t < max_distance {
                    let hit_point = origin + dir * hit_t;
                    let normal = (hit_point - pos).normalize();

                    if closest.is_none() || hit_t < closest.as_ref().unwrap().1 {
                        closest = Some((entity, hit_t, hit_point, normal));
                    }
                }
            }
        }
    }

    closest.map(|(e, _, p, n)| (e, p, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_layer_operations() {
        let layer_a = CollisionLayer::new(0);
        let layer_b = CollisionLayer::new(1);
        let combined = layer_a | layer_b;

        assert!(combined.contains(layer_a));
        assert!(combined.contains(layer_b));
        assert!(layer_a.overlaps(combined));
        assert!(!layer_a.overlaps(layer_b));
    }

    #[test]
    fn test_circle_collision() {
        let a = Collider::circle(10.0);
        let b = Collider::circle(10.0);

        // Overlapping
        let result = check_collision(&a, Vec2::ZERO, &b, Vec2::new(15.0, 0.0));
        assert!(result.is_some());

        // Not overlapping
        let result = check_collision(&a, Vec2::ZERO, &b, Vec2::new(25.0, 0.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_aabb_collision() {
        let a = Collider::aabb(20.0, 20.0);
        let b = Collider::aabb(20.0, 20.0);

        // Overlapping
        let result = check_collision(&a, Vec2::ZERO, &b, Vec2::new(15.0, 0.0));
        assert!(result.is_some());

        // Not overlapping
        let result = check_collision(&a, Vec2::ZERO, &b, Vec2::new(25.0, 0.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_layer_filtering() {
        let player_layer = CollisionLayer::new(0);
        let enemy_layer = CollisionLayer::new(1);

        let player = Collider::circle(10.0)
            .with_layer(player_layer)
            .collides_with(enemy_layer);

        let enemy = Collider::circle(10.0)
            .with_layer(enemy_layer)
            .collides_with(player_layer);

        let neutral = Collider::circle(10.0)
            .with_layer(CollisionLayer::new(2))
            .collides_with(CollisionLayer::new(2));

        assert!(player.can_collide_with(&enemy));
        assert!(enemy.can_collide_with(&player));
        assert!(!player.can_collide_with(&neutral));
    }

    #[test]
    fn test_trigger_collider() {
        let trigger = Collider::circle(10.0).as_trigger();
        assert!(trigger.is_trigger);
    }
}
