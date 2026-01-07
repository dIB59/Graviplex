use super::spatial::Quadtree;
use super::Body;

pub trait GravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f64,
        dt: f64,
        updates: &mut Vec<([f64; 2], [f64; 2])>,
    );
}

pub struct NaiveGravityStrategy;

impl GravityStrategy for NaiveGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f64,
        dt: f64,
        updates: &mut Vec<([f64; 2], [f64; 2])>,
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

                let force_magnitude = gravity_constant * body1.mass * body2.mass / distance_sq;
                let force_x = force_magnitude * (dx / distance);
                let force_y = force_magnitude * (dy / distance);

                total_force[0] += force_x;
                total_force[1] += force_y;
            }

            let body = &bodies[i];
            let acceleration = [total_force[0] / body.mass, total_force[1] / body.mass];
            let new_velocity = [
                body.velocity[0] + acceleration[0] * dt,
                body.velocity[1] + acceleration[1] * dt,
            ];
            let new_position = [
                body.position[0] + new_velocity[0] * dt,
                body.position[1] + new_velocity[1] * dt,
            ];

            updates.push((new_position, new_velocity));
        }
    }
}

pub struct BarnesHutGravityStrategy {
    quadtree: Quadtree,
}

impl BarnesHutGravityStrategy {
    pub fn new(theta: f64, epsilon: f64) -> Self {
        Self {
            quadtree: Quadtree::new(theta, epsilon),
        }
    }
}

impl GravityStrategy for BarnesHutGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f64,
        dt: f64,
        updates: &mut Vec<([f64; 2], [f64; 2])>,
    ) {
        use super::spatial::Quad;
        use rayon::prelude::*;

        let positions: Vec<[f64; 2]> = bodies.par_iter().map(|b| b.position).collect();
        let root_quad = Quad::new_containing(&positions);
        self.quadtree.clear(root_quad);

        let masses: Vec<f64> = bodies.par_iter().map(|b| b.mass).collect();
        self.quadtree.build(&positions, &masses, root_quad);

        *updates = bodies
            .par_iter()
            .map(|body| {
                let acc = self.quadtree.acc(body.position, gravity_constant);

                let new_velocity = [
                    body.velocity[0] + acc[0] * dt,
                    body.velocity[1] + acc[1] * dt,
                ];

                let new_position = [
                    body.position[0] + new_velocity[0] * dt,
                    body.position[1] + new_velocity[1] * dt,
                ];

                (new_position, new_velocity)
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Body;
    use rand::Rng;

    #[test]
    fn test_barnes_hut_vs_naive() {
        let mut bodies = Vec::new();
        let n = 100;
        let mut rng = rand::rng();
        for i in 0..n {
            bodies.push(Body::new(
                i as u32,
                [
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                ],
                [0.0, 0.0],
                rng.random_range(1.0..10.0),
                [0, 0, 0, 255],
                1.0,
            ));
        }

        let dt = 0.01;
        let gravity_constant = 1000.0;

        let mut naive_strategy = NaiveGravityStrategy;
        let mut naive_updates = Vec::new();
        naive_strategy.calculate_forces(&bodies, gravity_constant, dt, &mut naive_updates);

        let mut bh_strategy = BarnesHutGravityStrategy::new(0.01, 0.0); // Very low theta, no softening
        let mut bh_updates = Vec::new();
        bh_strategy.calculate_forces(&bodies, gravity_constant, dt, &mut bh_updates);

        for (i, (naive, bh)) in naive_updates.iter().zip(bh_updates.iter()).enumerate() {
            let pos_diff = [naive.0[0] - bh.0[0], naive.0[1] - bh.0[1]];
            let vel_diff = [naive.1[0] - bh.1[0], naive.1[1] - bh.1[1]];

            let pos_error = (pos_diff[0] * pos_diff[0] + pos_diff[1] * pos_diff[1]).sqrt();
            let vel_error = (vel_diff[0] * vel_diff[0] + vel_diff[1] * vel_diff[1]).sqrt();

            // With f64 and theta=0.01, error should be small
            assert!(
                pos_error < 5.0,
                "Position error too high for body {}: {}",
                i,
                pos_error
            );
            assert!(
                vel_error < 100.0,
                "Velocity error too high for body {}: {}",
                i,
                vel_error
            );
        }
    }
}
