use crate::simulation::core::{Body, SimulationState};
use crate::simulation::spatial::{Quad, Quadtree};
use crate::simulation::systems::{
    BarnesHutGravityStrategy, CollisionStrategyEnum, GravityStrategyEnum, KdTreeCollision,
    SimulationSystem,
};
use rand::Rng;

const SPACE_SCALE: f64 = 250000.0;

pub struct Simulation {
    pub state: SimulationState,
    pub next_id: u32,
    pub gravity_constant: f32,
    pub gravity_strategy: GravityStrategyEnum,
    pub collision_strategy: CollisionStrategyEnum,
    quadtree: Quadtree,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            state: SimulationState::new(),
            next_id: 0,
            gravity_constant: 100.0,
            gravity_strategy: GravityStrategyEnum::BarnesHut(BarnesHutGravityStrategy::new(
                0.5, 0.01,
            )),
            collision_strategy: CollisionStrategyEnum::KdTree(KdTreeCollision::new()),
            quadtree: Quadtree::new(0.5, 0.01),
        }
    }

    pub fn set_gravity_strategy(&mut self, strategy: GravityStrategyEnum) {
        self.gravity_strategy = strategy;
    }

    pub fn set_collision_strategy(&mut self, strategy: CollisionStrategyEnum) {
        self.collision_strategy = strategy;
    }

    pub fn generate_bodies(&mut self, count: i32) {
        let mut rng = rand::rng();

        // 1. Add central "Black Hole" or "Star"
        let central_mass = count as f32 * 100.0;
        let central_radius = count as f64 / 10.0;
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
            let r = rng.random_range(0.1..1.0f64).sqrt() * SPACE_SCALE * 0.9;
            let angle = rng.random_range(0.0..std::f64::consts::TAU);

            let x = r * angle.cos();
            let y = r * angle.sin();
            let pos = [x, y];

            // Velocity: Tangential to create orbital motion
            // v = sqrt(G * M_central / r)
            let orbital_speed = (self.gravity_constant * central_mass / r as f32).sqrt();

            // Tangential vector is [-sin(angle), cos(angle)]
            let vel = [
                -angle.sin() * orbital_speed as f64,
                angle.cos() * orbital_speed as f64,
            ];

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
            let mass = radius;
            self.add_body(pos, vel, mass, color, radius as f64);
        }
    }

    pub fn add_body(
        &mut self,
        position: [f64; 2],
        velocity: [f64; 2],
        mass: f32,
        color: [u8; 4],
        radius: f64,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let body = Body::new(id, position, velocity, mass as f64, color, radius);
        self.state.push(body);
        id
    }

    pub fn remove_body(&mut self, id: u32) -> bool {
        if let Some(pos) = self.state.ids.iter().position(|&x| x == id) {
            self.state.ids.swap_remove(pos);
            self.state.px.swap_remove(pos);
            self.state.py.swap_remove(pos);
            self.state.vx.swap_remove(pos);
            self.state.vy.swap_remove(pos);
            self.state.ax.swap_remove(pos);
            self.state.ay.swap_remove(pos);
            self.state.masses.swap_remove(pos);
            self.state.colors.swap_remove(pos);
            self.state.radii.swap_remove(pos);
            true
        } else {
            false
        }
    }

    pub fn bodies(&self) -> Vec<Body> {
        self.state.to_bodies()
    }

    pub fn body_count(&self) -> usize {
        self.state.len()
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.next_id = 0;
    }

    pub fn apply_interaction(&mut self, mouse_pos: [f64; 2], radius: f64, strength: f64) {
        let radius_sq = radius * radius;
        use rayon::prelude::*;

        let len = self.state.len();
        (0..len).into_par_iter().for_each(|i| {
            // Safety: We ensure all vectors have the same length in SimulationState
            unsafe {
                let px = self.state.px.as_ptr();
                let py = self.state.py.as_ptr();
                let vx = self.state.vx.as_ptr() as *mut f64;
                let vy = self.state.vy.as_ptr() as *mut f64;

                let dx = mouse_pos[0] - *px.add(i);
                let dy = mouse_pos[1] - *py.add(i);
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < radius_sq && dist_sq > 0.1 {
                    let dist = dist_sq.sqrt();
                    let force = strength * (1.0 - (dist / radius));
                    let ux = dx / dist;
                    let uy = dy / dist;

                    if strength > 0.0 {
                        let damping = 0.95;
                        *vx.add(i) = *vx.add(i) * damping + ux * force;
                        *vy.add(i) = *vy.add(i) * damping + uy * force;
                    } else {
                        *vx.add(i) += ux * force;
                        *vy.add(i) += uy * force;
                    }
                }
            }
        });
    }

    pub fn update(&mut self, dt: f32) {
        if self.state.is_empty() {
            return;
        }

        // 1. Build Quadtree ONCE
        let root_quad = Quad::new_containing(&self.state.px, &self.state.py);
        self.quadtree.build(
            &self.state.px,
            &self.state.py,
            &self.state.masses,
            root_quad,
        );

        // 2. Prepare Context
        let context = crate::simulation::systems::SimulationContext {
            dt,
            gravity_constant: self.gravity_constant,
        };

        // 3. Execution Pipeline (Velocity Verlet)

        // Stage 1: v += a * dt / 2; x += v * dt
        let mut v1 = crate::simulation::systems::VerletIntegratorStage1;
        v1.update(&mut self.state, &context, &self.quadtree);

        // Stage 2: Recalculate forces for a(t+1)
        // Rebuild Quadtree for new positions
        let root_quad = Quad::new_containing(&self.state.px, &self.state.py);
        self.quadtree.build(
            &self.state.px,
            &self.state.py,
            &self.state.masses,
            root_quad,
        );

        self.gravity_strategy
            .update(&mut self.state, &context, &self.quadtree);

        // Stage 3: v += a(t+1) * dt / 2
        let mut v2 = crate::simulation::systems::VerletIntegratorStage2;
        v2.update(&mut self.state, &context, &self.quadtree);

        // Stage 4: Collisions (Resolution)
        self.collision_strategy
            .update(&mut self.state, &context, &self.quadtree);
    }

    /// Get quadtree cells for visualization
    pub fn get_cells(&self) -> Vec<Quad> {
        self.quadtree.get_cells()
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
