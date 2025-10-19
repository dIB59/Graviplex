use super::Body;
use super::spatial::{build_kd_tree, search_radius};

pub trait CollisionStrategy {
    fn handle_collisions(&mut self, bodies: &mut [Body]);
}

pub struct NoCollisionStrategy;

impl CollisionStrategy for NoCollisionStrategy {
    fn handle_collisions(&mut self, _bodies: &mut [Body]) {}
}

pub struct NaiveCollisionStrategy;

impl CollisionStrategy for NaiveCollisionStrategy {
    fn handle_collisions(&mut self, bodies: &mut [Body]) {
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let (left, right) = bodies.split_at_mut(j);
                resolve_collision(&mut left[i], &mut right[0]);
            }
        }
    }
}

pub struct KdTreeCollision;

impl CollisionStrategy for KdTreeCollision {
    fn handle_collisions(&mut self, bodies: &mut [Body]) {
        let mut points: Vec<(usize, [f32; 2])> = bodies
            .iter()
            .enumerate()
            .map(|(idx, body)| (idx, body.position))
            .collect();
        
        let tree = build_kd_tree(&mut points, 0);

        for i in 0..bodies.len() {
            let mut neighbours = Vec::new();
            let (pos_i, radius_i) = (bodies[i].position, bodies[i].radius);

            search_radius(&tree, pos_i, radius_i * 5.0, 0, &mut neighbours);

            for &j in neighbours.iter() {
                if i >= j {
                    continue;
                }
                let (left, right) = bodies.split_at_mut(j);
                resolve_collision(&mut left[i], &mut right[0]);
            }
        }
    }
}

fn resolve_collision(a: &mut Body, b: &mut Body) {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dist_sq = dx * dx + dy * dy;
    let radius_sum = a.radius + b.radius;

    if dist_sq < 1e-6 {
        let overlap = radius_sum;
        a.position[0] -= overlap * 0.5;
        b.position[0] += overlap * 0.5;
        return;
    }

    let dist = dist_sq.sqrt();
    if dist >= radius_sum {
        return;
    }

    let nx = dx / dist;
    let ny = dy / dist;
    let overlap = radius_sum - dist;

    a.position[0] -= nx * (overlap * 0.5);
    a.position[1] -= ny * (overlap * 0.5);
    b.position[0] += nx * (overlap * 0.5);
    b.position[1] += ny * (overlap * 0.5);
}
