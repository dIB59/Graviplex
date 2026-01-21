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
