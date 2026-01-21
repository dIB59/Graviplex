//! Graviplex - N-Body Simulation
//!
//! A GPU-accelerated Barnes-Hut n-body simulation built on graviplex-engine.

pub mod nbody;

pub use nbody::{GpuEngine, NBodyGame};
