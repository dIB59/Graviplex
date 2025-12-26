#[derive(Clone, Copy, Debug)]
pub struct Quad {
    pub center: [f32; 2],
    pub size: f32,
}

impl Quad {
    pub fn new_containing(positions: &[[f32; 2]]) -> Self {
        if positions.is_empty() {
            return Self {
                center: [0.0, 0.0],
                size: 2.0,
            };
        }

        let mut min = [f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN];

        for &pos in positions {
            min[0] = min[0].min(pos[0]);
            min[1] = min[1].min(pos[1]);
            max[0] = max[0].max(pos[0]);
            max[1] = max[1].max(pos[1]);
        }

        let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        let size = (max[0] - min[0]).max(max[1] - min[1]) * 1.1;

        Self { center, size }
    }

    pub fn find_quadrant(&self, pos: [f32; 2]) -> usize {
        ((pos[1] > self.center[1]) as usize) << 1 | (pos[0] > self.center[0]) as usize
    }

    pub fn into_quadrant(mut self, quadrant: usize) -> Self {
        self.size *= 0.5;
        self.center[0] += ((quadrant & 1) as f32 - 0.5) * self.size;
        self.center[1] += ((quadrant >> 1) as f32 - 0.5) * self.size;
        self
    }

    pub fn subdivide(&self) -> [Quad; 4] {
        [0, 1, 2, 3].map(|i| self.into_quadrant(i))
    }
}

#[derive(Clone)]
pub struct Node {
    pub children: usize,
    pub next: usize,
    pub pos: [f32; 2],
    pub mass: f32,
    pub quad: Quad,
}

impl Node {
    pub fn new(next: usize, quad: Quad) -> Self {
        Self {
            children: 0,
            next,
            pos: [0.0, 0.0],
            mass: 0.0,
            quad,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children == 0
    }

    pub fn is_empty(&self) -> bool {
        self.mass == 0.0
    }
}

pub struct Quadtree {
    pub t_sq: f32,
    pub e_sq: f32,
    pub nodes: Vec<Node>,
    pub parents: Vec<usize>,
}

impl Quadtree {
    pub const ROOT: usize = 0;

    pub fn new(theta: f32, epsilon: f32) -> Self {
        Self {
            t_sq: theta * theta,
            e_sq: epsilon * epsilon,
            nodes: Vec::new(),
            parents: Vec::new(),
        }
    }

    pub fn clear(&mut self, quad: Quad) {
        self.nodes.clear();
        self.parents.clear();
        self.nodes.push(Node::new(0, quad));
    }

    fn subdivide(&mut self, node: usize) -> usize {
        self.parents.push(node);
        let children = self.nodes.len();
        self.nodes[node].children = children;

        let nexts = [
            children + 1,
            children + 2,
            children + 3,
            self.nodes[node].next,
        ];
        let quads = self.nodes[node].quad.subdivide();
        for i in 0..4 {
            self.nodes.push(Node::new(nexts[i], quads[i]));
        }

        children
    }

    pub fn insert(&mut self, pos: [f32; 2], mass: f32) {
        let mut node = Self::ROOT;

        while self.nodes[node].children != 0 {
            let q = self.nodes[node].quad.find_quadrant(pos);
            node = self.nodes[node].children + q;
        }

        if self.nodes[node].is_empty() {
            self.nodes[node].pos = pos;
            self.nodes[node].mass = mass;
            return;
        }

        let (p, m) = (self.nodes[node].pos, self.nodes[node].mass);
        if pos == p {
            self.nodes[node].mass += mass;
            return;
        }

        loop {
            let children = self.subdivide(node);
            let q1 = self.nodes[node].quad.find_quadrant(p);
            let q2 = self.nodes[node].quad.find_quadrant(pos);

            if q1 == q2 {
                node = children + q1;
            } else {
                let n1 = children + q1;
                let n2 = children + q2;
                self.nodes[n1].pos = p;
                self.nodes[n1].mass = m;
                self.nodes[n2].pos = pos;
                self.nodes[n2].mass = mass;
                return;
            }
        }
    }

    pub fn propagate(&mut self) {
        for &node in self.parents.iter().rev() {
            let i = self.nodes[node].children;

            let mut pos = [0.0, 0.0];
            let mut mass = 0.0;
            for j in 0..4 {
                pos[0] += self.nodes[i + j].pos[0] * self.nodes[i + j].mass;
                pos[1] += self.nodes[i + j].pos[1] * self.nodes[i + j].mass;
                mass += self.nodes[i + j].mass;
            }

            self.nodes[node].pos = if mass > 0.0 {
                [pos[0] / mass, pos[1] / mass]
            } else {
                [0.0, 0.0]
            };
            self.nodes[node].mass = mass;
        }
    }

    pub fn acc(&self, pos: [f32; 2], gravity_constant: f32) -> [f32; 2] {
        let mut acc = [0.0, 0.0];
        let mut node = Self::ROOT;

        loop {
            let n = &self.nodes[node];
            let d = [n.pos[0] - pos[0], n.pos[1] - pos[1]];
            let d_sq = d[0] * d[0] + d[1] * d[1];

            if n.is_leaf() || n.quad.size * n.quad.size < d_sq * self.t_sq {
                let denom = (d_sq + self.e_sq) * d_sq.sqrt();
                let force_scale = (gravity_constant * n.mass / denom).min(f32::MAX);
                acc[0] += d[0] * force_scale;
                acc[1] += d[1] * force_scale;

                if n.next == 0 {
                    break;
                }
                node = n.next;
            } else {
                node = n.children;
            }
        }

        acc
    }
}

// KD-Tree for collision detection
#[derive(Debug)]
pub struct KdNode {
    point: [f32; 2],
    idx: usize,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
}

pub fn build_kd_tree(points: &mut [(usize, [f32; 2])], depth: usize) -> Option<Box<KdNode>> {
    if points.is_empty() {
        return None;
    }

    let axis = depth % 2;
    points.sort_by(|a, b| {
        a.1[axis]
            .partial_cmp(&b.1[axis])
            .expect("Invalid float comparison")
    });

    let mid = points.len() / 2;
    let (idx, point) = points[mid];

    Some(Box::new(KdNode {
        point,
        idx,
        left: build_kd_tree(&mut points[..mid], depth + 1),
        right: build_kd_tree(&mut points[mid + 1..], depth + 1),
    }))
}

pub fn search_radius(
    node: &Option<Box<KdNode>>,
    target: [f32; 2],
    radius: f32,
    depth: usize,
    results: &mut Vec<usize>,
) {
    if let Some(n) = node {
        let axis = depth % 2;
        let dist_sq = (n.point[0] - target[0]).powi(2) + (n.point[1] - target[1]).powi(2);

        if dist_sq <= radius.powi(2) {
            results.push(n.idx);
        }

        let diff = target[axis] - n.point[axis];

        if diff <= radius {
            search_radius(&n.left, target, radius, depth + 1, results);
        }
        if diff >= -radius {
            search_radius(&n.right, target, radius, depth + 1, results);
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
    fn test_quadtree_basic_insertion() {
        let mut qt = Quadtree::new(0.5, 0.01);
        let quad = Quad {
            center: [0.0, 0.0],
            size: 2.0,
        };
        qt.clear(quad);

        qt.insert([0.5, 0.5], 1.0);
        qt.propagate();

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
        qt.clear(quad);

        qt.insert([0.1, 0.1], 1.0);
        qt.insert([-0.1, -0.1], 1.0);
        qt.propagate();

        assert_eq!(qt.nodes[Quadtree::ROOT].mass, 2.0);
        assert_eq!(qt.nodes[Quadtree::ROOT].pos, [0.0, 0.0]);
        assert!(qt.nodes[Quadtree::ROOT].children != 0);
    }

    #[test]
    fn test_quadtree_acceleration() {
        let mut qt = Quadtree::new(0.0, 0.0); // No approximation, theta=0
        let quad = Quad {
            center: [0.0, 0.0],
            size: 100.0,
        };
        qt.clear(quad);

        let p1 = [1.0, 0.0];
        let m1 = 1.0;
        let p2 = [-1.0, 0.0];
        let m2 = 1.0;

        qt.insert(p1, m1);
        qt.insert(p2, m2);
        qt.propagate();

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
}
