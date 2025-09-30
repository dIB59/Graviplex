use rand::Rng;

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

// ==================== Quadtree for Barnes-Hut ====================

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
        let size = (max[0] - min[0]).max(max[1] - min[1]) * 1.1; // 10% padding

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

// ==================== Strategy Pattern ====================

pub trait GravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f32; 2], [f32; 2])>,
    );
}

pub struct NaiveGravityStrategy;

impl GravityStrategy for NaiveGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f32; 2], [f32; 2])>,
    ) {
        updates.clear();
        updates.reserve(bodies.len());

        for i in 0..bodies.len() {
            let mut total_force = [0.0, 0.0];

            for j in 0..bodies.len() {
                if i == j {
                    continue;
                }

                let body1 = &bodies[i];
                let body2 = &bodies[j];

                let dx = body2.position[0] - body1.position[0];
                let dy = body2.position[1] - body1.position[1];
                let distance_sq = dx * dx + dy * dy;
                let distance = distance_sq.sqrt();

                if distance < 1e-6 {
                    continue;
                }

                let force_magnitude = gravity_constant * body1.mass * body2.mass / distance_sq;
                let force_x = force_magnitude * (dx / distance);
                let force_y = force_magnitude * (dy / distance);

                total_force[0] += force_x;
                total_force[1] += force_y;
            }

            let body = &bodies[i];
            let acceleration = [total_force[0] / body.mass, total_force[1] / body.mass];
            let new_velocity = [
                body.velocity[0] + acceleration[0] * dt,
                body.velocity[1] + acceleration[1] * dt,
            ];
            let new_position = [
                body.position[0] + new_velocity[0] * dt,
                body.position[1] + new_velocity[1] * dt,
            ];

            updates.push((new_position, new_velocity));
        }
    }
}

pub struct BarnesHutGravityStrategy {
    quadtree: Quadtree,
}

impl BarnesHutGravityStrategy {
    pub fn new(theta: f32, epsilon: f32) -> Self {
        Self {
            quadtree: Quadtree::new(theta, epsilon),
        }
    }
}

impl GravityStrategy for BarnesHutGravityStrategy {
    fn calculate_forces(
        &mut self,
        bodies: &[Body],
        gravity_constant: f32,
        dt: f32,
        updates: &mut Vec<([f32; 2], [f32; 2])>,
    ) {
        // Build quadtree
        let positions: Vec<[f32; 2]> = bodies.iter().map(|b| b.position).collect();
        let root_quad = Quad::new_containing(&positions);
        self.quadtree.clear(root_quad);

        for body in bodies {
            self.quadtree.insert(body.position, body.mass);
        }

        self.quadtree.propagate();

        // Calculate accelerations and update
        updates.clear();
        updates.reserve(bodies.len());

        for body in bodies {
            let acc = self.quadtree.acc(body.position, gravity_constant);

            let new_velocity = [
                body.velocity[0] + acc[0] * dt,
                body.velocity[1] + acc[1] * dt,
            ];

            let new_position = [
                body.position[0] + new_velocity[0] * dt,
                body.position[1] + new_velocity[1] * dt,
            ];

            updates.push((new_position, new_velocity));
        }
    }
}

pub trait CollisionStrategy {
    fn handle_collisions(&mut self, bodies: &mut [Body]);
}

pub struct NoCollisionStrategy;

impl CollisionStrategy for NoCollisionStrategy {
    fn handle_collisions(&mut self, _bodies: &mut [Body]) {
        // Do nothing
    }
}

pub struct NaiveCollisionStrategy;

impl CollisionStrategy for NaiveCollisionStrategy {
    fn handle_collisions(&mut self, bodies: &mut [Body]) {
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let (left, right) = bodies.split_at_mut(j);
                resolve_particle_collision(&mut left[i], &mut right[0]);
            }
        }
    }
}

fn resolve_particle_collision(a: &mut Body, b: &mut Body) {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dist_sq = dx * dx + dy * dy;
    let radius_sum = a.radius + b.radius;

    // tiny epsilon to avoid float-equality problems
    let eps = 1e-8_f32;

    // if they're exactly at the same position (or extremely close), pick an arbitrary separation axis
    if dist_sq < eps {
        // push them along x a bit so they separate
        let overlap = radius_sum;
        a.position[0] -= overlap * 0.5;
        b.position[0] += overlap * 0.5;
        return;
    }

    let dist = dist_sq.sqrt();

    if dist >= radius_sum {
        return; // no collision
    }

    // Normal (unit vector) from a -> b
    let nx = dx / dist;
    let ny = dy / dist;

    // how much they overlap
    let overlap = radius_sum - dist;

    // move each by half the overlap so they just touch
    a.position[0] -= nx * (overlap * 0.5);
    a.position[1] -= ny * (overlap * 0.5);
    b.position[0] += nx * (overlap * 0.5);
    b.position[1] += ny * (overlap * 0.5);
}

#[derive(Debug)]
struct KdNode {
    point: [f32; 2],
    idx: usize,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
}

fn build_kd_tree(points: &mut [(usize, [f32; 2])], depth: usize) -> Option<Box<KdNode>> {
    if points.is_empty() {
        return None;
    }

    let axis = depth % 2;
    points.sort_by(|a, b| a.1[axis].partial_cmp(&b.1[axis]).unwrap());

    let mid = points.len() / 2;
    let (idx, point) = points[mid];

    Some(Box::new(KdNode {
        point,
        idx,
        left: build_kd_tree(&mut points[..mid], depth + 1),
        right: build_kd_tree(&mut points[mid + 1..], depth + 1),
    }))
}

fn search_radius(
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

pub struct KdTreeCollision;

impl CollisionStrategy for KdTreeCollision {
    fn handle_collisions(&mut self, bodies: &mut [Body]) {
        // Build KD-tree points from body positions
        let mut points: Vec<(usize, [f32; 2])> = bodies
            .iter()
            .enumerate()
            .map(|(idx, body)| (idx, [body.position[0], body.position[1]]))
            .collect();
        let tree = build_kd_tree(&mut points, 0);

        for i in 0..bodies.len() {
            let mut neighbours = Vec::new();
            let (pos_i, radius_i) = {
                let body = &bodies[i];
                ([body.position[0], body.position[1]], body.radius)
            };

            search_radius(&tree, pos_i, radius_i * 2.0, 0, &mut neighbours);

            for &j in neighbours.iter() {
                if i >= j {
                    continue; // Only process each pair once
                }

                let (left, right) = bodies.split_at_mut(j);
                resolve_particle_collision(&mut left[i], &mut right[0]);
            }
        }
    }
}

// ==================== N-body Simulation ====================

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
    /// Create a new n-body simulation with default strategi
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            next_id: 0,
            gravity_constant: 100.0,
            gravity_strategy: Box::new(BarnesHutGravityStrategy::new(0.5, 0.01)),
            collision_strategy: Box::new(NaiveCollisionStrategy),
            updates_buffer: Vec::new(),
        }
    }

    /// Set the gravity calculation strategy
    pub fn set_gravity_strategy(&mut self, strategy: Box<dyn GravityStrategy>) {
        self.gravity_strategy = strategy;
    }

    /// Set the collision detection strategy
    pub fn set_collision_strategy(&mut self, strategy: Box<dyn CollisionStrategy>) {
        self.collision_strategy = strategy;
    }

    pub fn generate_bodies(&mut self, range: i32) {
        let mut rng = rand::rng();

        for _ in 0..range {
            let pos = [rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)];

            let vel = [rng.random_range(-0.01..0.01), rng.random_range(-0.01..0.01)];

            let radius = rng.random_range(0.01..0.05);

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

    pub fn get_body(&self, id: u32) -> Option<&Body> {
        self.bodies.iter().find(|b| b.id == id)
    }

    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter()
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
        self.next_id = 0;
    }

    pub fn set_gravity_constant(&mut self, g: f32) {
        self.gravity_constant = g;
    }

    pub fn gravity_constant(&self) -> f32 {
        self.gravity_constant
    }

    /// Update simulation by one time step
    pub fn update(&mut self, dt: f32) {
        if self.bodies.len() < 2 {
            return;
        }

        // Calculate gravity forces and get position/velocity updates
        self.gravity_strategy.calculate_forces(
            &self.bodies,
            self.gravity_constant,
            dt,
            &mut self.updates_buffer,
        );

        // Apply updates
        for (i, &(position, velocity)) in self.updates_buffer.iter().enumerate() {
            self.bodies[i].position = position;
            self.bodies[i].velocity = velocity;
        }

        // Handle collisions
        self.collision_strategy.handle_collisions(&mut self.bodies);
    }

    pub fn find_nearest_body(&self, position: [f32; 2]) -> Option<u32> {
        let mut nearest_id = None;
        let mut min_distance = f32::INFINITY;

        for body in &self.bodies {
            let dx = body.position[0] - position[0];
            let dy = body.position[1] - position[1];
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < min_distance {
                min_distance = distance;
                nearest_id = Some(body.id);
            }
        }

        nearest_id
    }
}
