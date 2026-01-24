//! System parameter types for ergonomic resource access.
//!
//! This module provides types like `Res<T>` and `ResMut<T>` that allow
//! systems to declare their resource dependencies explicitly.
//!
//! # Example
//!
//! ```ignore
//! use graviplex::ecs::{Res, ResMut};
//! use graviplex::Time;
//!
//! fn my_system(time: Res<Time>, mut counter: ResMut<FrameCounter>) {
//!     counter.count += 1;
//!     println!("Frame {} at {}s", counter.count, time.elapsed());
//! }
//! ```

use std::any::TypeId;
use std::ops::{Deref, DerefMut};

use super::World;

// =============================================================================
// ACCESS TYPES
// =============================================================================

/// Describes how a system parameter accesses data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Access {
    /// Read-only access to a resource.
    ReadResource(TypeId),
    /// Mutable access to a resource.
    WriteResource(TypeId),
    /// Read-only access to a component type.
    ReadComponent(TypeId),
    /// Mutable access to a component type.
    WriteComponent(TypeId),
    /// Access to the entire world.
    World,
}

impl Access {
    /// Returns true if this access conflicts with another.
    pub fn conflicts_with(&self, other: &Access) -> bool {
        match (self, other) {
            // World access conflicts with everything
            (Access::World, _) | (_, Access::World) => true,
            // Write-write conflicts
            (Access::WriteResource(a), Access::WriteResource(b)) if a == b => true,
            (Access::WriteComponent(a), Access::WriteComponent(b)) if a == b => true,
            // Read-write conflicts
            (Access::ReadResource(a), Access::WriteResource(b)) if a == b => true,
            (Access::WriteResource(a), Access::ReadResource(b)) if a == b => true,
            (Access::ReadComponent(a), Access::WriteComponent(b)) if a == b => true,
            (Access::WriteComponent(a), Access::ReadComponent(b)) if a == b => true,
            // No conflict
            _ => false,
        }
    }
}

// =============================================================================
// SYSTEM PARAM TRAIT
// =============================================================================

/// Trait for types that can be used as system parameters.
///
/// System parameters are automatically fetched from the World when a system runs.
/// This allows systems to declare their dependencies in the function signature.
///
/// Built-in implementations:
/// - `Res<T>` - Read-only access to a resource
/// - `ResMut<T>` - Mutable access to a resource
/// - `&World` - Read-only access to the world
/// - `&mut World` - Mutable access to the world
pub trait SystemParam: Sized {
    /// The item type that this parameter produces.
    type Item<'w>;
    
    /// Returns the access patterns for this parameter.
    fn access() -> Vec<Access>;
    
    /// Fetches the parameter from the world.
    ///
    /// # Safety
    /// The caller must ensure that the access patterns returned by `access()`
    /// are respected and no conflicting borrows exist.
    fn fetch(world: &World) -> Self::Item<'_>;
    
    /// Fetches the parameter mutably from the world.
    fn fetch_mut(world: &mut World) -> Self::Item<'_>;
}

// =============================================================================
// RES - READ-ONLY RESOURCE ACCESS
// =============================================================================

/// Provides read-only access to a resource stored in the World.
///
/// # Example
///
/// ```ignore
/// use graviplex::ecs::Res;
/// use graviplex::Time;
///
/// fn print_fps(time: Res<Time>) {
///     println!("FPS: {}", time.fps());
/// }
/// ```
///
/// # Panics
///
/// Panics if the resource doesn't exist in the World. Use `Option<Res<T>>`
/// for optional resources.
pub struct Res<'w, T: 'static> {
    value: &'w T,
}

impl<'w, T: 'static> Res<'w, T> {
    /// Creates a new Res from a reference.
    pub fn new(value: &'w T) -> Self {
        Self { value }
    }
    
    /// Returns the inner reference.
    pub fn into_inner(self) -> &'w T {
        self.value
    }
}

impl<T: 'static> Deref for Res<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: 'static + std::fmt::Debug> std::fmt::Debug for Res<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Res").field(self.value).finish()
    }
}

// =============================================================================
// RES MUT - MUTABLE RESOURCE ACCESS
// =============================================================================

/// Provides mutable access to a resource stored in the World.
///
/// # Example
///
/// ```ignore
/// use graviplex::ecs::ResMut;
///
/// struct Score(u32);
///
/// fn increment_score(mut score: ResMut<Score>) {
///     score.0 += 10;
/// }
/// ```
///
/// # Panics
///
/// Panics if the resource doesn't exist in the World. Use `Option<ResMut<T>>`
/// for optional resources.
pub struct ResMut<'w, T: 'static> {
    value: &'w mut T,
}

impl<'w, T: 'static> ResMut<'w, T> {
    /// Creates a new ResMut from a mutable reference.
    pub fn new(value: &'w mut T) -> Self {
        Self { value }
    }
    
    /// Returns the inner mutable reference.
    pub fn into_inner(self) -> &'w mut T {
        self.value
    }
}

impl<T: 'static> Deref for ResMut<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: 'static> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T: 'static + std::fmt::Debug> std::fmt::Debug for ResMut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ResMut").field(&self.value).finish()
    }
}

// =============================================================================
// LOCAL - SYSTEM-LOCAL STATE
// =============================================================================

/// System-local state that persists between system runs.
///
/// Unlike resources, `Local<T>` is unique to each system instance.
/// Useful for caching data or tracking state within a single system.
///
/// # Example
///
/// ```ignore
/// use graviplex::ecs::Local;
///
/// fn count_calls(mut count: Local<u32>) {
///     *count += 1;
///     println!("System called {} times", *count);
/// }
/// ```
pub struct Local<'s, T: Default + 'static> {
    value: &'s mut T,
}

impl<'s, T: Default + 'static> Local<'s, T> {
    /// Creates a new Local from a mutable reference.
    pub fn new(value: &'s mut T) -> Self {
        Self { value }
    }
}

impl<T: Default + 'static> Deref for Local<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: Default + 'static> DerefMut for Local<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

// =============================================================================
// HELPER FUNCTION
// =============================================================================

/// Helper to fetch a resource from the world, panicking if not found.
pub fn fetch_resource<T: 'static>(world: &World) -> &T {
    world.resource::<T>()
}

/// Helper to fetch a mutable resource from the world, panicking if not found.
#[allow(dead_code)]
pub fn fetch_resource_mut<T: 'static>(world: &mut World) -> &mut T {
    world.resource_mut::<T>()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_conflicts() {
        let read1 = Access::ReadResource(TypeId::of::<i32>());
        let read2 = Access::ReadResource(TypeId::of::<i32>());
        let write1 = Access::WriteResource(TypeId::of::<i32>());
        let write2 = Access::WriteResource(TypeId::of::<i32>());
        let other = Access::ReadResource(TypeId::of::<f32>());
        
        // Read-read doesn't conflict
        assert!(!read1.conflicts_with(&read2));
        
        // Write-write conflicts
        assert!(write1.conflicts_with(&write2));
        
        // Read-write conflicts
        assert!(read1.conflicts_with(&write1));
        assert!(write1.conflicts_with(&read1));
        
        // Different types don't conflict
        assert!(!read1.conflicts_with(&other));
        assert!(!write1.conflicts_with(&other));
        
        // World conflicts with everything
        let world = Access::World;
        assert!(world.conflicts_with(&read1));
        assert!(world.conflicts_with(&write1));
    }
}
