use super::spatial::{Quad, Quadtree};
use super::Body;

pub trait GravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f64; 2], [f64; 2])>,
        quadtree: &Quadtree,
    );

    /// Returns the quadtree cells for visualization. Default returns empty.
    fn get_cells(&self) -> Vec<Quad> {
        Vec::new()
    }
}

pub struct NaiveGravityStrategy;

impl GravityStrategy for NaiveGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f64; 2], [f64; 2])>,
        _quadtree: &Quadtree,
    ) {
        updates.clear();
        updates.reserve(bodies.len());

        for i in 0..bodies.len() {
            let mut total_force = [0.0, 0.0];

            for j in 0..bodies.len() {
                if i == j {
                    continue;
                }

                let body1 = &bodies[i];
                let body2 = &bodies[j];

                let dx = body2.position[0] - body1.position[0];
                let dy = body2.position[1] - body1.position[1];
                let distance_sq = dx * dx + dy * dy;
                let distance = distance_sq.sqrt();

                if distance < 1.0 {
                    continue;
                }

                let force_magnitude =
                    gravity_constant * (body1.mass * body2.mass / distance_sq) as f32;
                let force_x = force_magnitude * (dx / distance) as f32;
                let force_y = force_magnitude * (dy / distance) as f32;

                total_force[0] += force_x;
                total_force[1] += force_y;
            }

            let body = &bodies[i];
            let acceleration = [
                total_force[0] / body.mass as f32,
                total_force[1] / body.mass as f32,
            ];
            let new_velocity = [
                body.velocity[0] + (acceleration[0] * dt) as f64,
                body.velocity[1] + (acceleration[1] * dt) as f64,
            ];
            let new_position = [
                body.position[0] + new_velocity[0] * dt as f64,
                body.position[1] + new_velocity[1] * dt as f64,
            ];

            updates.push((new_position, new_velocity));
        }
    }
}

pub struct BarnesHutGravityStrategy;

impl BarnesHutGravityStrategy {
    pub fn new(_theta: f64, _epsilon: f64) -> Self {
        Self
    }
}

impl GravityStrategy for BarnesHutGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f64; 2], [f64; 2])>,
        quadtree: &Quadtree,
    ) {
        use rayon::prelude::*;

        *updates = bodies
            .par_iter()
            .map(|body| {
                let acc = quadtree.acc(body.position, gravity_constant as f64);

                let new_velocity = [
                    body.velocity[0] + acc[0] * dt as f64,
                    body.velocity[1] + acc[1] * dt as f64,
                ];

                let new_position = [
                    body.position[0] + new_velocity[0] * dt as f64,
                    body.position[1] + new_velocity[1] * dt as f64,
                ];

                (new_position, new_velocity)
            })
            .collect();
    }

    fn get_cells(&self) -> Vec<Quad> {
        // This is tricky now because we don't own the quadtree.
        // We'll handle visualization separately in Simulation.
        Vec::new()
    }
}
