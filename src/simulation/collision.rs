use super::spatial::{build_kd_tree, search_radius};
use super::Body;
use rand::Rng;

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
        let mut points: Vec<(usize, [f64; 2])> = bodies
            .iter()
            .enumerate()
            .map(|(idx, body)| (idx, body.position))
            .collect();

        let tree = build_kd_tree(&mut points, 0);

        use rayon::prelude::*;

        let mut pairs: Vec<(usize, usize)> = (0..bodies.len())
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut neighbours = Vec::new();
                let (pos_i, radius_i) = (bodies[i].position, bodies[i].radius);
                search_radius(&tree, pos_i, radius_i * 5.0, 0, &mut neighbours);

                neighbours
                    .into_iter()
                    .filter(move |&j| i < j)
                    .map(move |j| (i, j))
            })
            .collect();

        // Sort pairs for deterministic resolution order
        pairs.sort_unstable();

        for (i, j) in pairs {
            let (left, right) = bodies.split_at_mut(j);
            resolve_collision(&mut left[i], &mut right[0]);
        }
    }
}

fn resolve_collision(a: &mut Body, b: &mut Body) {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dist_sq = dx * dx + dy * dy;
    let radius_sum = a.radius + b.radius;

    if dist_sq < 1e-15 {
        let mut rng = rand::rng();
        let angle = rng.random_range(0.0..std::f64::consts::TAU);
        let nx = angle.cos();
        let ny = angle.sin();
        let overlap = radius_sum;

        a.position[0] -= nx * (overlap * 0.5);
        a.position[1] -= ny * (overlap * 0.5);
        b.position[0] += nx * (overlap * 0.5);
        b.position[1] += ny * (overlap * 0.5);
        return;
    }

    let dist = dist_sq.sqrt();
    if dist >= radius_sum {
        return;
    }

    let nx = dx / dist;
    let ny = dy / dist;
    let overlap = radius_sum - dist;

    // 1. Position resolution (push apart)
    a.position[0] -= nx * (overlap * 0.5);
    a.position[1] -= ny * (overlap * 0.5);
    b.position[0] += nx * (overlap * 0.5);
    b.position[1] += ny * (overlap * 0.5);

    // 2. Velocity resolution (elastic collision)
    let rvx = b.velocity[0] - a.velocity[0];
    let rvy = b.velocity[1] - a.velocity[1];

    let vel_along_normal = rvx * nx + rvy * ny;

    // Only resolve if velocities are approaching
    if vel_along_normal > 0.0 {
        return;
    }

    let e = 0.8; // Restitution coefficient
    let mut j = -(1.0 + e) * vel_along_normal;
    j /= 1.0 / a.mass + 1.0 / b.mass;

    let impulse_x = j * nx;
    let impulse_y = j * ny;

    a.velocity[0] -= 1.0 / a.mass * impulse_x;
    a.velocity[1] -= 1.0 / a.mass * impulse_y;
    b.velocity[0] += 1.0 / b.mass * impulse_x;
    b.velocity[1] += 1.0 / b.mass * impulse_y;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Body;
    use rand::Rng;

    struct KdTreeCollisionSequential;
    impl CollisionStrategy for KdTreeCollisionSequential {
        fn handle_collisions(&mut self, bodies: &mut [Body]) {
            let mut points: Vec<(usize, [f64; 2])> = bodies
                .iter()
                .enumerate()
                .map(|(idx, body)| (idx, body.position))
                .collect();
            let tree = build_kd_tree(&mut points, 0);

            let mut pairs = Vec::new();
            for i in 0..bodies.len() {
                let mut neighbours = Vec::new();
                let (pos_i, radius_i) = (bodies[i].position, bodies[i].radius);
                search_radius(&tree, pos_i, radius_i * 8.0, 0, &mut neighbours);
                for &j in neighbours.iter() {
                    if i < j {
                        pairs.push((i, j));
                    }
                }
            }
            pairs.sort_unstable();
            for (i, j) in pairs {
                let (left, right) = bodies.split_at_mut(j);
                resolve_collision(&mut left[i], &mut right[0]);
            }
        }
    }

    #[test]
    fn test_collision_parity() {
        let mut bodies_seq = Vec::new();
        let mut rng = rand::rng();

        for i in 0..100 {
            bodies_seq.push(Body::new(
                i,
                [rng.random_range(-10.0..10.0), rng.random_range(-10.0..10.0)],
                [rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)],
                1.0,
                [0, 0, 0, 255],
                1.0,
            ));
        }

        let mut bodies_par = bodies_seq.clone();

        KdTreeCollisionSequential.handle_collisions(&mut bodies_seq);
        KdTreeCollision.handle_collisions(&mut bodies_par);

        for i in 0..bodies_seq.len() {
            assert!(
                (bodies_seq[i].position[0] - bodies_par[i].position[0]).abs() < 1e-12,
                "Position mismatch at body {}",
                i
            );
            assert!(
                (bodies_seq[i].velocity[0] - bodies_par[i].velocity[0]).abs() < 1e-12,
                "Velocity mismatch at body {}",
                i
            );
        }
    }
}
