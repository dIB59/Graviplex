use super::{SimulationContext, SimulationState, SimulationSystem};
use crate::simulation::spatial::kdtree::KdTree;
use crate::simulation::spatial::quadtree::Quadtree;
use rand::Rng;

pub trait CollisionStrategy: SimulationSystem {}

pub struct NoCollisionStrategy;

impl SimulationSystem for NoCollisionStrategy {
    fn update(
        &mut self,
        _state: &mut SimulationState,
        _context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
    }
}

impl CollisionStrategy for NoCollisionStrategy {}

pub struct NaiveCollisionStrategy;

impl SimulationSystem for NaiveCollisionStrategy {
    fn update(
        &mut self,
        state: &mut SimulationState,
        _context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
        let len = state.len();
        for i in 0..len {
            for j in (i + 1)..len {
                resolve_collision_soa(state, i, j);
            }
        }
    }
}

impl CollisionStrategy for NaiveCollisionStrategy {}

pub struct QuadtreeCollision;

impl QuadtreeCollision {
    pub fn new() -> Self {
        Self
    }
}

impl SimulationSystem for QuadtreeCollision {
    fn update(
        &mut self,
        state: &mut SimulationState,
        _context: &SimulationContext,
        quadtree: &Quadtree,
    ) {
        use rayon::prelude::*;

        let len = state.len();
        let mut pairs: Vec<(usize, usize)> = (0..len)
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut neighbours = Vec::new();
                let pos_i = [state.px[i], state.py[i]];
                let radius_i = state.radii[i];
                quadtree.search_radius(pos_i, radius_i * 8.0, &mut neighbours);

                neighbours
                    .into_iter()
                    .filter(move |&j| i < j)
                    .map(move |j| (i, j))
            })
            .collect();

        pairs.sort_unstable();

        for (i, j) in pairs {
            resolve_collision_soa(state, i, j);
        }
    }
}

impl CollisionStrategy for QuadtreeCollision {}

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

impl SimulationSystem for KdTreeCollision {
    fn update(
        &mut self,
        state: &mut SimulationState,
        _context: &SimulationContext,
        _quadtree: &Quadtree,
    ) {
        let mut points: Vec<(usize, [f64; 2])> = (0..state.len())
            .map(|i| (i, [state.px[i], state.py[i]]))
            .collect();

        self.tree.build(&mut points);

        use rayon::prelude::*;

        let len = state.len();
        let mut pairs: Vec<(usize, usize)> = (0..len)
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut neighbours = Vec::new();
                let pos_i = [state.px[i], state.py[i]];
                let radius_i = state.radii[i];
                self.tree
                    .search_radius(pos_i, radius_i * 8.0, &mut neighbours);

                neighbours
                    .into_iter()
                    .filter(move |&j| i < j)
                    .map(move |j| (i, j))
            })
            .collect();

        pairs.sort_unstable();

        for (i, j) in pairs {
            resolve_collision_soa(state, i, j);
        }
    }
}

impl CollisionStrategy for KdTreeCollision {}

fn resolve_collision_soa(state: &mut SimulationState, i: usize, j: usize) {
    let dx = state.px[j] - state.px[i];
    let dy = state.py[j] - state.py[i];
    let dist_sq = dx * dx + dy * dy;
    let radius_sum = state.radii[i] + state.radii[j];

    if dist_sq < 1e-15 {
        let mut rng = rand::rng();
        let angle = rng.random_range(0.0..std::f64::consts::TAU);
        let nx = angle.cos();
        let ny = angle.sin();
        let overlap = radius_sum;

        state.px[i] -= nx * (overlap * 0.5);
        state.py[i] -= ny * (overlap * 0.5);
        state.px[j] += nx * (overlap * 0.5);
        state.py[j] += ny * (overlap * 0.5);
        return;
    }

    let dist = dist_sq.sqrt();
    if dist >= radius_sum {
        return;
    }

    let nx = dx / dist;
    let ny = dy / dist;
    let overlap = radius_sum - dist;

    state.px[i] -= nx * (overlap * 0.5);
    state.py[i] -= ny * (overlap * 0.5);
    state.px[j] += nx * (overlap * 0.5);
    state.py[j] += ny * (overlap * 0.5);

    let rvx = state.vx[j] - state.vx[i];
    let rvy = state.vy[j] - state.vy[i];
    let vel_along_normal = rvx * nx + rvy * ny;
    if vel_along_normal > 0.0 {
        return;
    }

    let e = 0.8;
    let mut j_imp = -(1.0 + e) * vel_along_normal;
    j_imp /= 1.0 / state.masses[i] + 1.0 / state.masses[j];

    let impulse_x = j_imp * nx;
    let impulse_y = j_imp * ny;

    let inv_m_i = 1.0 / state.masses[i];
    let inv_m_j = 1.0 / state.masses[j];

    state.vx[i] -= inv_m_i * impulse_x;
    state.vy[i] -= inv_m_i * impulse_y;
    state.vx[j] += inv_m_j * impulse_x;
    state.vy[j] += inv_m_j * impulse_y;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::body::Body;
    use crate::simulation::spatial::quadtree::Quad;
    use crate::simulation::spatial::quadtree::Quadtree;
    use rand::Rng;

    struct QuadtreeCollisionSequential;
    impl SimulationSystem for QuadtreeCollisionSequential {
        fn update(
            &mut self,
            state: &mut SimulationState,
            _context: &SimulationContext,
            quadtree: &Quadtree,
        ) {
            let mut pairs = Vec::new();
            for i in 0..state.len() {
                let mut neighbours = Vec::new();
                let pos_i = [state.px[i], state.py[i]];
                let radius_i = state.radii[i];
                quadtree.search_radius(pos_i, radius_i * 8.0, &mut neighbours);
                for &j in neighbours.iter() {
                    if i < j {
                        pairs.push((i, j));
                    }
                }
            }
            pairs.sort_unstable();
            for (i, j) in pairs {
                resolve_collision_soa(state, i, j);
            }
        }
    }

    #[test]
    fn test_collision_parity() {
        let mut state_seq = SimulationState::new();
        let mut rng = rand::rng();

        for i in 0..100 {
            state_seq.push(Body::new(
                i,
                [rng.random_range(-10.0..10.0), rng.random_range(-10.0..10.0)],
                [rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)],
                1.0,
                [0, 0, 0, 255],
                1.0,
            ));
        }

        let mut state_par = SimulationState {
            ids: state_seq.ids.clone(),
            px: state_seq.px.clone(),
            py: state_seq.py.clone(),
            vx: state_seq.vx.clone(),
            vy: state_seq.vy.clone(),
            ax: state_seq.ax.clone(),
            ay: state_seq.ay.clone(),
            masses: state_seq.masses.clone(),
            colors: state_seq.colors.clone(),
            radii: state_seq.radii.clone(),
        };

        let root_quad = Quad::new_containing(&state_seq.px, &state_seq.py);
        let mut quadtree = Quadtree::new(0.5, 0.01);
        quadtree.build(&state_seq.px, &state_seq.py, &state_seq.masses, root_quad);

        let context = SimulationContext {
            dt: 0.016,
            gravity_constant: 10.0,
        };

        QuadtreeCollisionSequential.update(&mut state_seq, &context, &quadtree);
        KdTreeCollision::new().update(&mut state_par, &context, &quadtree);

        for i in 0..state_seq.len() {
            assert!(
                (state_seq.px[i] - state_par.px[i]).abs() < 1e-12,
                "Position mismatch at body {} between Serial and KD-Tree",
                i
            );
        }
    }
}
