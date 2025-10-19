use rand::Rng;
use super::{Body, CollisionStrategy, GravityStrategy};
use super::{BarnesHutGravityStrategy, KdTreeCollision};

const SPACE_SCALE: f32 = 10000.0;

pub struct Simulation {
    bodies: Vec<Body>,
    next_id: u32,
    gravity_constant: f32,
    gravity_strategy: Box<dyn GravityStrategy>,
    collision_strategy: Box<dyn CollisionStrategy>,
    updates_buffer: Vec<([f32; 2], [f32; 2])>,
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

        for _ in 0..count {
            let pos = [
                rng.random_range(-SPACE_SCALE..SPACE_SCALE),
                rng.random_range(-SPACE_SCALE..SPACE_SCALE),
            ];
            let vel = [rng.random_range(-15.0..15.0), rng.random_range(-10.0..10.0)];
            let radius = rng.random_range(10.0..50.0);
            let color = [
                rng.random_range(0..=255),
                rng.random_range(0..=255),
                rng.random_range(0..=255),
                255,
            ];

            self.add_body(pos, vel, radius, color, radius);
        }
    }

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

    pub fn update(&mut self, dt: f32) {
        if self.bodies.len() < 2 {
            return;
        }

        self.gravity_strategy.calculate_forces(
            &self.bodies,
            self.gravity_constant,
            dt,
            &mut self.updates_buffer,
        );

        for (i, &(position, velocity)) in self.updates_buffer.iter().enumerate() {
            self.bodies[i].position = position;
            self.bodies[i].velocity = velocity;
        }

        self.collision_strategy.handle_collisions(&mut self.bodies);
    }
}