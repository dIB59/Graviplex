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
    interaction_pos: vec2<f32>,
    interaction_radius: f32,
    interaction_strength: f32,
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
