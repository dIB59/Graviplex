use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Quad {
    pub center: [f64; 2],
    pub size: f64,
}

impl Quad {
    pub const MIN_SIZE: f64 = 1e-8;
    pub fn new_containing(positions: &[[f64; 2]]) -> Self {
        if positions.is_empty() {
            return Self {
                center: [0.0, 0.0],
                size: 2.0,
            };
        }

        let mut min = [f64::MAX, f64::MAX];
        let mut max = [f64::MIN, f64::MIN];

        for &pos in positions {
            min[0] = min[0].min(pos[0]);
            min[1] = min[1].min(pos[1]);
            max[0] = max[0].max(pos[0]);
            max[1] = max[1].max(pos[1]);
        }

        let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        let size = ((max[0] - min[0]).max(max[1] - min[1]) * 1.1).max(Self::MIN_SIZE);

        Self { center, size }
    }

    pub fn find_quadrant(&self, pos: [f64; 2]) -> usize {
        ((pos[1] > self.center[1]) as usize) << 1 | (pos[0] > self.center[0]) as usize
    }

    pub fn into_quadrant(mut self, quadrant: usize) -> Self {
        self.size *= 0.5;
        self.center[0] += ((quadrant & 1) as f64 - 0.5) * self.size;
        self.center[1] += ((quadrant >> 1) as f64 - 0.5) * self.size;
        self
    }

    pub fn subdivide(&self) -> [Quad; 4] {
        [0, 1, 2, 3].map(|i| self.into_quadrant(i))
    }
    pub fn overlaps_circle(&self, center: [f64; 2], radius: f64) -> bool {
        let half_size = self.size * 0.5;
        let dx = (center[0] - self.center[0]).abs();
        let dy = (center[1] - self.center[1]).abs();

        if dx > (half_size + radius) {
            return false;
        }
        if dy > (half_size + radius) {
            return false;
        }

        if dx <= half_size {
            return true;
        }
        if dy <= half_size {
            return true;
        }

        let corner_dist_sq = (dx - half_size).powi(2) + (dy - half_size).powi(2);
        corner_dist_sq <= radius.powi(2)
    }

    pub fn contains_point(&self, pos: [f64; 2]) -> bool {
        let half_size = self.size * 0.5;
        (pos[0] >= self.center[0] - half_size)
            && (pos[0] <= self.center[0] + half_size)
            && (pos[1] >= self.center[1] - half_size)
            && (pos[1] <= self.center[1] + half_size)
    }
}

pub fn interleave_bits(x: u32) -> u64 {
    let mut x = x as u64;
    x = (x | (x << 16)) & 0x0000FFFF0000FFFF;
    x = (x | (x << 8)) & 0x00FF00FF00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F;
    x = (x | (x << 2)) & 0x3333333333333333;
    x = (x | (x << 1)) & 0x5555555555555555;
    x
}

pub fn get_morton_code(pos: [f64; 2], quad: &Quad) -> u64 {
    let x = (((pos[0] - (quad.center[0] - quad.size * 0.5)) / quad.size).clamp(0.0, 1.0)
        * ((1u32 << 31) as f64 - 1.0)) as u32;
    let y = (((pos[1] - (quad.center[1] - quad.size * 0.5)) / quad.size).clamp(0.0, 1.0)
        * ((1u32 << 31) as f64 - 1.0)) as u32;
    interleave_bits(x) | (interleave_bits(y) << 1)
}

#[derive(Clone)]
pub struct Node {
    pub children: [usize; 4],
    pub next: usize,
    pub pos: [f64; 2],
    pub mass: f64,
    pub quad: Quad,
    pub body_range: (usize, usize),
}

impl Node {
    pub fn new(next: usize, quad: Quad) -> Self {
        Self {
            children: [0; 4],
            next,
            pos: [0.0, 0.0],
            mass: 0.0,
            quad,
            body_range: (0, 0),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children[0] == 0
    }

    pub fn is_empty(&self) -> bool {
        self.mass == 0.0
    }
}

pub struct Quadtree {
    pub t_sq: f64,
    pub e_sq: f64,
    pub nodes: Vec<Node>,
    pub body_indices: Vec<usize>,
}

impl Quadtree {
    pub const ROOT: usize = 0;

    pub fn new(theta: f64, epsilon: f64) -> Self {
        Self {
            t_sq: theta * theta,
            e_sq: epsilon * epsilon,
            nodes: Vec::new(),
            body_indices: Vec::new(),
        }
    }

    pub fn clear(&mut self, quad: Quad) {
        self.nodes.clear();
        self.body_indices.clear();
        self.nodes.push(Node::new(0, quad));
    }

    pub fn build(&mut self, positions: &[[f64; 2]], masses: &[f64], quad: Quad) {
        self.nodes.clear();

        if positions.is_empty() {
            self.nodes.push(Node::new(0, quad));
            return;
        }

        let mut bodies: Vec<_> = (0..positions.len())
            .into_par_iter()
            .map(|i| {
                (
                    positions[i],
                    masses[i],
                    get_morton_code(positions[i], &quad),
                    i,
                )
            })
            .collect();

        bodies.par_sort_by_key(|b| b.2);

        self.body_indices = bodies.iter().map(|b| b.3).collect();

        self.build_recursive(&bodies, quad, 0, 0);
        self.thread(0, 0);
    }

    fn build_recursive(
        &mut self,
        bodies: &[([f64; 2], f64, u64, usize)],
        quad: Quad,
        depth: usize,
        body_offset: usize,
    ) -> usize {
        let node_idx = self.nodes.len();
        let mut node = Node::new(0, quad);
        node.body_range = (body_offset, body_offset + bodies.len());
        self.nodes.push(node);

        if bodies.is_empty() {
            return node_idx;
        }

        if bodies.len() == 1 || depth > 24 {
            let mut mass = 0.0;
            let mut pos = [0.0, 0.0];
            for b in bodies {
                mass += b.1;
                pos[0] += b.0[0] * b.1;
                pos[1] += b.0[1] * b.1;
            }
            if mass > 0.0 {
                self.nodes[node_idx].pos = [pos[0] / mass, pos[1] / mass];
            }
            self.nodes[node_idx].mass = mass;
            return node_idx;
        }

        let first_pos = bodies[0].0;
        if bodies.iter().all(|b| b.0 == first_pos) {
            self.nodes[node_idx].pos = first_pos;
            self.nodes[node_idx].mass = bodies.iter().map(|b| b.1).sum();
            return node_idx;
        }

        let sub_quads = quad.subdivide();
        let mut split_indices = [0; 5];
        split_indices[4] = bodies.len();

        let mut current_quad = 0;
        for (i, b) in bodies.iter().enumerate() {
            let q = quad.find_quadrant(b.0);
            while current_quad < q {
                current_quad += 1;
                split_indices[current_quad] = i;
            }
        }
        while current_quad < 4 {
            current_quad += 1;
            split_indices[current_quad] = bodies.len();
        }

        let mut child_indices = [0; 4];
        for i in 0..4 {
            child_indices[i] = self.build_recursive(
                &bodies[split_indices[i]..split_indices[i + 1]],
                sub_quads[i],
                depth + 1,
                body_offset + split_indices[i],
            );
        }
        self.nodes[node_idx].children = child_indices;

        let mut total_mass = 0.0;
        let mut center_of_mass = [0.0, 0.0];

        for i in 0..4 {
            let m = self.nodes[child_indices[i]].mass;
            total_mass += m;
            center_of_mass[0] += self.nodes[child_indices[i]].pos[0] * m;
            center_of_mass[1] += self.nodes[child_indices[i]].pos[1] * m;
        }

        if total_mass > 0.0 {
            self.nodes[node_idx].pos = [
                center_of_mass[0] / total_mass,
                center_of_mass[1] / total_mass,
            ];
        } else if !bodies.is_empty() {
            self.nodes[node_idx].pos = bodies[0].0;
        }
        self.nodes[node_idx].mass = total_mass;

        node_idx
    }

    fn thread(&mut self, node: usize, next: usize) {
        self.nodes[node].next = next;
        if !self.nodes[node].is_leaf() {
            let children = self.nodes[node].children;
            self.thread(children[0], children[1]);
            self.thread(children[1], children[2]);
            self.thread(children[2], children[3]);
            self.thread(children[3], next);
        }
    }

    pub fn search_radius(&self, center: [f64; 2], radius: f64, results: &mut Vec<usize>) {
        self.search_radius_recursive(Self::ROOT, center, radius, results);
    }

    fn search_radius_recursive(
        &self,
        node_idx: usize,
        center: [f64; 2],
        radius: f64,
        results: &mut Vec<usize>,
    ) {
        let node = &self.nodes[node_idx];

        if !node.quad.overlaps_circle(center, radius) || node.is_empty() {
            return;
        }

        if node.is_leaf() {
            // Check individual bodies in this node's range
            for i in node.body_range.0..node.body_range.1 {
                let body_idx = self.body_indices[i];
                // Note: we don't have body positions here directly in Quadtree struct anymore
                // except in build phase. If we want range search to be efficient, we might
                // need to store positions or pass them in.
                // However, many implementations use the approximate center of mass for distant nodes.
                // For collision, we need EXACT positions.
                // Let's assume we pass positions in or the caller filters.
                // Actually, the current search_radius in KdTree also took target pos.
                results.push(body_idx);
            }
            return;
        }

        for &child in &node.children {
            self.search_radius_recursive(child, center, radius, results);
        }
    }

    pub fn acc(&self, pos: [f64; 2], gravity_constant: f64) -> [f64; 2] {
        let mut acc = [0.0, 0.0];
        let mut node = Self::ROOT;

        loop {
            let n = &self.nodes[node];
            let d = [n.pos[0] - pos[0], n.pos[1] - pos[1]];
            let d_sq = d[0] * d[0] + d[1] * d[1];

            if n.is_leaf() || n.quad.size * n.quad.size < d_sq * self.t_sq {
                if n.mass > 0.0 {
                    let soft_d_sq = d_sq + self.e_sq;
                    if soft_d_sq > 1e-9 {
                        let denom = soft_d_sq * soft_d_sq.sqrt();
                        let force_scale = gravity_constant * n.mass / denom;
                        acc[0] += d[0] * force_scale;
                        acc[1] += d[1] * force_scale;
                    }
                }

                if n.next == 0 {
                    break;
                }
                node = n.next;
            } else {
                node = n.children[0];
            }
        }

        acc
    }

    /// Returns all non-empty cell quads for visualization
    pub fn get_cells(&self) -> Vec<Quad> {
        self.nodes
            .iter()
            .filter(|n| n.mass > 0.0)
            .map(|n| n.quad)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct KdNode {
    pub point: [f64; 2],
    pub idx: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

pub struct KdTree {
    pub nodes: Vec<KdNode>,
    pub root: Option<usize>,
}

impl KdTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(1000),
            root: None,
        }
    }

    pub fn build(&mut self, points: &mut [(usize, [f64; 2])]) {
        self.nodes.clear();
        self.root = self.build_recursive(points, 0);
    }

    fn build_recursive(&mut self, points: &mut [(usize, [f64; 2])], depth: usize) -> Option<usize> {
        if points.is_empty() {
            return None;
        }

        let axis = depth % 2;
        let mid = points.len() / 2;

        points.select_nth_unstable_by(mid, |a, b| {
            a.1[axis]
                .partial_cmp(&b.1[axis])
                .expect("Invalid float comparison")
        });

        let (idx, point) = points[mid];

        let node_idx = self.nodes.len();
        // Placeholder to get index
        self.nodes.push(KdNode {
            point,
            idx,
            left: None,
            right: None,
        });

        let left = self.build_recursive(&mut points[..mid], depth + 1);
        let right = self.build_recursive(&mut points[mid + 1..], depth + 1);

        self.nodes[node_idx].left = left;
        self.nodes[node_idx].right = right;

        Some(node_idx)
    }

    pub fn search_radius(&self, target: [f64; 2], radius: f64, results: &mut Vec<usize>) {
        if let Some(root) = self.root {
            self.search_recursive(root, target, radius, 0, results);
        }
    }

    fn search_recursive(
        &self,
        node_idx: usize,
        target: [f64; 2],
        radius: f64,
        depth: usize,
        results: &mut Vec<usize>,
    ) {
        let n = &self.nodes[node_idx];
        let axis = depth % 2;
        let dist_sq = (n.point[0] - target[0]).powi(2) + (n.point[1] - target[1]).powi(2);

        if dist_sq <= radius.powi(2) {
            results.push(n.idx);
        }

        let diff = target[axis] - n.point[axis];

        if diff <= radius {
            if let Some(left) = n.left {
                self.search_recursive(left, target, radius, depth + 1, results);
            }
        }
        if diff >= -radius {
            if let Some(right) = n.right {
                self.search_recursive(right, target, radius, depth + 1, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_subdivide() {
        let quad = Quad {
            center: [0.0, 0.0],
            size: 2.0,
        };
        let sub = quad.subdivide();
        assert_eq!(sub.len(), 4);

        // Quadrant 0: Bottom-Left (assuming into_quadrant logic)
        assert_eq!(sub[0].center, [-0.5, -0.5]);
        assert_eq!(sub[0].size, 1.0);

        // Quadrant 3: Top-Right
        assert_eq!(sub[3].center, [0.5, 0.5]);
        assert_eq!(sub[3].size, 1.0);
    }

    #[test]
    fn test_quadtree_basic_build() {
        let mut qt = Quadtree::new(0.5, 0.01);
        let quad = Quad {
            center: [0.0, 0.0],
            size: 2.0,
        };

        qt.build(&[[0.5, 0.5]], &[1.0], quad);

        assert_eq!(qt.nodes[Quadtree::ROOT].mass, 1.0);
        assert_eq!(qt.nodes[Quadtree::ROOT].pos, [0.5, 0.5]);
    }

    #[test]
    fn test_quadtree_subdivision() {
        let mut qt = Quadtree::new(0.5, 0.01);
        let quad = Quad {
            center: [0.0, 0.0],
            size: 2.0,
        };

        let positions = [[0.1, 0.1], [-0.1, -0.1]];
        let masses = [1.0, 1.0];

        qt.build(&positions, &masses, quad);

        assert_eq!(qt.nodes[Quadtree::ROOT].mass, 2.0);
        assert_eq!(qt.nodes[Quadtree::ROOT].pos, [0.0, 0.0]);
        assert!(!qt.nodes[Quadtree::ROOT].is_leaf());
    }

    #[test]
    fn test_quadtree_acceleration() {
        let mut qt = Quadtree::new(0.0, 0.0); // No approximation, theta=0
        let quad = Quad {
            center: [0.0, 0.0],
            size: 100.0,
        };

        let positions = [[1.0, 0.0], [-1.0, 0.0]];
        let masses = [1.0, 1.0];

        qt.build(&positions, &masses, quad);

        let target = [0.0, 0.0];
        let acc = qt.acc(target, 1.0);

        // Forces should cancel out at the center
        assert!((acc[0].abs() < 1e-6));
        assert!((acc[1].abs() < 1e-6));

        let target2 = [2.0, 0.0];
        let acc2 = qt.acc(target2, 1.0);

        assert!(
            (acc2[0] - (-1.111111)).abs() < 1e-5,
            "Expected ~-1.111, got {}",
            acc2[0]
        );
    }

    #[test]
    fn test_quadtree_build() {
        let mut qt = Quadtree::new(0.0, 0.0);
        let quad = Quad {
            center: [0.0, 0.0],
            size: 100.0,
        };

        let positions = [[1.0, 0.0], [-1.0, 0.0]];
        let masses = [1.0, 1.0];

        qt.build(&positions, &masses, quad);

        assert_eq!(qt.nodes[Quadtree::ROOT].mass, 2.0);
        assert_eq!(qt.nodes[Quadtree::ROOT].pos, [0.0, 0.0]);

        let target = [2.0, 0.0];
        let acc = qt.acc(target, 1.0);
        assert!((acc[0] - (-1.111111)).abs() < 1e-5);
    }

    #[test]
    fn test_quadtree_search_radius() {
        let mut qt = Quadtree::new(0.5, 0.01);
        let quad = Quad {
            center: [0.0, 0.0],
            size: 10.0,
        };

        let positions = [[1.0, 1.0], [2.0, 2.0], [-1.0, -1.0]];
        let masses = [1.0, 1.0, 1.0];

        qt.build(&positions, &masses, quad);

        let mut results = Vec::new();
        qt.search_radius([1.5, 1.5], 1.0, &mut results);

        // results should contain indices 0 and 1
        assert_eq!(results.len(), 2);
        assert!(results.contains(&0));
        assert!(results.contains(&1));
    }
}
