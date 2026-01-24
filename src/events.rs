//! Event system for decoupled communication between game systems.
//!
//! The event system allows different parts of your game to communicate
//! without tight coupling. Systems can emit events and other systems
//! can react to them.
//!
//! # Example
//!
//! ```
//! use graviplex::events::Events;
//!
//! // Define your event types
//! #[derive(Clone, Debug)]
//! struct DamageEvent {
//!     target: u32,  // Entity ID
//!     amount: f32,
//!     source: Option<u32>,
//! }
//!
//! #[derive(Clone, Debug)]
//! struct ScoreEvent(i32);
//!
//! // Create event channels
//! let mut damage_events = Events::<DamageEvent>::new();
//! let mut score_events = Events::<ScoreEvent>::new();
//!
//! // Emit events (in combat system)
//! damage_events.send(DamageEvent {
//!     target: 1,
//!     amount: 25.0,
//!     source: Some(2),
//! });
//!
//! // Read events (in health system)
//! for event in damage_events.read() {
//!     println!("Entity {} took {} damage", event.target, event.amount);
//! }
//!
//! // Clear at end of frame
//! damage_events.clear();
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

// =============================================================================
// EVENTS CHANNEL
// =============================================================================

/// A channel for events of a specific type.
///
/// Events are collected during a frame and can be read by any system.
/// Call `clear()` at the end of each frame to remove processed events.
///
/// # Example
///
/// ```
/// use graviplex::events::Events;
///
/// #[derive(Clone)]
/// struct PlayerDied { player_id: u32 }
///
/// let mut events = Events::<PlayerDied>::new();
///
/// // Send an event
/// events.send(PlayerDied { player_id: 1 });
///
/// // Read all events
/// for event in events.read() {
///     println!("Player {} died!", event.player_id);
/// }
///
/// // Clear for next frame
/// events.clear();
/// ```
#[derive(Debug)]
pub struct Events<T> {
    events: Vec<T>,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Events<T> {
    /// Create a new empty event channel.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Create with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// Send an event.
    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }

    /// Send multiple events.
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = T>) {
        self.events.extend(events);
    }

    /// Read all events (non-consuming iterator).
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    /// Drain all events (consuming iterator).
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.events.drain(..)
    }

    /// Check if there are any events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the number of pending events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

// =============================================================================
// EVENT BUS
// =============================================================================

/// Type-erased event storage for the EventBus.
trait EventStorage: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
    fn len(&self) -> usize;
}

impl<T: 'static> EventStorage for Events<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clear(&mut self) {
        Events::clear(self);
    }

    fn len(&self) -> usize {
        Events::len(self)
    }
}

/// Central event bus for managing all event types.
///
/// The EventBus provides a single place to manage events of different types,
/// useful for integrating with the ECS Resources system.
///
/// # Example
///
/// ```
/// use graviplex::events::EventBus;
///
/// #[derive(Clone, Debug)]
/// struct EnemySpawned { x: f32, y: f32 }
///
/// #[derive(Clone, Debug)]
/// struct ItemCollected { item_id: u32 }
///
/// let mut bus = EventBus::new();
///
/// // Send events
/// bus.send(EnemySpawned { x: 100.0, y: 200.0 });
/// bus.send(ItemCollected { item_id: 42 });
///
/// // Read events of a specific type
/// for event in bus.read::<EnemySpawned>() {
///     println!("Enemy spawned at ({}, {})", event.x, event.y);
/// }
///
/// // Clear all events at end of frame
/// bus.clear_all();
/// ```
#[derive(Default)]
pub struct EventBus {
    channels: HashMap<TypeId, Box<dyn EventStorage>>,
}

impl EventBus {
    /// Create a new empty event bus.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// Get or create a channel for events of type T.
    fn channel_mut<T: 'static>(&mut self) -> &mut Events<T> {
        self.channels
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Events::<T>::new()))
            .as_any_mut()
            .downcast_mut::<Events<T>>()
            .expect("Type mismatch in event bus")
    }

    /// Get a channel for events of type T (read-only).
    fn channel<T: 'static>(&self) -> Option<&Events<T>> {
        self.channels
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.as_any().downcast_ref::<Events<T>>())
    }

    /// Send an event.
    pub fn send<T: 'static>(&mut self, event: T) {
        self.channel_mut::<T>().send(event);
    }

    /// Send multiple events.
    pub fn send_batch<T: 'static>(&mut self, events: impl IntoIterator<Item = T>) {
        self.channel_mut::<T>().send_batch(events);
    }

    /// Read all events of type T.
    ///
    /// Returns an empty iterator if no events of this type exist.
    pub fn read<T: 'static>(&self) -> impl Iterator<Item = &T> {
        self.channel::<T>()
            .map(|events| events.read())
            .into_iter()
            .flatten()
    }

    /// Drain all events of type T.
    pub fn drain<T: 'static>(&mut self) -> impl Iterator<Item = T> + '_ {
        self.channel_mut::<T>().drain()
    }

    /// Check if there are any events of type T.
    pub fn has<T: 'static>(&self) -> bool {
        self.channel::<T>().map(|e| !e.is_empty()).unwrap_or(false)
    }

    /// Get the number of events of type T.
    pub fn count<T: 'static>(&self) -> usize {
        self.channel::<T>().map(|e| e.len()).unwrap_or(0)
    }

    /// Clear events of type T.
    pub fn clear<T: 'static>(&mut self) {
        if let Some(storage) = self.channels.get_mut(&TypeId::of::<T>()) {
            storage.clear();
        }
    }

    /// Clear all events of all types.
    pub fn clear_all(&mut self) {
        for storage in self.channels.values_mut() {
            storage.clear();
        }
    }

    /// Get total number of pending events across all types.
    pub fn total_events(&self) -> usize {
        self.channels.values().map(|s| s.len()).sum()
    }
}

// =============================================================================
// COMMON GAME EVENTS
// =============================================================================

/// Event emitted when an entity is spawned.
#[derive(Clone, Copy, Debug)]
pub struct SpawnEvent {
    /// The spawned entity.
    pub entity: crate::ecs::Entity,
}

/// Event emitted when an entity is about to be despawned.
#[derive(Clone, Copy, Debug)]
pub struct DespawnEvent {
    /// The entity being despawned.
    pub entity: crate::ecs::Entity,
}

/// Event emitted when a collision occurs.
#[derive(Clone, Copy, Debug)]
pub struct CollisionStartEvent {
    /// First entity in the collision.
    pub entity_a: crate::ecs::Entity,
    /// Second entity in the collision.
    pub entity_b: crate::ecs::Entity,
}

/// Event emitted when a collision ends.
#[derive(Clone, Copy, Debug)]
pub struct CollisionEndEvent {
    /// First entity.
    pub entity_a: crate::ecs::Entity,
    /// Second entity.
    pub entity_b: crate::ecs::Entity,
}

/// Event emitted when damage is dealt.
#[derive(Clone, Copy, Debug)]
pub struct DamageEvent {
    /// The entity receiving damage.
    pub target: crate::ecs::Entity,
    /// Amount of damage dealt.
    pub amount: f32,
    /// Source of the damage (if any).
    pub source: Option<crate::ecs::Entity>,
}

/// Event emitted when an entity dies/is destroyed.
#[derive(Clone, Copy, Debug)]
pub struct DeathEvent {
    /// The entity that died.
    pub entity: crate::ecs::Entity,
}

/// Generic trigger event for game logic.
#[derive(Clone, Debug)]
pub struct TriggerEvent {
    /// Unique identifier for the trigger.
    pub trigger_id: String,
    /// Optional data payload.
    pub data: Option<String>,
}

impl TriggerEvent {
    /// Create a new trigger event.
    pub fn new(trigger_id: impl Into<String>) -> Self {
        Self {
            trigger_id: trigger_id.into(),
            data: None,
        }
    }

    /// Create a trigger event with data.
    pub fn with_data(trigger_id: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            trigger_id: trigger_id.into(),
            data: Some(data.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvent {
        value: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct OtherEvent {
        name: String,
    }

    #[test]
    fn test_events_channel() {
        let mut events = Events::<TestEvent>::new();

        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        assert_eq!(events.len(), 2);

        let values: Vec<_> = events.read().map(|e| e.value).collect();
        assert_eq!(values, vec![1, 2]);

        events.clear();
        assert!(events.is_empty());
    }

    #[test]
    fn test_events_drain() {
        let mut events = Events::<TestEvent>::new();

        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        let drained: Vec<_> = events.drain().collect();
        assert_eq!(drained.len(), 2);
        assert!(events.is_empty());
    }

    #[test]
    fn test_event_bus_basic() {
        let mut bus = EventBus::new();

        bus.send(TestEvent { value: 42 });
        bus.send(OtherEvent { name: "test".to_string() });

        assert!(bus.has::<TestEvent>());
        assert!(bus.has::<OtherEvent>());

        let test_events: Vec<_> = bus.read::<TestEvent>().collect();
        assert_eq!(test_events.len(), 1);
        assert_eq!(test_events[0].value, 42);
    }

    #[test]
    fn test_event_bus_clear() {
        let mut bus = EventBus::new();

        bus.send(TestEvent { value: 1 });
        bus.send(OtherEvent { name: "a".to_string() });

        bus.clear::<TestEvent>();
        assert!(!bus.has::<TestEvent>());
        assert!(bus.has::<OtherEvent>());

        bus.clear_all();
        assert!(!bus.has::<OtherEvent>());
    }

    #[test]
    fn test_event_bus_empty_read() {
        let bus = EventBus::new();

        // Reading non-existent event type should return empty iterator
        let count = bus.read::<TestEvent>().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_send_batch() {
        let mut events = Events::<TestEvent>::new();

        events.send_batch(vec![
            TestEvent { value: 1 },
            TestEvent { value: 2 },
            TestEvent { value: 3 },
        ]);

        assert_eq!(events.len(), 3);
    }
}
