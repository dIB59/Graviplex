//! Change detection for efficient incremental updates.
//!
//! This module provides tick-based change tracking for components and resources.
//! Systems can query for entities where a component has changed since they last ran.
//!
//! # Example
//!
//! ```ignore
//! use graviplex::ecs::{Tick, ChangeTick};
//!
//! // Track when something was last changed
//! let mut tick = ChangeTick::new();
//! tick.mark_changed(Tick::new(5));
//!
//! // Check if it changed since a given tick
//! assert!(tick.is_changed_since(Tick::new(3)));
//! assert!(!tick.is_changed_since(Tick::new(10)));
//! ```

use std::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// TICK
// =============================================================================

/// A monotonically increasing counter used for change detection.
///
/// Ticks wrap around when they overflow, but the `is_newer_than` method
/// handles wraparound correctly as long as ticks are compared within
/// a reasonable window (about 2 billion increments).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tick(u32);

impl Tick {
    /// The initial tick value.
    pub const ZERO: Tick = Tick(0);
    
    /// Creates a new tick with the given value.
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    
    /// Returns the raw tick value.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
    
    /// Returns true if this tick is newer than `other`.
    ///
    /// Handles wraparound correctly as long as the difference between
    /// ticks is less than `u32::MAX / 2`.
    #[inline]
    pub fn is_newer_than(self, other: Tick) -> bool {
        // Use wrapping subtraction to handle overflow
        // If the difference is less than half of u32::MAX, self is newer
        self.0.wrapping_sub(other.0) < u32::MAX / 2 && self.0 != other.0
    }
    
    /// Returns the next tick.
    #[inline]
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    
    /// Returns true if this tick is within the window [start, end].
    #[inline]
    pub fn is_in_range(self, start: Tick, end: Tick) -> bool {
        let diff_from_start = self.0.wrapping_sub(start.0);
        let range_size = end.0.wrapping_sub(start.0);
        diff_from_start <= range_size
    }
}

// =============================================================================
// ATOMIC TICK
// =============================================================================

/// Thread-safe tick counter.
#[derive(Debug, Default)]
pub struct AtomicTick(AtomicU32);

impl AtomicTick {
    /// Creates a new atomic tick.
    pub const fn new(value: u32) -> Self {
        Self(AtomicU32::new(value))
    }
    
    /// Loads the current tick value.
    #[inline]
    pub fn load(&self) -> Tick {
        Tick(self.0.load(Ordering::Relaxed))
    }
    
    /// Stores a new tick value.
    #[inline]
    #[allow(dead_code)]
    pub fn store(&self, tick: Tick) {
        self.0.store(tick.0, Ordering::Relaxed);
    }
    
    /// Increments the tick and returns the new value.
    #[inline]
    pub fn increment(&self) -> Tick {
        Tick(self.0.fetch_add(1, Ordering::Relaxed).wrapping_add(1))
    }
}

// =============================================================================
// CHANGE TICK
// =============================================================================

/// Tracks when something was last changed.
///
/// Used for both components and resources to enable efficient
/// change detection queries.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChangeTick {
    /// The tick when this was last marked as changed.
    changed: Tick,
}

impl ChangeTick {
    /// Creates a new change tick at tick 0.
    pub const fn new() -> Self {
        Self {
            changed: Tick::ZERO,
        }
    }
    
    /// Creates a change tick marked as changed at the given tick.
    pub const fn at(tick: Tick) -> Self {
        Self { changed: tick }
    }
    
    /// Marks this as changed at the given tick.
    #[inline]
    pub fn mark_changed(&mut self, tick: Tick) {
        self.changed = tick;
    }
    
    /// Returns the tick when this was last changed.
    #[inline]
    pub fn last_changed(&self) -> Tick {
        self.changed
    }
    
    /// Returns true if this has changed since the given tick.
    ///
    /// Specifically, returns true if `self.changed` is newer than `since`.
    #[inline]
    pub fn is_changed_since(&self, since: Tick) -> bool {
        self.changed.is_newer_than(since)
    }
}

// =============================================================================
// CHANGE TRACKERS
// =============================================================================

/// Tracks the global tick and per-system last-run ticks.
#[derive(Debug, Default)]
pub struct ChangeTrackers {
    /// The current world tick, incremented each frame/stage.
    world_tick: AtomicTick,
}

impl ChangeTrackers {
    /// Creates a new change tracker.
    pub const fn new() -> Self {
        Self {
            world_tick: AtomicTick::new(0),
        }
    }
    
    /// Returns the current world tick.
    pub fn current_tick(&self) -> Tick {
        self.world_tick.load()
    }
    
    /// Increments and returns the new world tick.
    ///
    /// Call this at the start of each frame or stage.
    pub fn advance_tick(&self) -> Tick {
        self.world_tick.increment()
    }
}

// =============================================================================
// CHANGED WRAPPER (for future query integration)
// =============================================================================

/// Marker type for filtering queries by changed components.
///
/// When used in a query, only yields entities where the component
/// has changed since the system last ran.
///
/// # Example (future API)
///
/// ```ignore
/// fn update_moved_sprites(
///     query: Query<(&Transform, &Sprite), Changed<Transform>>,
/// ) {
///     // Only iterates entities where Transform changed
///     for (transform, sprite) in &query {
///         // Update render data
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Changed<T>(std::marker::PhantomData<T>);

impl<T> Default for Changed<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Marker type for filtering queries by newly added components.
///
/// Similar to `Changed`, but only yields entities where the component
/// was added since the system last ran.
#[derive(Debug)]
pub struct Added<T>(std::marker::PhantomData<T>);

impl<T> Default for Added<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_comparison() {
        let t1 = Tick::new(5);
        let t2 = Tick::new(10);
        let t3 = Tick::new(5);
        
        assert!(t2.is_newer_than(t1));
        assert!(!t1.is_newer_than(t2));
        assert!(!t1.is_newer_than(t3)); // Equal ticks are not "newer"
    }

    #[test]
    fn test_tick_wraparound() {
        // Test near the wraparound boundary
        let near_max = Tick::new(u32::MAX - 5);
        let wrapped = Tick::new(5);
        
        // After wrapping, 5 should be newer than MAX-5
        assert!(wrapped.is_newer_than(near_max));
        assert!(!near_max.is_newer_than(wrapped));
    }

    #[test]
    fn test_change_tick() {
        let mut ct = ChangeTick::new();
        
        // Initially unchanged (at tick 0)
        assert!(!ct.is_changed_since(Tick::new(0)));
        
        // Mark changed at tick 5
        ct.mark_changed(Tick::new(5));
        
        // Should be changed since tick 3, but not since tick 7
        assert!(ct.is_changed_since(Tick::new(3)));
        assert!(!ct.is_changed_since(Tick::new(7)));
    }

    #[test]
    fn test_change_trackers() {
        let trackers = ChangeTrackers::new();
        
        assert_eq!(trackers.current_tick(), Tick::new(0));
        
        let t1 = trackers.advance_tick();
        assert_eq!(t1, Tick::new(1));
        
        let t2 = trackers.advance_tick();
        assert_eq!(t2, Tick::new(2));
    }

    #[test]
    fn test_tick_range() {
        let start = Tick::new(5);
        let end = Tick::new(10);
        
        assert!(Tick::new(5).is_in_range(start, end));
        assert!(Tick::new(7).is_in_range(start, end));
        assert!(Tick::new(10).is_in_range(start, end));
        assert!(!Tick::new(4).is_in_range(start, end));
        assert!(!Tick::new(11).is_in_range(start, end));
    }
}
