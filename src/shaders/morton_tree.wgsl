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
// Pass 3: Bitonic Sort
// ============================================================================
// Bitonic sort is ideal for GPUs:
// - Completely data-parallel (no atomics needed)
// - Fixed comparison pattern (no data-dependent branches)
// - O(N log²N) comparisons, fully parallelizable
//
// The sort is performed in two phases:
// 1. Local sort: Each workgroup sorts 256 elements using shared memory
// 2. Global merge: Multiple passes merge sorted blocks into larger sorted blocks
// ============================================================================

const WORKGROUP_SIZE: u32 = 256u;

// Shared memory for local bitonic sort
var<workgroup> local_morton: array<MortonEntry, 256>;

/// Compare and swap two Morton entries based on sort direction
fn compare_and_swap(a: ptr<function, MortonEntry>, b: ptr<function, MortonEntry>, ascending: bool) {
    let should_swap = select((*a).key < (*b).key, (*a).key > (*b).key, ascending);
    if should_swap {
        let temp = *a;
        *a = *b;
        *b = temp;
    }
}

// ============================================================================
// Local Bitonic Sort (within workgroup using shared memory)
// ============================================================================
@compute @workgroup_size(256)
fn bitonic_sort_local(@builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) wg_id: vec3<u32>) {
    let tid = local_id.x;
    let gid = global_id.x;
    
    // Load into shared memory
    if gid < params.num_particles {
        local_morton[tid] = morton_entries[gid];
    } else {
        // Pad with max values for out-of-bounds
        local_morton[tid] = MortonEntry(0xFFFFFFFFu, 0xFFFFFFFFu);
    }
    workgroupBarrier();
    
    // Bitonic sort within workgroup (256 elements = 8 stages)
    // Stage k creates bitonic sequences of length 2^(k+1)
    for (var k = 0u; k < 8u; k = k + 1u) {
        let block_size = 1u << (k + 1u);
        
        // Within each stage, we have multiple steps
        for (var j = k + 1u; j > 0u; j = j - 1u) {
            let step_size = 1u << j;
            let half_step = step_size >> 1u;
            
            // Determine partner for this thread
            let pos_in_block = tid & (step_size - 1u);
            let is_first_half = pos_in_block < half_step;

            if is_first_half {
                let partner = tid + half_step;
                if partner < WORKGROUP_SIZE {
                    // Determine sort direction based on position in larger block
                    let block_idx = tid / block_size;
                    let ascending = (block_idx & 1u) == 0u;

                    var a = local_morton[tid];
                    var b = local_morton[partner];

                    let should_swap = select(a.key < b.key, a.key > b.key, ascending);
                    if should_swap {
                        local_morton[tid] = b;
                        local_morton[partner] = a;
                    }
                }
            }
            workgroupBarrier();
        }
    }
    
    // Write back to global memory
    if gid < params.num_particles {
        morton_entries[gid] = local_morton[tid];
    }
}

// ============================================================================
// Global Bitonic Merge (merges sorted blocks across workgroups)
// Uses histograms buffer to pass stage/step parameters:
// histograms[0] = block_size (2^k for stage k)
// histograms[1] = step_size (2^j for step j within stage)
// ============================================================================
@compute @workgroup_size(256)
fn bitonic_merge_global(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if gid >= params.num_particles { return; }
    
    // Read sort parameters from histograms buffer
    let block_size = histograms[0];
    let step_size = histograms[1];
    let half_step = step_size >> 1u;
    
    // Determine position within step
    let pos_in_step = gid & (step_size - 1u);
    let is_first_half = pos_in_step < half_step;

    if !is_first_half { return; } // Only first half of each pair does the compare-swap

    let partner = gid + half_step;
    if partner >= params.num_particles { return; }
    
    // Determine sort direction based on position in block
    let block_idx = gid / block_size;
    let ascending = (block_idx & 1u) == 0u;

    let a = morton_entries[gid];
    let b = morton_entries[partner];

    let should_swap = select(a.key < b.key, a.key > b.key, ascending);
    if should_swap {
        morton_entries[gid] = b;
        morton_entries[partner] = a;
    }
}

// ============================================================================
// Final Ascending Sort Pass
// After bitonic sort, we have alternating ascending/descending blocks.
// This pass ensures everything is sorted ascending.
// histograms[1] = step_size for this pass
// ============================================================================
@compute @workgroup_size(256)
fn bitonic_final_merge(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if gid >= params.num_particles { return; }

    let step_size = histograms[1];
    let half_step = step_size >> 1u;

    let pos_in_step = gid & (step_size - 1u);
    let is_first_half = pos_in_step < half_step;

    if !is_first_half { return; }

    let partner = gid + half_step;
    if partner >= params.num_particles { return; }

    let a = morton_entries[gid];
    let b = morton_entries[partner];
    
    // Always sort ascending in final pass
    if a.key > b.key {
        morton_entries[gid] = b;
        morton_entries[partner] = a;
    }
}

// ============================================================================
// Pass 4: Build Tree from Sorted Morton Codes
// ============================================================================
// After sorting, particles are spatially ordered by Morton code.
// We build a fixed-depth quadtree where:
// - Level 0: 1 root node covering all particles
// - Level 1: 4 nodes (each covers 1/4 of space)
// - Level 2: 16 nodes (each covers 1/16 of space)
// - etc.
//
// Node assignment: A particle at sorted position p belongs to the leaf node
// whose Morton prefix matches the particle's Morton code prefix at that level.
// ============================================================================

/// Get the quadrant (0-3) of a Morton code at a given level
/// Level 0 = root, Level 1 = first subdivision, etc.
fn get_quadrant_at_level(morton_code: u32, level: u32) -> u32 {
    // Morton codes interleave x,y bits: yxyx...
    // At level L, we look at bits (31 - 2*L) and (30 - 2*L)
    // Level 1: bits 30,31 (top 2 bits)
    // Level 2: bits 28,29
    // etc.
    if level == 0u { return 0u; }
    let shift = 32u - (level * 2u);
    return (morton_code >> shift) & 3u;
}

/// Get the node index for a particle based on its Morton code
/// Uses complete quadtree indexing: children of node i are at 4i+1, 4i+2, 4i+3, 4i+4
fn get_leaf_node_for_morton(morton_code: u32, max_level: u32) -> u32 {
    var node_idx = 0u; // Start at root
    for (var level = 1u; level <= max_level; level = level + 1u) {
        let quadrant = get_quadrant_at_level(morton_code, level);
        node_idx = 4u * node_idx + 1u + quadrant;
    }
    return node_idx;
}

// Initialize tree structure (run once before particle assignment)
@compute @workgroup_size(256)
fn build_tree(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let node_idx = global_id.x;
    if node_idx >= params.num_nodes { return; }
    
    // Calculate level from node index using complete quadtree formula
    // Level 0: node 0
    // Level 1: nodes 1-4
    // Level 2: nodes 5-20
    // etc.
    // Formula: level = floor(log4((3*idx + 1)))

    var level = 0u;
    var level_start = 0u;
    var level_size = 1u;
    
    // Find which level this node belongs to
    while level_start + level_size <= node_idx {
        level_start = level_start + level_size;
        level_size = level_size * 4u;
        level = level + 1u;
    }

    var node: TreeNode;
    node.level = level;
    node.center_of_mass = vec2<f32>(0.0);
    node.total_mass = 0.0;
    node._padding = 0.0;
    
    // Set up children pointers for non-leaf nodes
    if level < params.max_level {
        // Children are at 4*idx + 1, 4*idx + 2, 4*idx + 3, 4*idx + 4
        node.first_child = 4u * node_idx + 1u;
    } else {
        // Leaf node
        node.first_child = 0u;
    }
    
    // Particle assignment will be done in the next pass
    node.particle_start = 0u;
    node.particle_count = 0u;

    tree_nodes[node_idx] = node;
}

// Assign particles to leaf nodes based on sorted Morton codes
// This uses a boundary detection approach:
// - First particle in each node sets particle_start
// - We separately count particles after synchronization
@compute @workgroup_size(256)
fn assign_particles_to_nodes(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_sorted_idx = global_id.x;
    if particle_sorted_idx >= params.num_particles { return; }

    let morton_code = morton_entries[particle_sorted_idx].key;
    let leaf_node = get_leaf_node_for_morton(morton_code, params.max_level);

    if leaf_node >= params.num_nodes { return; }
    
    // Detect if this is the first particle in its node
    var is_first_in_node = true;
    if particle_sorted_idx > 0u {
        let prev_morton = morton_entries[particle_sorted_idx - 1u].key;
        let prev_node = get_leaf_node_for_morton(prev_morton, params.max_level);
        is_first_in_node = (leaf_node != prev_node);
    }
    
    // First particle in node sets particle_start
    if is_first_in_node {
        tree_nodes[leaf_node].particle_start = particle_sorted_idx;
    }
}

// Second pass: count particles in each leaf node
// Called AFTER assign_particles_to_nodes to avoid race
@compute @workgroup_size(256)
fn count_particles_in_nodes(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_sorted_idx = global_id.x;
    if particle_sorted_idx >= params.num_particles { return; }

    let morton_code = morton_entries[particle_sorted_idx].key;
    let leaf_node = get_leaf_node_for_morton(morton_code, params.max_level);

    if leaf_node >= params.num_nodes { return; }
    
    // Detect if this is the last particle in its node
    var is_last_in_node = true;
    if particle_sorted_idx + 1u < params.num_particles {
        let next_morton = morton_entries[particle_sorted_idx + 1u].key;
        let next_node = get_leaf_node_for_morton(next_morton, params.max_level);
        is_last_in_node = (leaf_node != next_node);
    }
    
    // Last particle in node sets particle_count
    if is_last_in_node {
        let start = tree_nodes[leaf_node].particle_start;
        tree_nodes[leaf_node].particle_count = particle_sorted_idx - start + 1u;
    }
}

// Propagate particle counts up the tree (run level by level, bottom-up)
// This computes particle_start and particle_count for internal nodes
@compute @workgroup_size(256)
fn propagate_particle_counts(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let node_idx = global_id.x;
    if node_idx >= params.num_nodes { return; }

    let node = tree_nodes[node_idx];
    
    // Only process nodes at the current level (set via params.max_level)
    if node.level != params.max_level { return; }
    
    // Skip if this is a leaf node or has no children
    if node.first_child == 0u { return; }
    
    // Aggregate from children
    var min_start = 0xFFFFFFFFu;
    var max_end = 0u;
    var total_count = 0u;

    for (var i = 0u; i < 4u; i = i + 1u) {
        let child_idx = node.first_child + i;
        if child_idx < params.num_nodes {
            let child = tree_nodes[child_idx];
            if child.particle_count > 0u {
                min_start = min(min_start, child.particle_start);
                max_end = max(max_end, child.particle_start + child.particle_count);
                total_count = total_count + child.particle_count;
            }
        }
    }

    if total_count > 0u {
        tree_nodes[node_idx].particle_start = min_start;
        tree_nodes[node_idx].particle_count = total_count;
    }
}

// ============================================================================
// Pass 5: Compute Centers of Mass (Bottom-up)
// Called once per level, from max_level down to 0
// params.max_level indicates which level we're currently processing
// ============================================================================

@compute @workgroup_size(256)
fn compute_center_of_mass(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let node_idx = global_id.x;
    if node_idx >= params.num_nodes { return; }

    var node = tree_nodes[node_idx];
    
    // Only process nodes at the current level
    if node.level != params.max_level { return; }
    
    // Check if this is a leaf node (no children)
    if node.first_child == 0u {
        // Leaf node: compute CoM directly from particles
        if node.particle_count > 0u {
            var total_mass = 0.0f;
            var weighted_pos = vec2<f32>(0.0);

            // Limit iterations to prevent infinite loops
            let max_particles = min(node.particle_count, 1024u);
            for (var i = 0u; i < max_particles; i = i + 1u) {
                let sorted_idx = node.particle_start + i;
                if sorted_idx >= params.num_particles { break; }

                let particle_idx = morton_entries[sorted_idx].particle_idx;
                if particle_idx >= params.num_particles { continue; }

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
    } else {
        // Internal node: aggregate from children
        var total_mass = 0.0f;
        var weighted_pos = vec2<f32>(0.0);
        
        // Sum up all 4 children
        for (var i = 0u; i < 4u; i = i + 1u) {
            let child_idx = node.first_child + i;
            if child_idx < params.num_nodes {
                let child = tree_nodes[child_idx];
                if child.total_mass > 0.0 {
                    total_mass += child.total_mass;
                    weighted_pos += child.center_of_mass * child.total_mass;
                }
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

// ============================================================================
// Tree-Based Collision Detection
// ============================================================================
// Exploits Morton code spatial locality: particles with similar Morton codes
// are spatially close. We check collisions only within a sliding window
// of the sorted array, achieving O(N × W) complexity where W = window size.
// ============================================================================

const COLLISION_WINDOW_SIZE: u32 = 64u;

@compute @workgroup_size(256)
fn tree_collisions(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let sorted_idx = global_id.x;
    if sorted_idx >= params.num_particles { return; }

    // Get the particle at this sorted position
    let my_entry = morton_entries[sorted_idx];
    let i = my_entry.particle_idx;
    if i >= params.num_particles { return; }

    var p_i = particles[i].position;
    var v_i = particles[i].velocity;
    let r_i = particles[i].radius;
    let m_i = particles[i].mass;

    // Check collisions within sliding window (spatial locality from Morton sort)
    // Particles with nearby sorted indices have similar Morton codes = nearby in space
    let start = select(0u, sorted_idx - COLLISION_WINDOW_SIZE, sorted_idx > COLLISION_WINDOW_SIZE);
    let end = min(sorted_idx + COLLISION_WINDOW_SIZE + 1u, params.num_particles);

    for (var k = start; k < end; k = k + 1u) {
        if k == sorted_idx { continue; }

        let other_entry = morton_entries[k];
        let j = other_entry.particle_idx;
        if j >= params.num_particles { continue; }

        let p_j = particles[j].position;
        let r_j = particles[j].radius;
        let m_j = particles[j].mass;

        let diff = p_j - p_i;
        let dist_sq = dot(diff, diff);
        let min_dist = r_i + r_j;

        // Collision detected
        if dist_sq < min_dist * min_dist && dist_sq > 0.0001 {
            let dist = sqrt(dist_sq);
            let normal = diff / dist;
            let penetration = min_dist - dist;

            // Penetration correction (push apart based on mass ratio)
            let total_mass = m_i + m_j;
            let mass_ratio_i = m_j / total_mass;  // Heavier particles move less
            p_i -= normal * penetration * mass_ratio_i;

            // Collision response (elastic collision)
            let v_j = particles[j].velocity;
            let relative_velocity = v_j - v_i;
            let velocity_along_normal = dot(relative_velocity, normal);

            // Only respond if particles are approaching
            if velocity_along_normal < 0.0 {
                let restitution = 0.5;
                let impulse = (1.0 + restitution) * velocity_along_normal;
                v_i += normal * (impulse * mass_ratio_i);
            }
        }
    }

    // Write back updated position and velocity
    particles[i].position = p_i;
    particles[i].velocity = v_i;
}
