//! Schedule and stage system for organizing system execution.
//!
//! This module provides a way to organize systems into stages that run in order.
//! Systems within the same stage run sequentially (parallel execution is a future goal).
//!
//! # Example
//!
//! ```ignore
//! use graviplex::ecs::{Schedule, CoreStage, IntoSystem};
//!
//! let mut schedule = Schedule::new();
//!
//! // Add systems to different stages
//! schedule.add_system(CoreStage::PreUpdate, input_system);
//! schedule.add_system(CoreStage::Update, movement_system);
//! schedule.add_system(CoreStage::Update, physics_system);
//! schedule.add_system(CoreStage::PostUpdate, despawn_system);
//!
//! // Run all stages in order
//! schedule.run(&mut world);
//! ```

use std::collections::HashMap;

use super::system::{BoxedSystem, IntoSystem, SystemSet};
use super::World;

// =============================================================================
// STAGE LABEL
// =============================================================================

/// A label identifying a stage in the schedule.
///
/// Stages run in a fixed order, with all systems in one stage completing
/// before the next stage begins.
#[allow(dead_code)]
pub trait StageLabel: 'static + Send + Sync + Clone + Eq + std::hash::Hash + std::fmt::Debug {}

// Blanket implementation for all qualifying types
impl<T> StageLabel for T where T: 'static + Send + Sync + Clone + Eq + std::hash::Hash + std::fmt::Debug {}

// =============================================================================
// CORE STAGES
// =============================================================================

/// Built-in stage labels for common execution points.
///
/// The stages execute in this order:
/// 1. `First` - Before anything else (e.g., event clearing)
/// 2. `PreUpdate` - Input processing, pre-physics prep
/// 3. `Update` - Main game logic
/// 4. `PostUpdate` - Physics resolution, collision response
/// 5. `Last` - Cleanup, despawning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreStage {
    /// Runs before all other stages.
    /// Use for clearing events, resetting per-frame state.
    First,
    
    /// Runs before Update.
    /// Use for input processing, pre-physics preparation.
    PreUpdate,
    
    /// Main update stage.
    /// Use for game logic, AI, player input handling.
    Update,
    
    /// Runs after Update.
    /// Use for physics, collision response, transform propagation.
    PostUpdate,
    
    /// Runs after all other stages.
    /// Use for cleanup, despawning marked entities.
    Last,
}

impl CoreStage {
    /// Returns all stages in execution order.
    pub fn all() -> &'static [CoreStage] {
        &[
            CoreStage::First,
            CoreStage::PreUpdate,
            CoreStage::Update,
            CoreStage::PostUpdate,
            CoreStage::Last,
        ]
    }
}

// =============================================================================
// STAGE
// =============================================================================

/// A collection of systems that run together.
///
/// All systems in a stage complete before the next stage begins.
pub struct Stage {
    systems: SystemSet,
}

impl Default for Stage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage {
    /// Creates a new empty stage.
    pub fn new() -> Self {
        Self {
            systems: SystemSet::new(),
        }
    }
    
    /// Adds a system to this stage.
    pub fn add_system<M, S: IntoSystem<M>>(&mut self, system: S) -> &mut Self {
        self.systems.add(system);
        self
    }
    
    /// Adds a boxed system to this stage.
    pub fn add_boxed(&mut self, system: BoxedSystem) -> &mut Self {
        self.systems.add_boxed(system);
        self
    }
    
    /// Returns the number of systems in this stage.
    pub fn len(&self) -> usize {
        self.systems.len()
    }
    
    /// Returns true if this stage has no systems.
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
    
    /// Runs all systems in this stage.
    pub fn run(&mut self, world: &mut World) {
        self.systems.run(world);
    }
}

// =============================================================================
// SCHEDULE
// =============================================================================

/// A schedule that organizes systems into stages.
///
/// The schedule runs stages in a predefined order, with all systems
/// in one stage completing before the next stage begins.
///
/// # Example
///
/// ```ignore
/// use graviplex::ecs::{Schedule, CoreStage};
///
/// let mut schedule = Schedule::new();
///
/// schedule
///     .add_system(CoreStage::Update, movement_system)
///     .add_system(CoreStage::Update, physics_system)
///     .add_system(CoreStage::Last, despawn_system);
///
/// // In your game loop:
/// schedule.run(&mut world);
/// ```
pub struct Schedule {
    stages: HashMap<CoreStage, Stage>,
    stage_order: Vec<CoreStage>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Schedule {
    /// Creates a new schedule with the default stage order.
    pub fn new() -> Self {
        let mut stages = HashMap::new();
        for stage in CoreStage::all() {
            stages.insert(*stage, Stage::new());
        }
        
        Self {
            stages,
            stage_order: CoreStage::all().to_vec(),
        }
    }
    
    /// Adds a system to the specified stage.
    ///
    /// # Example
    ///
    /// ```ignore
    /// schedule.add_system(CoreStage::Update, movement_system);
    /// ```
    pub fn add_system<M, S: IntoSystem<M>>(&mut self, stage: CoreStage, system: S) -> &mut Self {
        if let Some(s) = self.stages.get_mut(&stage) {
            s.add_system(system);
        }
        self
    }
    
    /// Adds a boxed system to the specified stage.
    pub fn add_boxed(&mut self, stage: CoreStage, system: BoxedSystem) -> &mut Self {
        if let Some(s) = self.stages.get_mut(&stage) {
            s.add_boxed(system);
        }
        self
    }
    
    /// Returns a reference to the stage, if it exists.
    pub fn get_stage(&self, stage: CoreStage) -> Option<&Stage> {
        self.stages.get(&stage)
    }
    
    /// Returns a mutable reference to the stage, if it exists.
    pub fn get_stage_mut(&mut self, stage: CoreStage) -> Option<&mut Stage> {
        self.stages.get_mut(&stage)
    }
    
    /// Runs all stages in order.
    ///
    /// Each stage completes fully before the next stage begins.
    pub fn run(&mut self, world: &mut World) {
        for stage_label in &self.stage_order {
            if let Some(stage) = self.stages.get_mut(stage_label) {
                stage.run(world);
            }
        }
    }
    
    /// Returns the total number of systems across all stages.
    pub fn system_count(&self) -> usize {
        self.stages.values().map(|s| s.len()).sum()
    }
}

// =============================================================================
// STARTUP SCHEDULE
// =============================================================================

/// A schedule that runs once at startup.
///
/// Startup systems are useful for initialization, spawning initial entities,
/// loading assets, etc.
///
/// # Example
///
/// ```ignore
/// let mut startup = StartupSchedule::new();
/// startup.add_system(spawn_player);
/// startup.add_system(spawn_enemies);
///
/// // Run once at app start
/// startup.run(&mut world);
/// ```
pub struct StartupSchedule {
    systems: SystemSet,
    has_run: bool,
}

impl Default for StartupSchedule {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupSchedule {
    /// Creates a new empty startup schedule.
    pub fn new() -> Self {
        Self {
            systems: SystemSet::new(),
            has_run: false,
        }
    }
    
    /// Adds a system to the startup schedule.
    pub fn add_system<M, S: IntoSystem<M>>(&mut self, system: S) -> &mut Self {
        self.systems.add(system);
        self
    }
    
    /// Runs all startup systems (only runs once).
    ///
    /// Subsequent calls to `run` will do nothing.
    pub fn run(&mut self, world: &mut World) {
        if !self.has_run {
            self.systems.run(world);
            self.has_run = true;
        }
    }
    
    /// Returns true if the startup schedule has already run.
    pub fn has_run(&self) -> bool {
        self.has_run
    }
    
    /// Resets the schedule so it can run again.
    pub fn reset(&mut self) {
        self.has_run = false;
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_stage_order() {
        #[derive(Clone, Default)]
        struct Order(Vec<&'static str>);
        
        fn first_sys(world: &mut World) {
            world.resource_mut::<Order>().0.push("first");
        }
        
        fn update_sys(world: &mut World) {
            world.resource_mut::<Order>().0.push("update");
        }
        
        fn last_sys(world: &mut World) {
            world.resource_mut::<Order>().0.push("last");
        }
        
        let mut schedule = Schedule::new();
        schedule.add_system(CoreStage::Last, last_sys);
        schedule.add_system(CoreStage::First, first_sys);
        schedule.add_system(CoreStage::Update, update_sys);
        
        let mut world = World::new();
        world.insert_resource(Order::default());
        
        schedule.run(&mut world);
        
        let order = world.resource::<Order>();
        assert_eq!(order.0, vec!["first", "update", "last"]);
    }

    #[test]
    fn test_startup_schedule_runs_once() {
        #[derive(Clone, Default)]
        struct Counter(u32);
        
        fn increment(world: &mut World) {
            world.resource_mut::<Counter>().0 += 1;
        }
        
        let mut startup = StartupSchedule::new();
        startup.add_system(increment);
        
        let mut world = World::new();
        world.insert_resource(Counter::default());
        
        // First run should work
        startup.run(&mut world);
        assert_eq!(world.resource::<Counter>().0, 1);
        
        // Second run should be no-op
        startup.run(&mut world);
        assert_eq!(world.resource::<Counter>().0, 1);
        
        // After reset, should run again
        startup.reset();
        startup.run(&mut world);
        assert_eq!(world.resource::<Counter>().0, 2);
    }
}
