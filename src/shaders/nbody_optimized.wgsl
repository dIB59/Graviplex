/// NOTE:
/// This only works for 32 bits
/// the Max Quadtree level is 15 including the root


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
    seed: u32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;

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

    p_i += v_i * params.dt;

    particles[i].position = p_i;
    particles[i].velocity = v_i;
}

/// De-interleave a 32-bit unsigned integer (extracts every other bit)
fn deinterleave32(x: u32) -> u32 {
    var y: u32 = x;
    // Step 1: spread bits apart by 1, mask every other bit
    y = (y | (y >> 1u)) & 0x33333333u;
    
    // Step 2: spread bits apart by 2, mask 2-bit groups
    y = (y | (y >> 2u)) & 0x0F0F0F0Fu;
    
    // Step 3: spread bits apart by 4, mask 4-bit groups
    y = (y | (y >> 4u)) & 0x00FF00FFu;
    
    // Step 4: spread bits apart by 8, mask 8-bit groups
    y = (y | (y >> 8u)) & 0x0000FFFFu;
    
    // Step 5: return lower 16 bits, which is the de-interleaved number
    return y & 0x0000FFFFu;
}

struct MortonKey {
    level: u32,
    pos: vec2<u32>,
}

/// Retrieve column major position and level from a 32-bit word
fn decode_morton(key: u32) -> MortonKey {
    var res: MortonKey;
    var out_pos: vec2<u32>;
    var out_level = key & 0xFu;
    out_pos.x = deinterleave32((key >> 4u) & 0x55555555u);
    out_pos.y = deinterleave32((key >> 5u) & 0x55555555u);
    res.level = out_level;
    res.pos = out_pos;
    return res;
}

/// Generate children nodes from a quadtree encoded in a 32-bit word
fn generate_children(key_in: u32) -> array<u32, 4> {
    var children: array<u32, 4>;

    var k = key_in + 1u;
    k = (k & 0xFu) | ((k & ~0xFu) << 2u);

    children[0] = k;
    children[1] = k | 0x10u;
    children[2] = k | 0x20u;
    children[3] = k | 0x30u;

    return children;
}

/// Generate parent node from a quadtree encoded in a 32-bit word
fn generate_parent(key_in: u32) -> u32 {
    var k = key_in - 1u;

    return (k & 0xFu) | ((key_in >> 2u) & 0x3FFFFFF0u);
}

fn is_upper_left(key_in: u32) -> bool {
    return (key_in & 0x30u) == 0u;
}

/// P is in [0,1)
/// Size is in [0,1]
struct CellResult {
    p: vec2<f32>,
    size: f32,
};

// Retrieve normalized coordinates and size of the cell
fn get_cell(key: u32) -> CellResult {
    var pos: vec2<u32>;
    var level: u32;
    var result: CellResult;

    // Assumes lt_decode_2_15 is defined elsewhere to update 'level' and 'pos'
    lt_decode_2_15(key, &level, &pos);

    result.size = 1.0 / f32(1u << level); // in [0,1]
    result.p = vec2<f32>(pos) * result.size; // in [0,1)

    return result;
}
