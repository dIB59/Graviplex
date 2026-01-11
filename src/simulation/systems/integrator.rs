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

        let len = state.len();
        (0..len).into_par_iter().for_each(|i| {
            // Safety: We ensure all vectors have the same length in SimulationState
            unsafe {
                let px = state.px.as_ptr() as *mut f64;
                let py = state.py.as_ptr() as *mut f64;
                let vx = state.vx.as_ptr() as *mut f64;
                let vy = state.vy.as_ptr() as *mut f64;
                let ax = state.ax.as_ptr();
                let ay = state.ay.as_ptr();

                *vx.add(i) += *ax.add(i) * dt * 0.5;
                *vy.add(i) += *ay.add(i) * dt * 0.5;
                *px.add(i) += *vx.add(i) * dt;
                *py.add(i) += *vy.add(i) * dt;
            }
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

        let len = state.len();
        (0..len).into_par_iter().for_each(|i| unsafe {
            let vx = state.vx.as_ptr() as *mut f64;
            let vy = state.vy.as_ptr() as *mut f64;
            let ax = state.ax.as_ptr();
            let ay = state.ay.as_ptr();

            *vx.add(i) += *ax.add(i) * dt * 0.5;
            *vy.add(i) += *ay.add(i) * dt * 0.5;
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

        let len = state.len();
        (0..len).into_par_iter().for_each(|i| {
            // Safety: We ensure all vectors have the same length in SimulationState
            unsafe {
                let px = state.px.as_ptr() as *mut f64;
                let py = state.py.as_ptr() as *mut f64;
                let vx = state.vx.as_ptr() as *mut f64;
                let vy = state.vy.as_ptr() as *mut f64;
                let ax = state.ax.as_ptr();
                let ay = state.ay.as_ptr();

                *vx.add(i) += *ax.add(i) * dt;
                *vy.add(i) += *ay.add(i) * dt;
                *px.add(i) += *vx.add(i) * dt;
                *py.add(i) += *vy.add(i) * dt;
            }
        });
    }
}
