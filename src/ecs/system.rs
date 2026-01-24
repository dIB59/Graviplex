//! System trait and function-to-system conversion.
//!
//! This module provides the infrastructure for defining systems as plain functions
//! that are automatically converted into runnable system objects.
//!
//! # Example
//!
//! ```ignore
//! use graviplex::ecs::{Res, IntoSystem, World};
//! use graviplex::Time;
//!
//! // Define a system as a plain function
//! fn movement_system(time: Res<Time>) {
//!     let dt = time.delta();
//!     println!("Delta: {}", dt);
//! }
//!
//! // Convert to a boxed system
//! let system = movement_system.into_system();
//! ```

use std::any::TypeId;
use std::marker::PhantomData;

use super::param::{fetch_resource, Access};
use super::World;

// =============================================================================
// SYSTEM TRAIT
// =============================================================================

/// Trait for types that can be executed as systems.
///
/// Systems are the primary way to implement game logic. They run each frame
/// and can access the World's entities, components, and resources.
pub trait SystemTrait: Send + Sync + 'static {
    /// Returns the name of this system (for debugging).
    fn name(&self) -> &'static str;
    
    /// Returns the access patterns for this system.
    fn access(&self) -> &[Access];
    
    /// Runs the system with the given world.
    fn run(&mut self, world: &mut World);
}

/// A boxed system that can be stored and executed dynamically.
pub type BoxedSystem = Box<dyn SystemTrait>;

// =============================================================================
// INTO SYSTEM TRAIT
// =============================================================================

/// Trait for types that can be converted into systems.
///
/// This is automatically implemented for functions with supported parameter types.
pub trait IntoSystem<Marker>: Sized {
    /// The system type this converts into.
    type System: SystemTrait;
    
    /// Converts this into a system.
    fn into_system(self) -> Self::System;
}

// =============================================================================
// FUNCTION SYSTEM
// =============================================================================

/// A system created from a function.
pub struct FunctionSystem<F, Marker> {
    func: F,
    name: &'static str,
    access: Vec<Access>,
    _marker: PhantomData<fn() -> Marker>,
}

impl<F, Marker> FunctionSystem<F, Marker> {
    fn new(func: F, name: &'static str, access: Vec<Access>) -> Self {
        Self {
            func,
            name,
            access,
            _marker: PhantomData,
        }
    }
}

// =============================================================================
// MACRO FOR IMPLEMENTING FUNCTION SYSTEMS
// =============================================================================

/// Marker type for systems with no parameters.
pub struct SystemMarker0;

/// Marker type for systems with 1 parameter.
pub struct SystemMarker1<P1>(PhantomData<P1>);

/// Marker type for systems with 2 parameters.
#[allow(dead_code)]
pub struct SystemMarker2<P1, P2>(PhantomData<(P1, P2)>);

/// Marker type for systems with 3 parameters.
#[allow(dead_code)]
pub struct SystemMarker3<P1, P2, P3>(PhantomData<(P1, P2, P3)>);

/// Marker type for systems with 4 parameters.
#[allow(dead_code)]
pub struct SystemMarker4<P1, P2, P3, P4>(PhantomData<(P1, P2, P3, P4)>);

// -----------------------------------------------------------------------------
// 0 Parameters
// -----------------------------------------------------------------------------

impl<F> IntoSystem<SystemMarker0> for F
where
    F: FnMut() + Send + Sync + 'static,
{
    type System = FunctionSystem<F, SystemMarker0>;
    
    fn into_system(self) -> Self::System {
        FunctionSystem::new(self, std::any::type_name::<F>(), vec![])
    }
}

impl<F> SystemTrait for FunctionSystem<F, SystemMarker0>
where
    F: FnMut() + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn access(&self) -> &[Access] {
        &self.access
    }
    
    fn run(&mut self, _world: &mut World) {
        (self.func)();
    }
}

// -----------------------------------------------------------------------------
// 1 Parameter - Res<T>
// -----------------------------------------------------------------------------

impl<F, T> IntoSystem<SystemMarker1<T>> for F
where
    F: FnMut(&T) + Send + Sync + 'static,
    T: 'static,
{
    type System = FunctionSystem<F, SystemMarker1<T>>;
    
    fn into_system(self) -> Self::System {
        FunctionSystem::new(
            self,
            std::any::type_name::<F>(),
            vec![Access::ReadResource(TypeId::of::<T>())],
        )
    }
}

impl<F, T> SystemTrait for FunctionSystem<F, SystemMarker1<T>>
where
    F: FnMut(&T) + Send + Sync + 'static,
    T: 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn access(&self) -> &[Access] {
        &self.access
    }
    
    fn run(&mut self, world: &mut World) {
        let res = fetch_resource::<T>(world);
        (self.func)(res);
    }
}

// -----------------------------------------------------------------------------
// 1 Parameter - ResMut<T> (via &mut T)
// -----------------------------------------------------------------------------
// Note: We intentionally don't implement IntoSystem for fn(&mut T) because
// it conflicts with fn(&mut World). Users should use fn(&mut World) and
// call world.resource_mut::<T>() inside. A future macro could provide
// better ergonomics.
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 1 Parameter - World
// -----------------------------------------------------------------------------

/// Marker for world parameter.
pub struct WorldMarker;

impl<F> IntoSystem<WorldMarker> for F
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    type System = FunctionSystem<F, WorldMarker>;
    
    fn into_system(self) -> Self::System {
        FunctionSystem::new(
            self,
            std::any::type_name::<F>(),
            vec![Access::World],
        )
    }
}

impl<F> SystemTrait for FunctionSystem<F, WorldMarker>
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn access(&self) -> &[Access] {
        &self.access
    }
    
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }
}

// -----------------------------------------------------------------------------
// 2 Parameters - World + Res<T>
// -----------------------------------------------------------------------------

/// Marker for world + resource parameters.
pub struct WorldResMarker<T>(PhantomData<T>);

impl<F, T> IntoSystem<WorldResMarker<T>> for F
where
    F: FnMut(&mut World, &T) + Send + Sync + 'static,
    T: 'static,
{
    type System = FunctionSystem<F, WorldResMarker<T>>;
    
    fn into_system(self) -> Self::System {
        FunctionSystem::new(
            self,
            std::any::type_name::<F>(),
            vec![Access::World, Access::ReadResource(TypeId::of::<T>())],
        )
    }
}

impl<F, T> SystemTrait for FunctionSystem<F, WorldResMarker<T>>
where
    F: FnMut(&mut World, &T) + Send + Sync + 'static,
    T: 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn access(&self) -> &[Access] {
        &self.access
    }
    
    fn run(&mut self, world: &mut World) {
        // We need to get the resource first, then call the function
        // This is safe because we're passing &T (shared reference)
        let res_ptr = world.resource::<T>() as *const T;
        // SAFETY: We're holding a reference to the resource for the duration of the call
        let res = unsafe { &*res_ptr };
        (self.func)(world, res);
    }
}

// =============================================================================
// SYSTEM SET - A collection of systems
// =============================================================================

/// A set of systems that can be run together.
pub struct SystemSet {
    systems: Vec<BoxedSystem>,
}

impl Default for SystemSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSet {
    /// Creates a new empty system set.
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    
    /// Adds a system to the set.
    pub fn add<M, S: IntoSystem<M>>(&mut self, system: S) -> &mut Self {
        self.systems.push(Box::new(system.into_system()));
        self
    }
    
    /// Adds a boxed system to the set.
    pub fn add_boxed(&mut self, system: BoxedSystem) -> &mut Self {
        self.systems.push(system);
        self
    }
    
    /// Returns the number of systems in the set.
    pub fn len(&self) -> usize {
        self.systems.len()
    }
    
    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
    
    /// Runs all systems in order.
    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }
    
    /// Returns an iterator over the systems.
    pub fn iter(&self) -> impl Iterator<Item = &BoxedSystem> {
        self.systems.iter()
    }
    
    /// Returns a mutable iterator over the systems.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BoxedSystem> {
        self.systems.iter_mut()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_system_no_params() {
        static mut CALLED: bool = false;
        
        fn my_system() {
            unsafe { CALLED = true; }
        }
        
        let mut system = my_system.into_system();
        let mut world = World::new();
        
        system.run(&mut world);
        
        unsafe { assert!(CALLED); }
    }

    #[test]
    fn test_function_system_with_world() {
        #[derive(Clone)]
        struct Counter(u32);
        
        fn increment_system(world: &mut World) {
            let counter = world.resource_mut::<Counter>();
            counter.0 += 1;
        }
        
        let mut system = increment_system.into_system();
        let mut world = World::new();
        world.insert_resource(Counter(0));
        
        system.run(&mut world);
        
        assert_eq!(world.resource::<Counter>().0, 1);
    }

    #[test]
    fn test_system_set() {
        #[derive(Clone)]
        struct Counter(u32);
        
        fn inc1(world: &mut World) {
            world.resource_mut::<Counter>().0 += 1;
        }
        
        fn inc2(world: &mut World) {
            world.resource_mut::<Counter>().0 += 2;
        }
        
        let mut set = SystemSet::new();
        set.add(inc1);
        set.add(inc2);
        
        let mut world = World::new();
        world.insert_resource(Counter(0));
        
        set.run(&mut world);
        
        assert_eq!(world.resource::<Counter>().0, 3);
    }
}
