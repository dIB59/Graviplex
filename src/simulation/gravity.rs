use super::Body;
use super::spatial::Quadtree;

pub trait GravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f32; 2], [f32; 2])>,
    );
}

pub struct NaiveGravityStrategy;

impl GravityStrategy for NaiveGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f32; 2], [f32; 2])>,
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

                if distance < 1e-6 {
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
    pub fn new(theta: f32, epsilon: f32) -> Self {
        Self {
            quadtree: Quadtree::new(theta, epsilon),
        }
    }
}

impl GravityStrategy for BarnesHutGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f32; 2], [f32; 2])>,
    ) {
        use super::spatial::Quad;
        
        let positions: Vec<[f32; 2]> = bodies.iter().map(|b| b.position).collect();
        let root_quad = Quad::new_containing(&positions);
        self.quadtree.clear(root_quad);

        for body in bodies {
            self.quadtree.insert(body.position, body.mass);
        }

        self.quadtree.propagate();

        updates.clear();
        updates.reserve(bodies.len());

        for body in bodies {
            let acc = self.quadtree.acc(body.position, gravity_constant);

            let new_velocity = [
                body.velocity[0] + acc[0] * dt,
                body.velocity[1] + acc[1] * dt,
            ];

            let new_position = [
                body.position[0] + new_velocity[0] * dt,
                body.position[1] + new_velocity[1] * dt,
            ];

            updates.push((new_position, new_velocity));
        }
    }
}
