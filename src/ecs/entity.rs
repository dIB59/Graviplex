//! Entity handle - a lightweight reference to an entity in the world.

use std::fmt;

/// A lightweight handle to an entity in the [`World`](super::World).
///
/// Entities are just IDs - they don't store any data themselves.
/// Components are attached to entities to give them behavior and data.
///
/// # Stability
///
/// This type wraps the underlying ECS library's entity type.
/// The API is stable even when the backend changes.
///
/// # Example
///
/// ```ignore
/// let entity = world.spawn((Transform::default(), Sprite::circle(50.0, Color::RED)));
///
/// // Use entity to access components later
/// if let Some(mut transform) = world.get_mut::<Transform>(entity) {
///     transform.position.x += 10.0;
/// }
///
/// // Despawn when done
/// world.despawn(entity);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) hecs::Entity);

impl Entity {
    /// Converts the entity to a unique u64 identifier.
    ///
    /// Useful for serialization or debugging. The bits can be converted
    /// back to an Entity with [`Entity::from_bits`], but only if the
    /// entity still exists in the world.
    #[inline]
    pub fn to_bits(self) -> u64 {
        self.0.to_bits().get()
    }

    /// Reconstructs an entity from bits produced by [`Entity::to_bits`].
    ///
    /// # Safety
    ///
    /// The returned entity may not be valid in the current world.
    /// Always check with [`World::contains`] before using.
    #[inline]
    pub fn from_bits(bits: u64) -> Option<Self> {
        match hecs::Entity::from_bits(bits) {
            Some(e) => Some(Entity(e)),
            None => None,
        }
    }

    /// Returns the underlying hecs entity (for internal use).
    #[inline]
    pub(crate) fn inner(self) -> hecs::Entity {
        self.0
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.to_bits())
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.to_bits())
    }
}

impl From<hecs::Entity> for Entity {
    fn from(entity: hecs::Entity) -> Self {
        Entity(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_from_bits_roundtrip() {
        // Create an entity through the world
        let mut world = hecs::World::new();
        let hecs_entity = world.spawn(());
        let entity = Entity(hecs_entity);

        let bits = entity.to_bits();
        let reconstructed = Entity::from_bits(bits).unwrap();

        assert_eq!(entity, reconstructed);
    }

    #[test]
    fn test_from_bits_zero_returns_none() {
        assert!(Entity::from_bits(0).is_none());
    }

    #[test]
    fn test_debug_display() {
        let mut world = hecs::World::new();
        let entity = Entity(world.spawn(()));

        let debug = format!("{:?}", entity);
        let display = format!("{}", entity);

        assert!(debug.starts_with("Entity("));
        assert!(display.starts_with("Entity("));
    }
}
