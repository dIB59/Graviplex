pub mod collision;
pub mod gravity;
pub mod integrator;

pub use collision::{
    CollisionStrategy, KdTreeCollision, NaiveCollisionStrategy, NoCollisionStrategy,
    QuadtreeCollision,
};
pub use gravity::{BarnesHutGravityStrategy, GravityStrategy, NaiveGravityStrategy};
pub use integrator::{EulerIntegrator, VerletIntegratorStage1, VerletIntegratorStage2};

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

pub enum GravityStrategyEnum {
    Naive(NaiveGravityStrategy),
    BarnesHut(BarnesHutGravityStrategy),
}

impl GravityStrategyEnum {
    pub fn get_cells(&self, quadtree: &Quadtree) -> Vec<super::spatial::quadtree::Quad> {
        match self {
            Self::Naive(_) => Vec::new(),
            Self::BarnesHut(s) => s.get_cells(quadtree),
        }
    }
}

impl SimulationSystem for GravityStrategyEnum {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        quadtree: &Quadtree,
    ) {
        match self {
            Self::Naive(s) => s.update(state, context, quadtree),
            Self::BarnesHut(s) => s.update(state, context, quadtree),
        }
    }
}

pub enum CollisionStrategyEnum {
    None(NoCollisionStrategy),
    Naive(NaiveCollisionStrategy),
    KdTree(KdTreeCollision),
    Quadtree(QuadtreeCollision),
}

impl SimulationSystem for CollisionStrategyEnum {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        quadtree: &Quadtree,
    ) {
        match self {
            Self::None(s) => s.update(state, context, quadtree),
            Self::Naive(s) => s.update(state, context, quadtree),
            Self::KdTree(s) => s.update(state, context, quadtree),
            Self::Quadtree(s) => s.update(state, context, quadtree),
        }
    }
}
