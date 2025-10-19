pub mod body;
pub mod collision;
pub mod gravity;
pub mod simulation;
pub mod spatial;

pub use body::Body;
pub use collision::{CollisionStrategy, KdTreeCollision, NaiveCollisionStrategy, NoCollisionStrategy};
pub use gravity::{BarnesHutGravityStrategy, GravityStrategy, NaiveGravityStrategy};
pub use simulation::Simulation;