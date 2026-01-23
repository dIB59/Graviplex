//! World - the central container for all entities and components.

use super::Entity;
use hecs::DynamicBundle;
use std::fmt;

/// Central container for all entities and their components.
///
/// The World stores all game entities and their associated components.
/// Use it to spawn entities, attach components, and query for entities
/// with specific component combinations.
///
/// # Stability
///
/// This type wraps the underlying ECS library. The API is stable
/// even when the backend implementation changes.
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
///
/// let mut world = World::new();
///
/// // Spawn an entity with components
/// let player = world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity(Vec2::new(100.0, 0.0)),
///     Sprite::circle(50.0, Color::RED),
///     Visible,
/// ));
///
/// // Query entities with specific components
/// for (entity, (transform, velocity)) in world.query::<(&Transform, &Velocity)>() {
///     println!("{:?} at {:?}", entity, transform.position);
/// }
///
/// // Mutate components
/// for (_, (mut transform, velocity)) in world.query::<(&mut Transform, &Velocity)>() {
///     transform.position += velocity.0 * dt;
/// }
/// ```
pub struct World {
    inner: hecs::World,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Creates a new empty world.
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
        }
    }

    // =========================================================================
    // Entity Management
    // =========================================================================

    /// Spawns a new entity with the given components.
    ///
    /// Returns an [`Entity`] handle that can be used to access the entity later.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Spawn with a tuple of components
    /// let entity = world.spawn((
    ///     Transform::default(),
    ///     Sprite::circle(50.0, Color::RED),
    /// ));
    /// ```
    pub fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        Entity(self.inner.spawn(components))
    }

    /// Returns a builder for spawning an entity with components added incrementally.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let entity = world.spawn_builder()
    ///     .add(Transform::default())
    ///     .add(Sprite::circle(50.0, Color::RED))
    ///     .build();
    /// ```
    pub fn spawn_builder(&mut self) -> EntityBuilder<'_> {
        EntityBuilder {
            world: self,
            builder: hecs::EntityBuilder::new(),
        }
    }

    /// Despawns an entity, removing it and all its components from the world.
    ///
    /// Returns `Ok(())` if the entity existed, or `Err` if it didn't.
    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.inner
            .despawn(entity.inner())
            .map_err(|_| NoSuchEntity(entity))
    }

    /// Returns `true` if the entity exists in this world.
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity.inner())
    }

    /// Returns the number of entities in the world.
    #[inline]
    pub fn len(&self) -> u32 {
        self.inner.len()
    }

    /// Returns `true` if the world contains no entities.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes all entities from the world.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    // =========================================================================
    // Component Access
    // =========================================================================

    /// Gets an immutable reference to a component on an entity.
    ///
    /// Returns `None` if the entity doesn't exist or doesn't have the component.
    pub fn get<T: hecs::Component>(&self, entity: Entity) -> Option<hecs::Ref<'_, T>> {
        self.inner.get::<&T>(entity.inner()).ok()
    }

    /// Gets a mutable reference to a component on an entity.
    ///
    /// Returns `None` if the entity doesn't exist or doesn't have the component.
    pub fn get_mut<T: hecs::Component>(&mut self, entity: Entity) -> Option<hecs::RefMut<'_, T>> {
        self.inner.get::<&mut T>(entity.inner()).ok()
    }

    /// Checks if an entity has a specific component.
    pub fn has<T: hecs::Component>(&self, entity: Entity) -> bool {
        self.inner.satisfies::<&T>(entity.inner()).unwrap_or(false)
    }

    /// Inserts a component on an entity, replacing any existing component of the same type.
    ///
    /// Returns the old component if one existed.
    pub fn insert<T: hecs::Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), NoSuchEntity> {
        self.inner
            .insert_one(entity.inner(), component)
            .map_err(|_| NoSuchEntity(entity))
    }

    /// Removes a component from an entity.
    ///
    /// Returns the removed component if it existed.
    pub fn remove<T: hecs::Component>(&mut self, entity: Entity) -> Option<T> {
        self.inner.remove_one::<T>(entity.inner()).ok()
    }

    // =========================================================================
    // Queries
    // =========================================================================

    /// Queries for entities with specific components.
    ///
    /// Returns an iterator over matching entities and their components.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Immutable query
    /// for (entity, (transform, sprite)) in world.query::<(&Transform, &Sprite)>() {
    ///     println!("{:?} at {:?}", entity, transform.position);
    /// }
    ///
    /// // Mutable query
    /// for (_, (mut transform, velocity)) in world.query::<(&mut Transform, &Velocity)>() {
    ///     transform.position += velocity.0 * dt;
    /// }
    ///
    /// // Single component
    /// for (entity, transform) in world.query::<&Transform>() {
    ///     println!("{:?}", transform.position);
    /// }
    /// ```
    pub fn query<Q: hecs::Query>(&self) -> QueryBorrow<'_, Q> {
        QueryBorrow {
            inner: self.inner.query::<Q>(),
        }
    }

    /// Queries for entities with specific components, with mutable world access.
    ///
    /// Use this when you need to modify the world during iteration
    /// (e.g., spawning or despawning entities).
    pub fn query_mut<Q: hecs::Query>(&mut self) -> QueryMut<'_, Q> {
        QueryMut {
            inner: self.inner.query_mut::<Q>(),
        }
    }

    // =========================================================================
    // Internal Access (for advanced use)
    // =========================================================================

    /// Returns a reference to the underlying hecs World.
    ///
    /// # Warning
    ///
    /// Using this directly ties your code to the hecs library.
    /// Prefer using the wrapped API when possible.
    #[doc(hidden)]
    pub fn inner(&self) -> &hecs::World {
        &self.inner
    }

    /// Returns a mutable reference to the underlying hecs World.
    #[doc(hidden)]
    pub fn inner_mut(&mut self) -> &mut hecs::World {
        &mut self.inner
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("entity_count", &self.len())
            .finish()
    }
}

// =============================================================================
// Entity Builder
// =============================================================================

/// Builder for spawning an entity with components added incrementally.
pub struct EntityBuilder<'w> {
    world: &'w mut World,
    builder: hecs::EntityBuilder,
}

impl<'w> EntityBuilder<'w> {
    /// Adds a component to the entity being built.
    pub fn add<T: hecs::Component>(mut self, component: T) -> Self {
        self.builder.add(component);
        self
    }

    /// Adds a bundle of components to the entity being built.
    pub fn add_bundle(mut self, bundle: impl DynamicBundle) -> Self {
        self.builder.add_bundle(bundle);
        self
    }

    /// Finishes building and spawns the entity.
    pub fn build(mut self) -> Entity {
        Entity(self.world.inner.spawn(self.builder.build()))
    }
}

// =============================================================================
// Query Wrappers
// =============================================================================

/// A borrow of the world for a specific query type.
///
/// Use in a for loop directly, or call `.iter()` for explicit iteration.
pub struct QueryBorrow<'w, Q: hecs::Query + 'w> {
    inner: hecs::QueryBorrow<'w, Q>,
}

impl<'w, Q: hecs::Query + 'w> QueryBorrow<'w, Q> {
    /// Returns an iterator over matching entities and their components.
    pub fn iter(&mut self) -> impl Iterator<Item = (Entity, Q::Item<'_>)> + '_ {
        self.inner.iter().map(|(e, item)| (Entity(e), item))
    }

    /// Returns the number of matching entities (may be slow for large worlds).
    pub fn count(&mut self) -> usize {
        self.inner.iter().count()
    }
}

/// A mutable query borrow of the world.
///
/// Can be iterated directly.
pub struct QueryMut<'w, Q: hecs::Query + 'w> {
    inner: hecs::QueryMut<'w, Q>,
}

impl<'w, Q: hecs::Query + 'w> IntoIterator for QueryMut<'w, Q> {
    type Item = (Entity, Q::Item<'w>);
    type IntoIter = QueryIter<'w, Q>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter {
            inner: self.inner.into_iter(),
        }
    }
}

/// Iterator over query results.
pub struct QueryIter<'w, Q: hecs::Query + 'w> {
    inner: hecs::QueryIter<'w, Q>,
}

impl<'w, Q: hecs::Query + 'w> Iterator for QueryIter<'w, Q> {
    type Item = (Entity, Q::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(e, item)| (Entity(e), item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

// =============================================================================
// Errors
// =============================================================================

/// Error returned when an entity doesn't exist in the world.
#[derive(Debug, Clone, Copy)]
pub struct NoSuchEntity(pub Entity);

impl fmt::Display for NoSuchEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "entity {:?} does not exist", self.0)
    }
}

impl std::error::Error for NoSuchEntity {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    // Marker component (no data)
    struct Player;

    // Component with data
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Health {
        current: f32,
        max: f32,
    }

    #[test]
    fn test_spawn_and_contains() {
        let mut world = World::new();
        assert!(world.is_empty());

        let entity = world.spawn((Position { x: 1.0, y: 2.0 },));
        assert!(world.contains(entity));
        assert_eq!(world.len(), 1);
    }

    #[test]
    fn test_get_component() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 1.0, y: 2.0 },));

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_get_mut_component() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 1.0, y: 2.0 },));

        {
            let mut pos = world.get_mut::<Position>(entity).unwrap();
            pos.x = 100.0;
        }

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 100.0);
    }

    #[test]
    fn test_insert_component() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 1.0, y: 2.0 },));

        assert!(!world.has::<Velocity>(entity));

        world.insert(entity, Velocity { x: 5.0, y: 0.0 }).unwrap();

        assert!(world.has::<Velocity>(entity));
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let entity = world.spawn((
            Position { x: 1.0, y: 2.0 },
            Velocity { x: 5.0, y: 0.0 },
        ));

        let removed = world.remove::<Velocity>(entity);
        assert!(removed.is_some());
        assert!(!world.has::<Velocity>(entity));
        assert!(world.has::<Position>(entity));
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 1.0, y: 2.0 },));

        assert!(world.despawn(entity).is_ok());
        assert!(!world.contains(entity));
        assert!(world.despawn(entity).is_err());
    }

    #[test]
    fn test_query() {
        let mut world = World::new();

        world.spawn((Position { x: 1.0, y: 2.0 }, Velocity { x: 1.0, y: 0.0 }));
        world.spawn((Position { x: 3.0, y: 4.0 }, Velocity { x: 0.0, y: 1.0 }));
        world.spawn((Position { x: 5.0, y: 6.0 },)); // No velocity

        let count = world.query::<(&Position, &Velocity)>().count();
        assert_eq!(count, 2);

        let positions: Vec<_> = world
            .query::<&Position>()
            .iter()
            .map(|(_, p)| (p.x, p.y))
            .collect();
        assert_eq!(positions.len(), 3);
    }

    #[test]
    fn test_entity_builder() {
        let mut world = World::new();

        let entity = world
            .spawn_builder()
            .add(Position { x: 1.0, y: 2.0 })
            .add(Velocity { x: 5.0, y: 0.0 })
            .build();

        assert!(world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));
    }

    #[test]
    fn test_clear() {
        let mut world = World::new();

        world.spawn((Position { x: 1.0, y: 2.0 },));
        world.spawn((Position { x: 3.0, y: 4.0 },));
        assert_eq!(world.len(), 2);

        world.clear();
        assert!(world.is_empty());
    }

    #[test]
    fn test_spawn_with_custom_components() {
        let mut world = World::new();

        // Spawn entity with multiple custom components
        let entity = world.spawn((
            Position { x: 100.0, y: 200.0 },
            Velocity { x: 50.0, y: 0.0 },
            Health { current: 100.0, max: 100.0 },
            Player,
        ));

        assert!(world.contains(entity));
        assert!(world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));
        assert!(world.has::<Health>(entity));
        assert!(world.has::<Player>(entity));

        // Verify component data
        let health = world.get::<Health>(entity).unwrap();
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
    }

    #[test]
    fn test_spawn_marker_component() {
        let mut world = World::new();

        // Marker components have no data but can be queried
        let player_entity = world.spawn((Position { x: 0.0, y: 0.0 }, Player));
        let enemy_entity = world.spawn((Position { x: 10.0, y: 10.0 },));

        assert!(world.has::<Player>(player_entity));
        assert!(!world.has::<Player>(enemy_entity));

        // Query only entities with Player marker
        let player_count = world.query::<(&Position, &Player)>().count();
        assert_eq!(player_count, 1);
    }

    #[test]
    fn test_spawn_builder_with_custom_components() {
        let mut world = World::new();

        let entity = world
            .spawn_builder()
            .add(Position { x: 50.0, y: 75.0 })
            .add(Health { current: 80.0, max: 100.0 })
            .add(Player)
            .build();

        assert!(world.has::<Position>(entity));
        assert!(world.has::<Health>(entity));
        assert!(world.has::<Player>(entity));

        let health = world.get::<Health>(entity).unwrap();
        assert_eq!(health.current, 80.0);
    }

    #[test]
    fn test_mutate_custom_component() {
        let mut world = World::new();

        let entity = world.spawn((Health { current: 100.0, max: 100.0 },));

        // Take damage
        {
            let mut health = world.get_mut::<Health>(entity).unwrap();
            health.current -= 25.0;
        }

        let health = world.get::<Health>(entity).unwrap();
        assert_eq!(health.current, 75.0);
    }

    #[test]
    fn test_query_custom_components() {
        let mut world = World::new();

        // Spawn multiple entities with different component combinations
        world.spawn((Position { x: 0.0, y: 0.0 }, Health { current: 100.0, max: 100.0 }, Player));
        world.spawn((Position { x: 10.0, y: 10.0 }, Health { current: 50.0, max: 50.0 }));
        world.spawn((Position { x: 20.0, y: 20.0 },));

        // Query all entities with Health
        let health_count = world.query::<&Health>().count();
        assert_eq!(health_count, 2);

        // Query only player with health
        let player_health_count = world.query::<(&Health, &Player)>().count();
        assert_eq!(player_health_count, 1);

        // Sum all health values
        let total_health: f32 = world
            .query::<&Health>()
            .iter()
            .map(|(_, h)| h.current)
            .sum();
        assert_eq!(total_health, 150.0);
    }
}
