use super::{SimulationContext, SimulationState, SimulationSystem};
use crate::nbody::spatial::{Quad, Quadtree};

pub trait GravityStrategy: SimulationSystem {
    fn get_cells(&self, _tree: &Quadtree) -> Vec<Quad> {
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
            let mut acc_x = 0.0;
            let mut acc_y = 0.0;
            let (px_i, py_i) = (state.px[i], state.py[i]);

            for j in 0..len {
                if i == j {
                    continue;
                }
                let (px_j, py_j) = (state.px[j], state.py[j]);
                let mass_j = state.masses[j];

                let dx = px_j - px_i;
                let dy = py_j - py_i;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt();

                if dist < 1.0 {
                    continue;
                }

                let force_scale = g * mass_j / (dist_sq * dist);
                acc_x += dx * force_scale;
                acc_y += dy * force_scale;
            }

            state.ax[i] = acc_x;
            state.ay[i] = acc_y;
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

        let results: Vec<[f64; 2]> = (0..state.len())
            .into_par_iter()
            .map(|i| {
                let pos = [state.px[i], state.py[i]];
                quadtree.acc(pos, g)
            })
            .collect();

        for i in 0..state.len() {
            state.ax[i] = results[i][0];
            state.ay[i] = results[i][1];
        }
    }
}

impl GravityStrategy for BarnesHutGravityStrategy {
    fn get_cells(&self, tree: &Quadtree) -> Vec<Quad> {
        tree.get_cells()
    }
}
