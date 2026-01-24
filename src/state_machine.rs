//! State machine component for entity behavior.
//!
//! A simple but powerful state machine for managing entity states like
//! idle, walking, jumping, attacking, etc. Perfect for 2D action games.
//!
//! # Example
//!
//! ```ignore
//! use graviplex::state_machine::{StateMachine, StateId};
//!
//! // Define states as constants
//! const IDLE: StateId = StateId::new("idle");
//! const RUN: StateId = StateId::new("run");
//! const JUMP: StateId = StateId::new("jump");
//! const ATTACK: StateId = StateId::new("attack");
//!
//! // Create state machine with initial state
//! let mut sm = StateMachine::new(IDLE);
//!
//! // Add transitions
//! sm.add_transition(IDLE, RUN, |ctx| ctx.input_x.abs() > 0.1);
//! sm.add_transition(RUN, IDLE, |ctx| ctx.input_x.abs() < 0.1);
//! sm.add_transition(IDLE, JUMP, |ctx| ctx.jump_pressed && ctx.on_ground);
//! sm.add_transition(RUN, JUMP, |ctx| ctx.jump_pressed && ctx.on_ground);
//!
//! // In update:
//! let ctx = StateContext { input_x: 1.0, jump_pressed: false, on_ground: true };
//! sm.update(&ctx);
//! ```

use std::collections::HashMap;
use std::hash::Hash;

// =============================================================================
// STATE ID
// =============================================================================

/// Identifier for a state.
///
/// Can be created from a string for debugging or as a numeric ID for performance.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateId(StateIdInner);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum StateIdInner {
    Named(String),
    Numeric(u32),
}

impl StateId {
    /// Create a named state ID (useful for debugging).
    pub fn named(name: impl Into<String>) -> Self {
        Self(StateIdInner::Named(name.into()))
    }

    /// Create a numeric state ID (more efficient).
    pub const fn numeric(id: u32) -> Self {
        Self(StateIdInner::Numeric(id))
    }

    /// Get the name if this is a named state.
    pub fn name(&self) -> Option<&str> {
        match &self.0 {
            StateIdInner::Named(s) => Some(s),
            StateIdInner::Numeric(_) => None,
        }
    }
}

impl From<&str> for StateId {
    fn from(s: &str) -> Self {
        Self::named(s)
    }
}

impl From<String> for StateId {
    fn from(s: String) -> Self {
        Self::named(s)
    }
}

impl From<u32> for StateId {
    fn from(id: u32) -> Self {
        Self::numeric(id)
    }
}

// =============================================================================
// COMMON STATE IDs (PRESETS)
// =============================================================================

/// Common state IDs for typical 2D action game characters.
pub mod states {
    use super::StateId;

    pub fn idle() -> StateId { StateId::named("idle") }
    pub fn walk() -> StateId { StateId::named("walk") }
    pub fn run() -> StateId { StateId::named("run") }
    pub fn jump() -> StateId { StateId::named("jump") }
    pub fn fall() -> StateId { StateId::named("fall") }
    pub fn land() -> StateId { StateId::named("land") }
    pub fn attack() -> StateId { StateId::named("attack") }
    pub fn hurt() -> StateId { StateId::named("hurt") }
    pub fn dead() -> StateId { StateId::named("dead") }
    pub fn crouch() -> StateId { StateId::named("crouch") }
    pub fn dash() -> StateId { StateId::named("dash") }
    pub fn climb() -> StateId { StateId::named("climb") }
    pub fn swim() -> StateId { StateId::named("swim") }
}

// =============================================================================
// STATE INFO
// =============================================================================

/// Information about a state for the state machine.
#[derive(Clone)]
pub struct StateInfo {
    /// Minimum time to stay in this state before transitioning.
    pub min_duration: f32,
    /// Maximum time in state (auto-transition after this).
    pub max_duration: Option<f32>,
    /// State to transition to when max_duration is reached.
    pub timeout_state: Option<StateId>,
    /// Whether this state can be interrupted by any transition.
    pub interruptible: bool,
}

impl Default for StateInfo {
    fn default() -> Self {
        Self {
            min_duration: 0.0,
            max_duration: None,
            timeout_state: None,
            interruptible: true,
        }
    }
}

impl StateInfo {
    /// Create a new state info with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum duration before allowing transitions.
    pub fn with_min_duration(mut self, duration: f32) -> Self {
        self.min_duration = duration;
        self
    }

    /// Set maximum duration with auto-transition to another state.
    pub fn with_timeout(mut self, duration: f32, next_state: impl Into<StateId>) -> Self {
        self.max_duration = Some(duration);
        self.timeout_state = Some(next_state.into());
        self
    }

    /// Mark this state as non-interruptible (must complete min_duration).
    pub fn non_interruptible(mut self) -> Self {
        self.interruptible = false;
        self
    }
}

// =============================================================================
// TRANSITION
// =============================================================================

/// A transition between states.
pub struct Transition<C> {
    /// Source state.
    pub from: StateId,
    /// Target state.
    pub to: StateId,
    /// Condition function that returns true when transition should happen.
    pub condition: Box<dyn Fn(&C) -> bool + Send + Sync>,
    /// Priority (higher = checked first).
    pub priority: i32,
}

// =============================================================================
// STATE MACHINE COMPONENT
// =============================================================================

/// State machine component for managing entity behavior states.
///
/// Generic over `C`, the context type that provides information for
/// evaluating transitions.
///
/// # Example
///
/// ```ignore
/// #[derive(Default)]
/// struct PlayerContext {
///     input_x: f32,
///     input_y: f32,
///     on_ground: bool,
///     jump_pressed: bool,
///     attack_pressed: bool,
/// }
///
/// let mut sm = StateMachine::<PlayerContext>::new(states::idle());
///
/// // Idle -> Run when moving
/// sm.add_transition(states::idle(), states::run(), |ctx| {
///     ctx.input_x.abs() > 0.1
/// });
///
/// // Run -> Idle when stopped
/// sm.add_transition(states::run(), states::idle(), |ctx| {
///     ctx.input_x.abs() < 0.1
/// });
///
/// // Jump from ground states
/// sm.add_transition_from_any(
///     &[states::idle(), states::run()],
///     states::jump(),
///     |ctx| ctx.jump_pressed && ctx.on_ground
/// );
/// ```
pub struct StateMachine<C> {
    /// Current state.
    current: StateId,
    /// Previous state (for transition callbacks).
    previous: Option<StateId>,
    /// Time spent in current state.
    state_time: f32,
    /// State information (min/max duration, etc).
    state_info: HashMap<StateId, StateInfo>,
    /// Transitions.
    transitions: Vec<Transition<C>>,
    /// Whether a transition just occurred this frame.
    just_changed: bool,
}

impl<C> StateMachine<C> {
    /// Create a new state machine with an initial state.
    pub fn new(initial_state: impl Into<StateId>) -> Self {
        Self {
            current: initial_state.into(),
            previous: None,
            state_time: 0.0,
            state_info: HashMap::new(),
            transitions: Vec::new(),
            just_changed: true, // Consider initial state as "just entered"
        }
    }

    /// Get the current state.
    pub fn current(&self) -> &StateId {
        &self.current
    }

    /// Get the previous state (if any).
    pub fn previous(&self) -> Option<&StateId> {
        self.previous.as_ref()
    }

    /// Get time spent in current state.
    pub fn state_time(&self) -> f32 {
        self.state_time
    }

    /// Check if state just changed this frame.
    pub fn just_changed(&self) -> bool {
        self.just_changed
    }

    /// Check if current state matches the given state.
    pub fn is(&self, state: &StateId) -> bool {
        &self.current == state
    }

    /// Check if current state is any of the given states.
    pub fn is_any(&self, states: &[StateId]) -> bool {
        states.iter().any(|s| &self.current == s)
    }

    /// Add information about a state.
    pub fn add_state_info(&mut self, state: impl Into<StateId>, info: StateInfo) {
        self.state_info.insert(state.into(), info);
    }

    /// Add a transition between two states.
    pub fn add_transition<F>(&mut self, from: impl Into<StateId>, to: impl Into<StateId>, condition: F)
    where
        F: Fn(&C) -> bool + Send + Sync + 'static,
    {
        self.transitions.push(Transition {
            from: from.into(),
            to: to.into(),
            condition: Box::new(condition),
            priority: 0,
        });
    }

    /// Add a transition with priority.
    pub fn add_transition_with_priority<F>(
        &mut self,
        from: impl Into<StateId>,
        to: impl Into<StateId>,
        priority: i32,
        condition: F,
    ) where
        F: Fn(&C) -> bool + Send + Sync + 'static,
    {
        self.transitions.push(Transition {
            from: from.into(),
            to: to.into(),
            condition: Box::new(condition),
            priority,
        });
    }

    /// Add a transition from multiple source states.
    pub fn add_transition_from_any<F>(
        &mut self,
        from_states: &[StateId],
        to: impl Into<StateId>,
        condition: F,
    ) where
        F: Fn(&C) -> bool + Send + Sync + Clone + 'static,
    {
        let to_state = to.into();
        for from in from_states {
            let condition = condition.clone();
            self.transitions.push(Transition {
                from: from.clone(),
                to: to_state.clone(),
                condition: Box::new(condition),
                priority: 0,
            });
        }
    }

    /// Force transition to a state, ignoring conditions.
    pub fn force_transition(&mut self, state: impl Into<StateId>) {
        let new_state = state.into();
        if self.current != new_state {
            self.previous = Some(std::mem::replace(&mut self.current, new_state));
            self.state_time = 0.0;
            self.just_changed = true;
        }
    }

    /// Update the state machine, evaluating transitions.
    ///
    /// Returns the current state after any transitions.
    pub fn update(&mut self, context: &C, dt: f32) -> &StateId {
        self.just_changed = false;
        self.state_time += dt;

        // Check for timeout
        if let Some(info) = self.state_info.get(&self.current) {
            if let Some(max_duration) = info.max_duration {
                if self.state_time >= max_duration {
                    if let Some(timeout_state) = &info.timeout_state {
                        self.previous = Some(std::mem::replace(&mut self.current, timeout_state.clone()));
                        self.state_time = 0.0;
                        self.just_changed = true;
                        return &self.current;
                    }
                }
            }
        }

        // Check if we can transition (respect min_duration and interruptible)
        let can_transition = self
            .state_info
            .get(&self.current)
            .map(|info| {
                info.interruptible || self.state_time >= info.min_duration
            })
            .unwrap_or(true);

        if !can_transition {
            return &self.current;
        }

        // Collect applicable transitions and sort by priority
        let mut applicable: Vec<&Transition<C>> = self
            .transitions
            .iter()
            .filter(|t| t.from == self.current && (t.condition)(context))
            .collect();

        // Sort by priority (highest first)
        applicable.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Take the first (highest priority) transition
        if let Some(transition) = applicable.first() {
            self.previous = Some(std::mem::replace(&mut self.current, transition.to.clone()));
            self.state_time = 0.0;
            self.just_changed = true;
        }

        &self.current
    }

    /// Clear just_changed flag (call at start of frame).
    pub fn begin_frame(&mut self) {
        self.just_changed = false;
    }
}

impl<C> Clone for StateMachine<C> {
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            previous: self.previous.clone(),
            state_time: self.state_time,
            state_info: self.state_info.clone(),
            transitions: Vec::new(), // Transitions can't be cloned (contain closures)
            just_changed: self.just_changed,
        }
    }
}

// =============================================================================
// SIMPLE ENUM-BASED STATE MACHINE
// =============================================================================

/// A simpler state machine for enum-based states.
///
/// Use this when you have a fixed set of states as an enum
/// and want a lightweight state tracking component.
///
/// # Example
///
/// ```
/// use graviplex::state_machine::SimpleState;
///
/// #[derive(Clone, Copy, PartialEq, Default)]
/// enum EnemyState {
///     #[default]
///     Idle,
///     Patrol,
///     Chase,
///     Attack,
/// }
///
/// let mut state = SimpleState::new(EnemyState::Idle);
///
/// // Update state
/// state.set(EnemyState::Patrol);
///
/// // Check state
/// if state.is(EnemyState::Patrol) {
///     // Do patrol behavior
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SimpleState<S: Clone + Copy + PartialEq> {
    current: S,
    previous: S,
    time_in_state: f32,
    just_changed: bool,
}

impl<S: Clone + Copy + PartialEq + Default> Default for SimpleState<S> {
    fn default() -> Self {
        let initial = S::default();
        Self {
            current: initial,
            previous: initial,
            time_in_state: 0.0,
            just_changed: true,
        }
    }
}

impl<S: Clone + Copy + PartialEq> SimpleState<S> {
    /// Create a new simple state.
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            previous: initial,
            time_in_state: 0.0,
            just_changed: true,
        }
    }

    /// Get the current state.
    pub fn current(&self) -> S {
        self.current
    }

    /// Get the previous state.
    pub fn previous(&self) -> S {
        self.previous
    }

    /// Check if current state matches.
    pub fn is(&self, state: S) -> bool {
        self.current == state
    }

    /// Check if state just changed.
    pub fn just_changed(&self) -> bool {
        self.just_changed
    }

    /// Get time in current state.
    pub fn time_in_state(&self) -> f32 {
        self.time_in_state
    }

    /// Set a new state.
    pub fn set(&mut self, state: S) {
        if self.current != state {
            self.previous = self.current;
            self.current = state;
            self.time_in_state = 0.0;
            self.just_changed = true;
        }
    }

    /// Update time tracking (call each frame).
    pub fn update(&mut self, dt: f32) {
        self.just_changed = false;
        self.time_in_state += dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Default, Debug)]
    enum TestState {
        #[default]
        Idle,
        Walk,
        Jump,
    }

    struct TestContext {
        moving: bool,
        jump_pressed: bool,
    }

    #[test]
    fn test_state_id_creation() {
        let named = StateId::named("idle");
        let numeric = StateId::numeric(0);
        let from_str: StateId = "walk".into();

        assert_eq!(named.name(), Some("idle"));
        assert_eq!(numeric.name(), None);
        assert_eq!(from_str.name(), Some("walk"));
    }

    #[test]
    fn test_state_machine_basic() {
        let mut sm = StateMachine::<TestContext>::new(states::idle());

        sm.add_transition(states::idle(), states::walk(), |ctx| ctx.moving);

        let ctx = TestContext { moving: false, jump_pressed: false };
        sm.update(&ctx, 0.016);
        assert!(sm.is(&states::idle()));

        let ctx = TestContext { moving: true, jump_pressed: false };
        sm.update(&ctx, 0.016);
        assert!(sm.is(&states::walk()));
        assert!(sm.just_changed());
    }

    #[test]
    fn test_simple_state() {
        let mut state = SimpleState::new(TestState::Idle);

        assert!(state.is(TestState::Idle));
        assert!(state.just_changed());

        state.update(0.016);
        assert!(!state.just_changed());

        state.set(TestState::Walk);
        assert!(state.is(TestState::Walk));
        assert!(state.just_changed());
        assert_eq!(state.previous(), TestState::Idle);
    }

    #[test]
    fn test_state_info_timeout() {
        let mut sm = StateMachine::<TestContext>::new(states::jump());
        sm.add_state_info(states::jump(), StateInfo::new().with_timeout(0.5, states::idle()));

        let ctx = TestContext { moving: false, jump_pressed: false };

        // Update until timeout
        for _ in 0..50 {
            sm.update(&ctx, 0.016);
        }

        assert!(sm.is(&states::idle()));
    }
}
