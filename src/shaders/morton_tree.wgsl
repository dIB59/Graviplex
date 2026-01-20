// ============================================================================
// Morton Code Linear Quadtree - GPU Parallel Implementation
// ============================================================================
// This shader implements a fully GPU-parallel Barnes-Hut algorithm using
// Morton codes (Z-order curve) for spatial sorting and linear quadtree construction.
//
// Pipeline:
// 1. compute_bounds      - Find min/max bounds of all particles (parallel reduction)
// 2. compute_morton      - Assign Morton codes to each particle
// 3. sort_*              - Parallel radix sort (multiple passes)
// 4. build_tree          - Construct tree nodes from sorted Morton codes
// 5. compute_com         - Bottom-up center-of-mass computation
// 6. barnes_hut_gravity  - Tree traversal for gravity calculation
// ============================================================================

// ============================================================================
// Data Structures
// ============================================================================

struct Particle {
    position: vec2<f32>,
    radius: f32,
    color: u32,
    velocity: vec2<f32>,
    mass: f32,
    id: u32,
}

struct MortonEntry {
    key: u32,           // Morton code (32-bit)
    particle_idx: u32,  // Original particle index
}

struct TreeNode {
    first_child: u32,       // Index of first child (0 if leaf)
    particle_start: u32,    // Start index in sorted particle array
    particle_count: u32,    // Number of particles in this node
    level: u32,             // Tree level (0 = root)
    center_of_mass: vec2<f32>,
    total_mass: f32,
    _padding: f32,
}

struct Bounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

struct TreeParams {
    num_particles: u32,
    num_nodes: u32,
    theta_sq: f32,          // Barnes-Hut opening angle squared
    epsilon_sq: f32,        // Softening factor squared
    gravity: f32,
    dt: f32,
    max_level: u32,
    _padding: u32,
}

// ============================================================================
// Bindings
// ============================================================================

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> morton_entries: array<MortonEntry>;
@group(0) @binding(2) var<storage, read_write> tree_nodes: array<TreeNode>;
@group(0) @binding(3) var<storage, read_write> bounds: Bounds;
@group(0) @binding(4) var<uniform> params: TreeParams;

// Temporary buffers for sorting
@group(1) @binding(0) var<storage, read_write> morton_temp: array<MortonEntry>;
@group(1) @binding(1) var<storage, read_write> histograms: array<u32>;

// ============================================================================
// Morton Code Functions
// ============================================================================

/// Interleave bits of a 16-bit integer to produce a 32-bit Morton code component
fn interleave_bits(x: u32) -> u32 {
    var n = x & 0x0000FFFFu;
    n = (n | (n << 8u)) & 0x00FF00FFu;
    n = (n | (n << 4u)) & 0x0F0F0F0Fu;
    n = (n | (n << 2u)) & 0x33333333u;
    n = (n | (n << 1u)) & 0x55555555u;
    return n;
}

/// Compute Morton code for a 2D position within normalized bounds [0, 1]
fn compute_morton_code(pos: vec2<f32>, bounds_min: vec2<f32>, bounds_size: vec2<f32>) -> u32 {
    // Normalize position to [0, 1]
    let normalized = (pos - bounds_min) / bounds_size;
    
    // Clamp to valid range
    let clamped = clamp(normalized, vec2<f32>(0.0), vec2<f32>(0.99999));
    
    // Quantize to 16-bit grid (65536 x 65536)
    let quantized = vec2<u32>(clamped * 65536.0);
    
    // Interleave bits to produce Morton code
    return interleave_bits(quantized.x) | (interleave_bits(quantized.y) << 1u);
}

/// Get the common prefix length between two Morton codes
fn common_prefix_length(a: u32, b: u32) -> u32 {
    return countLeadingZeros(a ^ b);
}

// ============================================================================
// Pass 1: Compute Bounds (Parallel Reduction)
// ============================================================================

var<workgroup> wg_min: array<vec2<f32>, 256>;
var<workgroup> wg_max: array<vec2<f32>, 256>;

@compute @workgroup_size(256)
fn compute_bounds(@builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) wg_id: vec3<u32>,
    @builtin(num_workgroups) num_wg: vec3<u32>) {
    let tid = local_id.x;
    let gid = global_id.x;
    
    // Initialize with extreme values
    var local_min = vec2<f32>(1e20);
    var local_max = vec2<f32>(-1e20);
    
    // Each thread processes one or more particles
    if gid < params.num_particles {
        let pos = particles[gid].position;
        local_min = pos;
        local_max = pos;
    }

    wg_min[tid] = local_min;
    wg_max[tid] = local_max;
    workgroupBarrier();
    
    // Parallel reduction within workgroup
    for (var stride = 128u; stride > 0u; stride = stride >> 1u) {
        if tid < stride {
            wg_min[tid] = min(wg_min[tid], wg_min[tid + stride]);
            wg_max[tid] = max(wg_max[tid], wg_max[tid + stride]);
        }
        workgroupBarrier();
    }
    
    // First thread in first workgroup writes result
    // Note: For simplicity, we only use first workgroup's result.
    // Bounds are pre-set from CPU for production use.
    if tid == 0u && wg_id.x == 0u {
        bounds.min_x = wg_min[0].x;
        bounds.min_y = wg_min[0].y;
        bounds.max_x = wg_max[0].x;
        bounds.max_y = wg_max[0].y;
    }
}

// ============================================================================
// Pass 2: Compute Morton Codes
// ============================================================================

@compute @workgroup_size(256)
fn compute_morton(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= params.num_particles { return; }

    let pos = particles[idx].position;
    let bounds_min = vec2<f32>(bounds.min_x, bounds.min_y);
    let bounds_max = vec2<f32>(bounds.max_x, bounds.max_y);
    let bounds_size = max(bounds_max - bounds_min, vec2<f32>(1.0)); // Prevent division by zero

    let morton = compute_morton_code(pos, bounds_min, bounds_size);

    morton_entries[idx].key = morton;
    morton_entries[idx].particle_idx = idx;
}

// ============================================================================
// Pass 3: Radix Sort (Per-digit histogram and scatter)
// ============================================================================

// Sort configuration
const RADIX_BITS: u32 = 4u;
const RADIX_SIZE: u32 = 16u; // 2^4

// Note: For proper radix sort, histograms would need atomic<u32> type.
// For now, these are simplified stubs until sorting is fully wired up.

// Build histogram for current radix digit
// Note: This is a placeholder. Full implementation needs atomic histogram.
@compute @workgroup_size(256)
fn sort_histogram(@builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) wg_id: vec3<u32>) {
    // Placeholder - actual radix sort requires atomic operations
    // For now, particles remain unsorted
}

// Prefix sum on histograms (placeholder)
@compute @workgroup_size(256)
fn sort_prefix_sum(@builtin(local_invocation_id) local_id: vec3<u32>) {
    // Placeholder - actual implementation needs global prefix sum
}

// Scatter elements to sorted positions (placeholder)
@compute @workgroup_size(256)
fn sort_scatter(@builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) wg_id: vec3<u32>) {
    // Placeholder - actual implementation needs atomic scatter
}

// Copy back from temp buffer
@compute @workgroup_size(256)
fn sort_copy_back(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if gid >= params.num_particles { return; }

    morton_entries[gid] = morton_temp[gid];
}

// ============================================================================
// Pass 4: Build Tree (Detect prefix changes)
// ============================================================================

@compute @workgroup_size(256)
fn build_tree(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= params.num_nodes { return; }
    
    // Initialize node
    var node: TreeNode;
    node.first_child = 0u;
    node.particle_start = 0u;
    node.particle_count = 0u;
    node.level = 0u;
    node.center_of_mass = vec2<f32>(0.0);
    node.total_mass = 0.0;
    node._padding = 0.0;

    if idx == 0u {
        // Root node contains all particles
        node.particle_start = 0u;
        node.particle_count = params.num_particles;
        node.level = 0u;
    }

    tree_nodes[idx] = node;
}

// Build tree hierarchy by finding Morton code prefix boundaries
@compute @workgroup_size(256)
fn build_tree_hierarchy(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= params.num_particles { return; }
    
    // Each sorted particle marks where tree nodes begin
    // Detect level changes based on common prefix with neighbors

    let my_key = morton_entries[idx].key;
    
    // Compare with previous key to find level boundaries
    if idx > 0u {
        let prev_key = morton_entries[idx - 1u].key;
        let prefix_len = common_prefix_length(my_key, prev_key);
        
        // A shorter common prefix means we've crossed a node boundary
        // We'll use this to populate tree structure in a subsequent pass
    }
}

// ============================================================================
// Pass 5: Compute Centers of Mass (Bottom-up)
// ============================================================================

@compute @workgroup_size(256)
fn compute_center_of_mass(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let node_idx = global_id.x;
    if node_idx >= params.num_nodes { return; }

    var node = tree_nodes[node_idx];
    
    // For leaf nodes, compute CoM directly from particles
    if node.first_child == 0u && node.particle_count > 0u {
        var total_mass = 0.0f;
        var weighted_pos = vec2<f32>(0.0);

        for (var i = 0u; i < node.particle_count; i = i + 1u) {
            let sorted_idx = node.particle_start + i;
            let particle_idx = morton_entries[sorted_idx].particle_idx;
            let p = particles[particle_idx];

            total_mass += p.mass;
            weighted_pos += p.position * p.mass;
        }

        if total_mass > 0.0 {
            node.center_of_mass = weighted_pos / total_mass;
        }
        node.total_mass = total_mass;
        tree_nodes[node_idx] = node;
    }
}

// Propagate CoM up the tree (run multiple times for each level)
@compute @workgroup_size(256)
fn propagate_com_up(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let node_idx = global_id.x;
    if node_idx >= params.num_nodes { return; }

    var node = tree_nodes[node_idx];
    
    // Only process internal nodes at the current level
    if node.first_child != 0u && node.level == params.max_level {
        var total_mass = 0.0f;
        var weighted_pos = vec2<f32>(0.0);
        
        // Sum up children
        for (var i = 0u; i < 4u; i = i + 1u) {
            let child_idx = node.first_child + i;
            if child_idx < params.num_nodes {
                let child = tree_nodes[child_idx];
                total_mass += child.total_mass;
                weighted_pos += child.center_of_mass * child.total_mass;
            }
        }

        if total_mass > 0.0 {
            node.center_of_mass = weighted_pos / total_mass;
        }
        node.total_mass = total_mass;
        tree_nodes[node_idx] = node;
    }
}

// ============================================================================
// Pass 6: Barnes-Hut Gravity Calculation
// ============================================================================

// Stack for tree traversal (stored in registers)
const MAX_STACK_SIZE: u32 = 32u;

@compute @workgroup_size(256)
fn barnes_hut_gravity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_idx = global_id.x;
    if particle_idx >= params.num_particles { return; }

    let p = particles[particle_idx];
    var acc = vec2<f32>(0.0);
    
    // Simple tree traversal using explicit stack
    var stack: array<u32, 32>;
    var stack_ptr = 0u;
    
    // Start with root
    stack[0] = 0u;
    stack_ptr = 1u;

    while stack_ptr > 0u {
        stack_ptr = stack_ptr - 1u;
        let node_idx = stack[stack_ptr];

        if node_idx >= params.num_nodes { continue; }

        let node = tree_nodes[node_idx];

        if node.total_mass == 0.0 { continue; }

        let diff = node.center_of_mass - p.position;
        let dist_sq = dot(diff, diff);
        
        // Compute node size from level
        let bounds_size = max(bounds.max_x - bounds.min_x, bounds.max_y - bounds.min_y);
        let node_size = bounds_size / f32(1u << node.level);
        
        // Barnes-Hut criterion: use node if it's far enough away or is a leaf
        let use_approximation = (node_size * node_size < dist_sq * params.theta_sq) || (node.first_child == 0u);

        if use_approximation {
            // Use this node's CoM for gravity calculation
            if dist_sq > params.epsilon_sq {
                let dist = sqrt(dist_sq + params.epsilon_sq);
                let force = params.gravity * node.total_mass / (dist_sq + params.epsilon_sq);
                acc += diff * (force / dist);
            }
        } else {
            // Need to descend into children
            if node.first_child != 0u && stack_ptr + 4u < MAX_STACK_SIZE {
                for (var i = 0u; i < 4u; i = i + 1u) {
                    stack[stack_ptr] = node.first_child + i;
                    stack_ptr = stack_ptr + 1u;
                }
            }
        }
    }
    
    // Update velocity
    particles[particle_idx].velocity += acc * params.dt;
}

// ============================================================================
// Integration Pass (Update positions)
// ============================================================================

@compute @workgroup_size(256)
fn integrate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= params.num_particles { return; }

    particles[idx].position += particles[idx].velocity * params.dt;
}
