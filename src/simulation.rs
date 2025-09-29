use std::collections::HashMap;

use rand::Rng;

/// A body in the n-body simulation
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: u32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
    pub color: [u8; 4],
    pub radius: f32,
}

impl Body {
    pub fn new(
        id: u32,
        position: [f32; 2],
        velocity: [f32; 2],
        mass: f32,
        color: [u8; 4],
        radius: f32,
    ) -> Self {
        Self {
            id,
            position,
            velocity,
            mass,
            color,
            radius,
        }
    }
}

/// N-body gravitational simulation
pub struct Simulation {
    bodies: HashMap<u32, Body>,
    next_id: u32,
    gravity_constant: f32,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    /// Create a new n-body simulation
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            next_id: 0,
            gravity_constant: 100.0,
        }
    }

    pub fn generate_bodies(&mut self) {
        let mut rng = rand::rng();

        for _ in 0..100 {
            let pos = [
                rng.random_range(-1.0..1.0), // normalized x
                rng.random_range(-1.0..1.0), // normalized y
            ];

            let vel = [
                rng.random_range(-0.01..0.01), // small velocity so things drift
                rng.random_range(-0.01..0.01),
            ];

            let radius = rng.random_range(0.01..0.05); // fraction of screen size

            let color = [
                rng.random_range(0..=255), // red
                rng.random_range(0..=255), // green
                rng.random_range(0..=255), // blue
                255,                       // fully opaque
            ];

            self.add_body(pos, vel, radius, color, radius);
        }
    }
    /// Add a body to the simulation
    pub fn add_body(
        &mut self,
        position: [f32; 2],
        velocity: [f32; 2],
        mass: f32,
        color: [u8; 4],
        radius: f32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let body = Body::new(id, position, velocity, mass, color, radius);
        self.bodies.insert(id, body);
        id
    }

    /// Remove a body from the simulation
    pub fn remove_body(&mut self, id: u32) -> bool {
        self.bodies.remove(&id).is_some()
    }

    /// Get a body by ID
    pub fn get_body(&self, id: u32) -> Option<&Body> {
        self.bodies.get(&id)
    }

    /// Get all bodies
    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.values()
    }

    /// Get number of bodies
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Clear all bodies
    pub fn clear(&mut self) {
        self.bodies.clear();
        self.next_id = 0;
    }

    /// Set gravity constant
    pub fn set_gravity_constant(&mut self, g: f32) {
        self.gravity_constant = g;
    }

    /// Get gravity constant
    pub fn gravity_constant(&self) -> f32 {
        self.gravity_constant
    }

    /// Update simulation by one time step
    pub fn update(&mut self, dt: f32) {
        if self.bodies.len() < 2 {
            return;
        }

        let mut forces: HashMap<u32, [f32; 2]> = HashMap::new();

        // Calculate gravitational forces between all pairs
        let body_ids: Vec<u32> = self.bodies.keys().cloned().collect();

        for &id1 in &body_ids {
            let mut total_force = [0.0, 0.0];

            for &id2 in &body_ids {
                if id1 == id2 {
                    continue;
                }

                let body1 = self.bodies[&id1];
                let body2 = self.bodies[&id2];

                let dx = body2.position[0] - body1.position[0];
                let dy = body2.position[1] - body1.position[1];
                let distance_sq = dx * dx + dy * dy;
                let distance = distance_sq.sqrt();

                // Avoid singularities
                if distance < 1e-6 {
                    continue;
                }

                // F = G * m1 * m2 / r^2
                let force_magnitude = self.gravity_constant * body1.mass * body2.mass / distance_sq;
                let force_x = force_magnitude * (dx / distance);
                let force_y = force_magnitude * (dy / distance);

                total_force[0] += force_x;
                total_force[1] += force_y;
            }

            forces.insert(id1, total_force);
        }

        // Apply forces and update velocities and positions
        for (&id, &force) in &forces {
            if let Some(body) = self.bodies.get_mut(&id) {
                // a = F / m
                let acceleration = [force[0] / body.mass, force[1] / body.mass];

                // Update velocity: v = v + a * dt
                body.velocity[0] += acceleration[0] * dt;
                body.velocity[1] += acceleration[1] * dt;

                // Update position: p = p + v * dt
                body.position[0] += body.velocity[0] * dt;
                body.position[1] += body.velocity[1] * dt;
            }
        }
    }

    /// Find the closest body to a point
    pub fn find_nearest_body(&self, position: [f32; 2]) -> Option<u32> {
        let mut nearest_id = None;
        let mut min_distance = f32::INFINITY;

        for (&id, body) in &self.bodies {
            let dx = body.position[0] - position[0];
            let dy = body.position[1] - position[1];
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < min_distance {
                min_distance = distance;
                nearest_id = Some(id);
            }
        }

        nearest_id
    }
}
