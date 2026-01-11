pub mod collision;
pub mod gravity;

pub use collision::{
    CollisionStrategy, KdTreeCollision, NaiveCollisionStrategy, NoCollisionStrategy,
    QuadtreeCollision,
};
pub use gravity::{BarnesHutGravityStrategy, GravityStrategy, NaiveGravityStrategy};

use super::core::state::SimulationState;
use super::spatial::quadtree::Quadtree;

pub struct SimulationContext {
    pub dt: f32,
    pub gravity_constant: f32,
}

pub trait SimulationSystem {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        quadtree: &Quadtree,
    );
}
