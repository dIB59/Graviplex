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
    pass_index: u32,
    num_nodes: u32,
    interaction_pos: vec2<f32>,
    interaction_radius: f32,
    interaction_strength: f32,
    _extended_pad: array<vec4<u32>, 13>,
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
@group(0) @binding(6) var<storage, read_write> atomic_counters: array<atomic<u32>>;

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
    let vel_mag = 0.001 * sqrt(800.0 / (r + 1.0));
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

    // Domain for 1M particles: [-10000, 10000]
    let p = particles[i].position;
    let x = u32(clamp((p.x + 10000.0) / 20000.0, 0.0, 1.0) * 65535.0);
    let y = u32(clamp((p.y + 10000.0) / 20000.0, 0.0, 1.0) * 65535.0);

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

// --- Bitonic Sort Pass ---

@compute @workgroup_size(256)
fn bitonic_sort_step(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    let p = params.pass_index; // Current stage size (power of 2)
    let q = params.num_nodes;  // Current step size (power of 2)

    let j = i ^ q;

    if j > i {
        let key_i = sort_data[i].key;
        let key_j = sort_data[j].key;

        let direction = (i & p) == 0u;
        if (key_i > key_j) == direction {
            let temp = sort_data[i];
            sort_data[i] = sort_data[j];
            sort_data[j] = temp;
        }
    }
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
    var iter = 0;
    while delta(i, i + l_max * d, n) > delta_min && iter < 32 {
        l_max = l_max * 2;
        iter++;
    }

    // 3. Find the other end using binary search
    var l = 0;
    var t = l_max / 2;
    iter = 0;
    while t > 0 && iter < 32 {
        if delta(i, i + (l + t) * d, n) > delta_min {
            l = l + t;
        }
        t = t / 2;
        iter++;
    }
    let j = i + l * d;

    // 4. Find the split point
    let delta_node = delta(i, j, n);
    var s = 0;
    t = (l + 1) / 2;
    iter = 0;
    while t > 0 && iter < 32 {
        if delta(i, i + (s + t) * d, n) > delta_node {
            s = s + t;
        }
        t = t / 2;
        iter++;
    }
    let split = i + s * d + min(d, 0);

    // 5. Store links
    let leaf_offset = n - 1;
    
    // Left child
    if split == min(i, j) {
        let leaf_idx = leaf_offset + split;
        nodes[i].children[0] = leaf_idx;
        nodes[leaf_idx].parent = i;
        // Init leaf from sorted particle
        let p_idx = sort_data[split].value;
        nodes[leaf_idx].pos = particles[p_idx].position;
        nodes[leaf_idx].mass = particles[p_idx].mass;
        nodes[leaf_idx].size = particles[p_idx].radius;
        nodes[leaf_idx].next = i32(p_idx); // Store original particle index in 'next' for self-collision check
        nodes[leaf_idx].children = vec4<i32>(-1, -1, -1, -1);
    } else {
        let child_idx = split;
        nodes[i].children[0] = child_idx;
        nodes[child_idx].parent = i;
    }

    // Right child
    if split + 1 == max(i, j) {
        let leaf_idx = leaf_offset + split + 1;
        nodes[i].children[1] = leaf_idx;
        nodes[leaf_idx].parent = i;
        // Init leaf from sorted particle
        let p_idx = sort_data[split + 1].value;
        nodes[leaf_idx].pos = particles[p_idx].position;
        nodes[leaf_idx].mass = particles[p_idx].mass;
        nodes[leaf_idx].size = particles[p_idx].radius;
        nodes[leaf_idx].next = i32(p_idx); // Store original particle index in 'next' for self-collision check
        nodes[leaf_idx].children = vec4<i32>(-1, -1, -1, -1);
    } else {
        let child_idx = split + 1;
        nodes[i].children[1] = child_idx;
        nodes[child_idx].parent = i;
    }
}

@compute @workgroup_size(256)
fn compute_mass(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = i32(global_id.x);
    let n = i32(params.num_particles);
    if i >= n { return; }

    // Start from leaf node
    var current = (n - 1) + i;
    var iter = 0;

    loop {
        if iter > 32 { break; }
        iter++;

        let parent = nodes[current].parent;
        if parent == -1 { break; }

        let res = atomicAdd(&atomic_counters[parent], 1u);
        if res == 0u {
            // First child to arrive, we are done
            break;
        }
        
        // Second child arrived, process the parent
        let left = nodes[parent].children[0];
        let right = nodes[parent].children[1];

        if left == -1 || right == -1 { break; }

        let m_l = nodes[left].mass;
        let m_r = nodes[right].mass;
        let m_tot = m_l + m_r;

        nodes[parent].mass = m_tot;
        nodes[parent].pos = (nodes[left].pos * m_l + nodes[right].pos * m_r) / max(m_tot, 0.0001);
        
        // Bounding box size (simplified as distance from center of mass)
        let d_l = distance(nodes[parent].pos, nodes[left].pos) + nodes[left].size;
        let d_r = distance(nodes[parent].pos, nodes[right].pos) + nodes[right].size;
        
        // Add a 5% conservative buffer to node size to account for motion during gravity pass
        nodes[parent].size = max(d_l, d_r) * 1.05;

        current = parent;
    }
}

@compute @workgroup_size(256)
fn resolve_collisions(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    var p_i = particles[i].position;
    var v_i = particles[i].velocity;
    let r_i = particles[i].radius;

    for (var j: u32 = 0u; j < params.num_particles; j = j + 1u) {
        if i == j { continue; }

        let p_j = particles[j].position;
        let r_j = particles[j].radius;

        let diff = p_j - p_i;
        let dist_sq = dot(diff, diff);
        let min_dist = r_i + r_j;

        if dist_sq < min_dist * min_dist && dist_sq > 0.0001 {
            let dist = sqrt(dist_sq);
            let normal = diff / dist;
            let penetration = min_dist - dist;
            
            // Simple penetration correction
            p_i -= normal * penetration * 0.5;
            
            // Collision response
            let relative_velocity = particles[j].velocity - v_i;
            let velocity_along_normal = dot(relative_velocity, normal);

            if velocity_along_normal < 0.0 {
                let restitution = 0.5;
                let impulse = (1.0 + restitution) * velocity_along_normal;
                v_i += normal * (impulse * 0.5);
            }
        }
    }

    particles[i].position = p_i;
    particles[i].velocity = v_i;
}

@compute @workgroup_size(256)
fn update_gravity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    var p_i = particles[i].position;
    var v_i = particles[i].velocity;
    var acc = vec2<f32>(0.0, 0.0);

    for (var j: u32 = 0u; j < params.num_particles; j = j + 1u) {
        if i == j { continue; }

        let p_j = particles[j].position;
        let m_j = particles[j].mass;

        let diff = p_j - p_i;
        let dist_sq = dot(diff, diff) + 1.0; // Softening factor

        acc += diff * (params.gravity * m_j / (dist_sq * sqrt(dist_sq)));
    }

    v_i += acc * params.dt;

    // GPU Interaction Force
    let dist_to_mouse = distance(p_i, params.interaction_pos);
    if dist_to_mouse < params.interaction_radius {
        let pull_dir = normalize(params.interaction_pos - p_i);
        let pull_strength = (1.0 - dist_to_mouse / params.interaction_radius) * params.interaction_strength;
        v_i += pull_dir * (pull_strength * params.dt);
    }

    p_i += v_i * params.dt;

    particles[i].position = p_i;
    particles[i].velocity = v_i;
}
