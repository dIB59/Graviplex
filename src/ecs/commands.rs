//! Commands for deferred entity and component operations.
//!
//! Commands allow systems to queue structural changes (spawn, despawn, insert, remove)
//! that are applied after all systems in the current stage complete.
//!
//! This is necessary because structural changes cannot happen while iterating
//! over entities in a query.
//!
//! # Example
//!
//! ```ignore
//! use graviplex::ecs::{Commands, Transform, Sprite, Visible};
//!
//! fn spawn_enemies(commands: &mut Commands) {
//!     for i in 0..10 {
//!         commands.spawn((
//!             Transform::from_position(Vec2::new(i as f32 * 50.0, 0.0)),
//!             Sprite::circle(20.0, Color::RED),
//!             Visible,
//!         ));
//!     }
//! }
//!
//! // Commands are automatically applied at the end of the stage
//! ```

use std::any::Any;
use std::collections::VecDeque;

use hecs::DynamicBundle;

use super::Entity;
use super::World;

// =============================================================================
// COMMAND TRAIT
// =============================================================================

/// A deferred operation on the World.
trait Command: Send + Sync + 'static {
    /// Applies the command to the world.
    fn apply(self: Box<Self>, world: &mut World);
}

// =============================================================================
// SPAWN COMMAND
// =============================================================================

struct SpawnCommand<B: DynamicBundle + Send + Sync + 'static> {
    bundle: B,
}

impl<B: DynamicBundle + Send + Sync + 'static> Command for SpawnCommand<B> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.spawn(self.bundle);
    }
}

// =============================================================================
// DESPAWN COMMAND
// =============================================================================

struct DespawnCommand {
    entity: Entity,
}

impl Command for DespawnCommand {
    fn apply(self: Box<Self>, world: &mut World) {
        let _ = world.despawn(self.entity);
    }
}

// =============================================================================
// INSERT COMPONENT COMMAND
// =============================================================================

struct InsertCommand<C: hecs::Component + Send + Sync> {
    entity: Entity,
    component: C,
}

impl<C: hecs::Component + Send + Sync> Command for InsertCommand<C> {
    fn apply(self: Box<Self>, world: &mut World) {
        let _ = world.insert(self.entity, self.component);
    }
}

// =============================================================================
// REMOVE COMPONENT COMMAND
// =============================================================================

struct RemoveCommand<C: hecs::Component> {
    entity: Entity,
    _marker: std::marker::PhantomData<C>,
}

impl<C: hecs::Component> Command for RemoveCommand<C> {
    fn apply(self: Box<Self>, world: &mut World) {
        let _ = world.remove::<C>(self.entity);
    }
}

// =============================================================================
// INSERT RESOURCE COMMAND
// =============================================================================

struct InsertResourceCommand<R: Any + Send + Sync + Clone + 'static> {
    resource: R,
}

impl<R: Any + Send + Sync + Clone + 'static> Command for InsertResourceCommand<R> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.insert_resource(self.resource);
    }
}

// =============================================================================
// CUSTOM COMMAND
// =============================================================================

struct CustomCommand<F: FnOnce(&mut World) + Send + Sync + 'static> {
    func: F,
}

impl<F: FnOnce(&mut World) + Send + Sync + 'static> Command for CustomCommand<F> {
    fn apply(self: Box<Self>, world: &mut World) {
        (self.func)(world);
    }
}

// =============================================================================
// COMMAND QUEUE
// =============================================================================

/// A queue of deferred commands to be applied to the World.
///
/// Commands are applied in the order they were queued.
pub struct CommandQueue {
    commands: VecDeque<Box<dyn Command>>,
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandQueue {
    /// Creates a new empty command queue.
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }
    
    /// Adds a command to the queue.
    fn push<C: Command>(&mut self, command: C) {
        self.commands.push_back(Box::new(command));
    }
    
    /// Applies all queued commands to the world.
    ///
    /// The queue is cleared after applying.
    pub fn apply(&mut self, world: &mut World) {
        while let Some(command) = self.commands.pop_front() {
            command.apply(world);
        }
    }
    
    /// Returns the number of pending commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }
    
    /// Returns true if there are no pending commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
    
    /// Clears all pending commands without applying them.
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

// =============================================================================
// COMMANDS - User-facing API
// =============================================================================

/// Interface for queuing deferred World operations.
///
/// Commands are collected during system execution and applied afterward,
/// which allows structural changes while iterating.
///
/// # Example
///
/// ```ignore
/// fn cleanup_dead_entities(commands: &mut Commands, world: &World) {
///     for (entity, health) in world.query::<&Health>() {
///         if health.current <= 0.0 {
///             commands.despawn(entity);
///         }
///     }
/// }
/// ```
pub struct Commands<'q> {
    queue: &'q mut CommandQueue,
}

impl<'q> Commands<'q> {
    /// Creates a new Commands wrapper around a queue.
    pub fn new(queue: &'q mut CommandQueue) -> Self {
        Self { queue }
    }
    
    /// Spawns a new entity with the given components.
    ///
    /// Returns an `EntityCommands` for adding more components.
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.spawn((Transform::default(), Sprite::circle(50.0, Color::RED)));
    /// ```
    pub fn spawn<B: DynamicBundle + Send + Sync + 'static>(&mut self, bundle: B) -> &mut Self {
        self.queue.push(SpawnCommand { bundle });
        self
    }
    
    /// Despawns an entity and all its components.
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.despawn(entity);
    /// ```
    pub fn despawn(&mut self, entity: Entity) -> &mut Self {
        self.queue.push(DespawnCommand { entity });
        self
    }
    
    /// Inserts a component on an existing entity.
    ///
    /// If the entity already has this component type, it will be replaced.
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.insert(entity, Velocity(Vec2::new(100.0, 0.0)));
    /// ```
    pub fn insert<C: hecs::Component + Send + Sync>(&mut self, entity: Entity, component: C) -> &mut Self {
        self.queue.push(InsertCommand { entity, component });
        self
    }
    
    /// Removes a component from an entity.
    ///
    /// Does nothing if the entity doesn't have this component.
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.remove::<Velocity>(entity);
    /// ```
    pub fn remove<C: hecs::Component>(&mut self, entity: Entity) -> &mut Self {
        self.queue.push(RemoveCommand::<C> {
            entity,
            _marker: std::marker::PhantomData,
        });
        self
    }
    
    /// Inserts a resource into the World.
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.insert_resource(GameState::Playing);
    /// ```
    pub fn insert_resource<R: Any + Send + Sync + Clone + 'static>(&mut self, resource: R) -> &mut Self {
        self.queue.push(InsertResourceCommand { resource });
        self
    }
    
    /// Queues a custom command.
    ///
    /// Use this for complex operations that don't fit other methods.
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.add(|world: &mut World| {
    ///     // Complex world manipulation
    /// });
    /// ```
    pub fn add<F: FnOnce(&mut World) + Send + Sync + 'static>(&mut self, func: F) -> &mut Self {
        self.queue.push(CustomCommand { func });
        self
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::ecs::Transform;

    #[test]
    fn test_spawn_command() {
        let mut world = World::new();
        let mut queue = CommandQueue::new();
        
        {
            let mut commands = Commands::new(&mut queue);
            commands.spawn((Transform::from_position(Vec2::ZERO),));
        }
        
        assert_eq!(world.len(), 0);
        queue.apply(&mut world);
        assert_eq!(world.len(), 1);
    }

    #[test]
    fn test_despawn_command() {
        let mut world = World::new();
        let entity = world.spawn((Transform::from_position(Vec2::ZERO),));
        
        let mut queue = CommandQueue::new();
        {
            let mut commands = Commands::new(&mut queue);
            commands.despawn(entity);
        }
        
        assert_eq!(world.len(), 1);
        queue.apply(&mut world);
        assert_eq!(world.len(), 0);
    }

    #[test]
    fn test_insert_component_command() {
        let mut world = World::new();
        let entity = world.spawn((Transform::from_position(Vec2::ZERO),));
        
        // Verify entity doesn't have the component yet
        assert!(!world.has::<crate::ecs::Visible>(entity));
        
        let mut queue = CommandQueue::new();
        {
            let mut commands = Commands::new(&mut queue);
            commands.insert(entity, crate::ecs::Visible);
        }
        
        queue.apply(&mut world);
        assert!(world.has::<crate::ecs::Visible>(entity));
    }

    #[test]
    fn test_custom_command() {
        #[derive(Clone)]
        struct Counter(u32);
        
        let mut world = World::new();
        world.insert_resource(Counter(0));
        
        let mut queue = CommandQueue::new();
        {
            let mut commands = Commands::new(&mut queue);
            commands.add(|world: &mut World| {
                world.resource_mut::<Counter>().0 += 42;
            });
        }
        
        queue.apply(&mut world);
        assert_eq!(world.resource::<Counter>().0, 42);
    }

    #[test]
    fn test_command_ordering() {
        #[derive(Clone, Default)]
        struct Log(Vec<&'static str>);
        
        let mut world = World::new();
        world.insert_resource(Log::default());
        
        let mut queue = CommandQueue::new();
        {
            let mut commands = Commands::new(&mut queue);
            commands.add(|w: &mut World| w.resource_mut::<Log>().0.push("first"));
            commands.add(|w: &mut World| w.resource_mut::<Log>().0.push("second"));
            commands.add(|w: &mut World| w.resource_mut::<Log>().0.push("third"));
        }
        
        queue.apply(&mut world);
        assert_eq!(world.resource::<Log>().0, vec!["first", "second", "third"]);
    }
}
