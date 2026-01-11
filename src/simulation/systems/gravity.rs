use super::{SimulationContext, SimulationState, SimulationSystem};
use crate::simulation::spatial::quadtree::{Quad, Quadtree};

pub trait GravityStrategy: SimulationSystem {
    fn get_cells(&self) -> Vec<Quad> {
        Vec::new()
    }
}

pub struct NaiveGravityStrategy;

impl SimulationSystem for NaiveGravityStrategy {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
        let g = context.gravity_constant as f64;
        let len = state.len();

        for i in 0..len {
            let mut acc = [0.0, 0.0];
            let pos_i = state.positions[i];

            for j in 0..len {
                if i == j {
                    continue;
                }
                let pos_j = state.positions[j];
                let mass_j = state.masses[j];

                let dx = pos_j[0] - pos_i[0];
                let dy = pos_j[1] - pos_i[1];
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt();

                if dist < 1.0 {
                    continue;
                }

                let force_scale = g * mass_j / (dist_sq * dist);
                acc[0] += dx * force_scale;
                acc[1] += dy * force_scale;
            }

            state.accelerations[i] = acc;
        }
    }
}

impl GravityStrategy for NaiveGravityStrategy {}

pub struct BarnesHutGravityStrategy;

impl BarnesHutGravityStrategy {
    pub fn new(_theta: f64, _epsilon: f64) -> Self {
        Self
    }
}

impl SimulationSystem for BarnesHutGravityStrategy {
    fn update(
        &mut self,
        state: &mut SimulationState,
        context: &SimulationContext,
        quadtree: &Quadtree,
    ) {
        use rayon::prelude::*;
        let g = context.gravity_constant as f64;

        state
            .positions
            .par_iter()
            .zip(state.accelerations.par_iter_mut())
            .for_each(|(pos, acc)| {
                *acc = quadtree.acc(*pos, g);
            });
    }
}

impl GravityStrategy for BarnesHutGravityStrategy {}
