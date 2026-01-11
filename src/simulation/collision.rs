use super::spatial::{KdTree, Quadtree};
use super::Body;
use rand::Rng;

pub trait CollisionStrategy {
    fn handle_collisions(&mut self, bodies: &mut [Body], quadtree: &Quadtree);
}

pub struct NoCollisionStrategy;

impl CollisionStrategy for NoCollisionStrategy {
    fn handle_collisions(&mut self, _bodies: &mut [Body], _quadtree: &Quadtree) {}
}

pub struct NaiveCollisionStrategy;

impl CollisionStrategy for NaiveCollisionStrategy {
    fn handle_collisions(&mut self, bodies: &mut [Body], _quadtree: &Quadtree) {
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let (left, right) = bodies.split_at_mut(j);
                resolve_collision(&mut left[i], &mut right[0]);
            }
        }
    }
}

pub struct QuadtreeCollision;

impl QuadtreeCollision {
    pub fn new() -> Self {
        Self
    }
}

pub struct KdTreeCollision {
    tree: KdTree,
}

impl KdTreeCollision {
    pub fn new() -> Self {
        Self {
            tree: KdTree::new(),
        }
    }
}

impl CollisionStrategy for QuadtreeCollision {
    fn handle_collisions(&mut self, bodies: &mut [Body], quadtree: &Quadtree) {
        use rayon::prelude::*;

        let pairs: Vec<(usize, usize)> = (0..bodies.len())
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut neighbours = Vec::new();
                let (pos_i, radius_i) = (bodies[i].position, bodies[i].radius);
                quadtree.search_radius(pos_i, radius_i * 5.0, &mut neighbours);

                neighbours
                    .into_iter()
                    .filter(move |&j| i < j)
                    .map(move |j| (i, j))
            })
            .collect();

        // Sort pairs for deterministic resolution order
        //pairs.sort_unstable();

        for (i, j) in pairs {
            let (left, right) = bodies.split_at_mut(j);
            resolve_collision(&mut left[i], &mut right[0]);
        }
    }
}

impl CollisionStrategy for KdTreeCollision {
    fn handle_collisions(&mut self, bodies: &mut [Body], _quadtree: &Quadtree) {
        let mut points: Vec<(usize, [f64; 2])> = bodies
            .iter()
            .enumerate()
            .map(|(idx, body)| (idx, body.position))
            .collect();

        self.tree.build(&mut points);

        use rayon::prelude::*;

        let mut pairs: Vec<(usize, usize)> = (0..bodies.len())
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut neighbours = Vec::new();
                let (pos_i, radius_i) = (bodies[i].position, bodies[i].radius);
                self.tree
                    .search_radius(pos_i, radius_i * 5.0, &mut neighbours);

                neighbours
                    .into_iter()
                    .filter(move |&j| i < j)
                    .map(move |j| (i, j))
            })
            .collect();
        // Sort pairs for deterministic resolution order
        //pairs.sort_unstable();

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

    struct QuadtreeCollisionSequential;
    impl CollisionStrategy for QuadtreeCollisionSequential {
        fn handle_collisions(&mut self, bodies: &mut [Body], quadtree: &Quadtree) {
            let mut pairs = Vec::new();
            for i in 0..bodies.len() {
                let mut neighbours = Vec::new();
                let (pos_i, radius_i) = (bodies[i].position, bodies[i].radius);
                quadtree.search_radius(pos_i, radius_i * 8.0, &mut neighbours);
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
        use crate::simulation::spatial::Quad;

        let positions: Vec<[f64; 2]> = bodies_seq.iter().map(|b| b.position).collect();
        let masses: Vec<f64> = bodies_seq.iter().map(|b| b.mass).collect();
        let root_quad = Quad::new_containing(&positions);
        let mut quadtree = Quadtree::new(0.5, 0.01);
        quadtree.build(&positions, &masses, root_quad);

        QuadtreeCollisionSequential.handle_collisions(&mut bodies_seq, &quadtree);
        KdTreeCollision::new().handle_collisions(&mut bodies_par, &quadtree);

        for i in 0..bodies_seq.len() {
            assert!(
                (bodies_seq[i].position[0] - bodies_par[i].position[0]).abs() < 1e-12,
                "Position mismatch at body {} between Serial and KD-Tree",
                i
            );
            assert!(
                (bodies_seq[i].velocity[0] - bodies_par[i].velocity[0]).abs() < 1e-12,
                "Velocity mismatch at body {} between Serial and KD-Tree",
                i
            );
        }
    }
}
