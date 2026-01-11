use super::{SimulationContext, SimulationState, SimulationSystem};
use crate::simulation::spatial::quadtree::Quadtree;

pub trait Integrator: SimulationSystem {}

/// Stage 1 of Velocity Verlet:
/// v += a * dt / 2
/// x += v * dt
pub struct VerletIntegratorStage1;

impl SimulationSystem for VerletIntegratorStage1 {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
        use rayon::prelude::*;
        let dt = context.dt as f64;

        state
            .positions
            .par_iter_mut()
            .zip(state.velocities.par_iter_mut())
            .zip(state.accelerations.par_iter())
            .for_each(|((pos, vel), acc)| {
                vel[0] += acc[0] * dt * 0.5;
                vel[1] += acc[1] * dt * 0.5;
                pos[0] += vel[0] * dt;
                pos[1] += vel[1] * dt;
            });
    }
}

/// Stage 2 of Velocity Verlet:
/// v += a * dt / 2
pub struct VerletIntegratorStage2;

impl SimulationSystem for VerletIntegratorStage2 {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
        use rayon::prelude::*;
        let dt = context.dt as f64;

        state
            .velocities
            .par_iter_mut()
            .zip(state.accelerations.par_iter())
            .for_each(|(vel, acc)| {
                vel[0] += acc[0] * dt * 0.5;
                vel[1] += acc[1] * dt * 0.5;
            });
    }
}

/// Simple Semi-Implicit Euler Integrator:
/// v += a * dt
/// x += v * dt
pub struct EulerIntegrator;

impl SimulationSystem for EulerIntegrator {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
        use rayon::prelude::*;
        let dt = context.dt as f64;

        state
            .positions
            .par_iter_mut()
            .zip(state.velocities.par_iter_mut())
            .zip(state.accelerations.par_iter())
            .for_each(|((pos, vel), acc)| {
                vel[0] += acc[0] * dt;
                vel[1] += acc[1] * dt;
                pos[0] += vel[0] * dt;
                pos[1] += vel[1] * dt;
            });
    }
}
