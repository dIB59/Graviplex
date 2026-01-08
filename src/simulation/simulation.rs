use super::{BarnesHutGravityStrategy, KdTreeCollision};
use super::{Body, CollisionStrategy, GravityStrategy};
use rand::Rng;

const SPACE_SCALE: f64 = 100000.0;

pub struct Simulation {
    pub bodies: Vec<Body>,
    pub next_id: u32,
    pub gravity_constant: f64,
    gravity_strategy: Box<dyn GravityStrategy>,
    collision_strategy: Box<dyn CollisionStrategy>,
    updates_buffer: Vec<([f64; 2], [f64; 2])>,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            next_id: 0,
            gravity_constant: 100.0,
            gravity_strategy: Box::new(BarnesHutGravityStrategy::new(0.5, 0.01)),
            collision_strategy: Box::new(KdTreeCollision),
            updates_buffer: Vec::new(),
        }
    }

    pub fn set_gravity_strategy(&mut self, strategy: Box<dyn GravityStrategy>) {
        self.gravity_strategy = strategy;
    }

    pub fn set_collision_strategy(&mut self, strategy: Box<dyn CollisionStrategy>) {
        self.collision_strategy = strategy;
    }

    pub fn generate_bodies(&mut self, count: i32) {
        let mut rng = rand::rng();

        // 1. Add central "Black Hole" or "Star"
        let central_mass = count as f64 * 1000.0;
        let central_radius = 1000.0;
        self.add_body(
            [1.0, -1.0],
            [0.0, 0.0],
            central_mass,
            [255, 255, 255, 255], // White
            central_radius,
        );

        // 2. Add orbiting bodies
        for _ in 0..count {
            // Distribution: Uniform in a circle
            let r = rng.random_range(0.1..1.0f64).sqrt() * SPACE_SCALE * 0.7;
            let angle = rng.random_range(0.0..std::f64::consts::TAU);

            let x = r * angle.cos();
            let y = r * angle.sin();
            let pos = [x, y];

            // Velocity: Tangential to create orbital motion
            // v = sqrt(G * M_central / r)
            let orbital_speed = (self.gravity_constant * central_mass / r).sqrt();

            // Tangential vector is [-sin(angle), cos(angle)]
            let vel = [-angle.sin() * orbital_speed, angle.cos() * orbital_speed];

            let radius = rng.random_range(5.0..200.0);
            let t = (radius - 5.0) / 195.0; // 0.0 to 1.0 range
            let percentile = (t * 100.0) as i32;

            let color = match percentile {
                0..=19 => {
                    // Deep Crimson -> Blood Red
                    let f = percentile as f64 / 19.0;
                    [
                        (139.0 + f * 60.0) as u8,
                        (0.0 + f * 20.0) as u8,
                        (0.0 + f * 20.0) as u8,
                        255,
                    ]
                }
                20..=39 => {
                    // Deep Navy -> Royal Blue
                    let f = (percentile - 20) as f64 / 19.0;
                    [
                        (0.0 + f * 25.0) as u8,
                        (0.0 + f * 105.0) as u8,
                        (128.0 + f * 77.0) as u8,
                        255,
                    ]
                }
                40..=59 => {
                    // Forest Green -> Emerald
                    let f = (percentile - 40) as f64 / 19.0;
                    [
                        (0.0 + f * 30.0) as u8,
                        (100.0 + f * 56.0) as u8,
                        (0.0 + f * 48.0) as u8,
                        255,
                    ]
                }
                60..=79 => {
                    // Deep Navy -> Royal Blue (same as 20-39%)
                    let f = (percentile - 60) as f64 / 19.0;
                    [
                        (0.0 + f * 25.0) as u8,
                        (0.0 + f * 105.0) as u8,
                        (128.0 + f * 77.0) as u8,
                        255,
                    ]
                }
                _ => {
                    // Dark Orange -> Burnt Sienna
                    let f = (percentile - 80) as f64 / 19.0;
                    [
                        (204.0 - f * 44.0) as u8,
                        (85.0 - f * 30.0) as u8,
                        (0.0 + f * 19.0) as u8,
                        255,
                    ]
                }
            };
            self.add_body(pos, vel, radius, color, radius);
        }
    }

    pub fn add_body(
        &mut self,
        position: [f64; 2],
        velocity: [f64; 2],
        mass: f64,
        color: [u8; 4],
        radius: f64,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let body = Body::new(id, position, velocity, mass, color, radius);
        self.bodies.push(body);
        id
    }

    pub fn remove_body(&mut self, id: u32) -> bool {
        if let Some(pos) = self.bodies.iter().position(|b| b.id == id) {
            self.bodies.swap_remove(pos);
            true
        } else {
            false
        }
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
        self.next_id = 0;
    }

    pub fn update(&mut self, dt: f64) {
        if self.bodies.len() < 2 {
            return;
        }

        self.gravity_strategy.calculate_forces(
            &self.bodies,
            self.gravity_constant,
            dt,
            &mut self.updates_buffer,
        );

        use rayon::prelude::*;
        self.bodies
            .par_iter_mut()
            .zip(self.updates_buffer.par_iter())
            .for_each(|(body, &(position, velocity))| {
                body.position = position;
                body.velocity = velocity;
            });

        self.collision_strategy.handle_collisions(&mut self.bodies);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disc_generation() {
        let mut sim = Simulation::new();
        let count = 1000;
        sim.generate_bodies(count);

        let bodies = sim.bodies();
        assert_eq!(bodies.len(), (count + 1) as usize);

        // Check for non-zero velocities (orbital motion)
        // Skip the central body which has zero velocity
        for body in bodies.iter().skip(1) {
            let speed_sq =
                body.velocity[0] * body.velocity[0] + body.velocity[1] * body.velocity[1];
            assert!(speed_sq > 0.0, "Body has zero velocity");
        }
    }
}
