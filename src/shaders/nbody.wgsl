struct Particle {
    position: vec2<f32>,
    radius: f32,
    color: u32,
    velocity: vec2<f32>,
    mass: f32,
    id: u32,
}

struct Params {
    dt: f32,
    gravity: f32,
    num_particles: u32,
    theta: f32,
    seed: u32,
    pass_index: u32,       // Multi-purpose pass counter
    num_nodes: u32,        // Internal nodes count
}

struct Node {
    children: vec4<i32>, // Indices of children
    pos: vec2<f32>,      // Center of mass
    mass: f32,
    parent: i32,
    size: f32,           // Bounding box size
    next: i32,           // Next node for stack-less traversal
}

struct KeyValue {
    key: u32,
    value: u32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read_write> nodes: array<Node>;
@group(0) @binding(3) var<storage, read_write> sort_data: array<KeyValue>;
@group(0) @binding(4) var<storage, read_write> sort_temp: array<KeyValue>;
@group(0) @binding(5) var<storage, read_write> histograms: array<atomic<u32>>;

fn hash(n: u32) -> u32 {
    var x = n;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = (x >> 16u) ^ x;
    return x;
}

fn rand_f32(n: u32) -> f32 {
    return f32(hash(n)) / 4294967295.0;
}

@compute @workgroup_size(256)
fn init_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    let seed = params.seed + i;
    // Spread out more: radius up to 1800
    let r = sqrt(rand_f32(seed)) * 1800.0;
    let theta = rand_f32(seed + 1000000u) * 6.2831853;

    let pos = vec2<f32>(r * cos(theta), r * sin(theta));
    // Orbital velocity approximation: v = sqrt(G * M / r)
    // For a 1M particle simulation, we just scale it to look good.
    let vel_mag = 0.5 * sqrt(800.0 / (r + 1.0));
    let vel = vec2<f32>(-pos.y, pos.x) * vel_mag;

    particles[i].position = pos;
    particles[i].velocity = vel;
    particles[i].radius = 1.0;
    particles[i].mass = 1.0;
    particles[i].color = 0xFFFFFFFFu;
    particles[i].id = i;
}

fn interleave_bits(n: u32) -> u32 {
    var x = n & 0x0000FFFFu;
    x = (x | (x << 8u)) & 0x00FF00FFu;
    x = (x | (x << 4u)) & 0x0F0F0F0Fu;
    x = (x | (x << 2u)) & 0x33333333u;
    x = (x | (x << 1u)) & 0x55555555u;
    return x;
}

@compute @workgroup_size(256)
fn morton_encode(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    // Domain for 1M particles: [-2000, 2000]
    let p = particles[i].position;
    let x = u32(clamp((p.x + 2000.0) / 4000.0, 0.0, 1.0) * 65535.0);
    let y = u32(clamp((p.y + 2000.0) / 4000.0, 0.0, 1.0) * 65535.0);

    let code = interleave_bits(x) | (interleave_bits(y) << 1u);
    sort_data[i].key = code;
    sort_data[i].value = i;
}

// Radix Sort Pass 1: Local Histogram
@compute @workgroup_size(256)
fn radix_histogram(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    let key = sort_data[i].key;
    let bit_offset = params.pass_index * 4u;
    let digit = (key >> bit_offset) & 0xFu;

    // Use atomicAdd into histograms buffer
    // Layout: [pass_index][digit]
    // Since histograms are global, we need unique entries for each workgroup if we want to avoid massive contention,
    // but for now, we'll use a simple global atomic for simplicity and improve if it's too slow.
    atomicAdd(&histograms[digit], 1u);
}

// Radix Sort Pass 3: Shuffle
@compute @workgroup_size(256)
fn radix_shuffle(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // This requires the scanned histograms (offsets)
    // To be implemented once prefix scan is ready
}

// --- Tree Building Pass (Karras 2012) ---

fn common_upper_bits(a: u32, b: u32) -> i32 {
    return i32(countLeadingZeros(a ^ b));
}

fn delta(i: i32, j: i32, n: i32) -> i32 {
    if j < 0 || j >= n { return -1; }
    let a = sort_data[i].key;
    let b = sort_data[j].key;
    if a == b { return 32 + i32(countLeadingZeros(u32(i ^ j))); }
    return i32(countLeadingZeros(a ^ b));
}

@compute @workgroup_size(256)
fn build_tree(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = i32(global_id.x);
    let n = i32(params.num_particles);
    if i >= n - 1 { return; }

    // 1. Determine direction of the range
    let d = select(-1, 1, delta(i, i + 1, n) - delta(i, i - 1, n) >= 0);

    // 2. Compute upper bound for the length of the range
    let delta_min = delta(i, i - d, n);
    var l_max = 2;
    while delta(i, i + l_max * d, n) > delta_min {
        l_max = l_max * 2;
    }

    // 3. Find the other end using binary search
    var l = 0;
    var t = l_max / 2;
    while t > 0 {
        if delta(i, i + (l + t) * d, n) > delta_min {
            l = l + t;
        }
        t = t / 2;
    }
    let j = i + l * d;

    // 4. Find the split point
    let delta_node = delta(i, j, n);
    var s = 0;
    t = (l + 1) / 2;
    while t > 0 {
        if delta(i, i + (s + t) * d, n) > delta_node {
            s = s + t;
        }
        t = t / 2;
    }
    let split = i + s * d + min(d, 0);

    // 5. Store links
    let left = select(split, i32(params.num_particles) + split, split == min(i, j));
    let right = select(split + 1, i32(params.num_particles) + split + 1, split + 1 == max(i, j));
    
    // For simplicity in this demo, we link internal node i to its children
    // Nodes [0, N-2] are internal, [N-1, 2N-2] are leaves
    nodes[i].children = vec4<i32>(left, right, -1, -1);
}

@compute @workgroup_size(256)
fn compute_mass(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = i32(global_id.x);
    let n = i32(params.num_particles);
    if i >= n { return; }

    // Start from leaf nodes (particles)
    // For simplicity in this demo, we'll use a multi-pass approach dispatched from Rust
    // or a simple iterative loop if possible.
    // However, a true bottom-up needs synchronization.
    // We'll use the 'parent' links we set in build_tree.
}

@compute @workgroup_size(256)
fn update_gravity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    var p_i = particles[i].position;
    var v_i = particles[i].velocity;
    var acc = vec2<f32>(0.0, 0.0);

    // Iterative stack-less traversal
    var node_idx = 0i;
    // We use a fixed number of steps or a condition to avoid infinite loops
    for (var step = 0; step < 512; step++) {
        let node = nodes[node_idx];
        let diff = node.pos - p_i;
        let dist_sq = dot(diff, diff) + 1.0; // Softening

        let s = node.size;
        // theta condition: s/d < theta  => s^2 < theta^2 * d^2
        if s * s < params.theta * params.theta * dist_sq || node.children[0] == -1 {
            // Leaf or distant enough internal node
            acc += diff * (params.gravity * node.mass / (dist_sq * sqrt(dist_sq)));
            
            // "Next" or escape to sibling
            // In a binary tree with rope pointers, we'd follow the rope.
            // Without ropes, we can store a 'next' index in the node structure.
            node_idx = node.parent; // This is a placeholder, real ropes are better
            break; // Temporary break to avoid infinite loop until ropes are implemented
        } else {
            // Descend to first child
            node_idx = node.children[0];
        }
    }

    v_i += acc * params.dt;
    p_i += v_i * params.dt;

    particles[i].position = p_i;
    particles[i].velocity = v_i;
}
