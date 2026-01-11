pub mod bridge;
pub mod core;
pub mod simulation;
pub mod spatial;
pub mod systems;

pub use bridge::{QuadCell, SimulationBridge, SimulationCommand};
pub use core::{Body, SimulationState};
pub use simulation::Simulation;
pub use spatial::{KdNode, KdTree, Node, Quad, Quadtree};
pub use systems::{
    BarnesHutGravityStrategy, CollisionStrategy, GravityStrategy, KdTreeCollision,
    NaiveCollisionStrategy, NaiveGravityStrategy, NoCollisionStrategy, QuadtreeCollision,
};
