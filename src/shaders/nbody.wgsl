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
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;

var<workgroup> tile_positions: array<vec2<f32>, 256>;
var<workgroup> tile_masses: array<f32, 256>;

@compute @workgroup_size(256)
fn update_gravity(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let i = global_id.x;
    if i >= params.num_particles { return; }

    var p_i = particles[i].position;
    var v_i = particles[i].velocity;
    var acc = vec2<f32>(0.0, 0.0);

    let num_tiles = (params.num_particles + 255u) / 256u;

    for (var tile = 0u; tile < num_tiles; tile = tile + 1u) {
        let load_idx = tile * 256u + local_id.x;
        if load_idx < params.num_particles {
            tile_positions[local_id.x] = particles[load_idx].position;
            tile_masses[local_id.x] = particles[load_idx].mass;
        } else {
            tile_positions[local_id.x] = vec2<f32>(0.0, 0.0);
            tile_masses[local_id.x] = 0.0;
        }

        workgroupBarrier();

        for (var j = 0u; j < 256u; j = j + 1u) {
            let p_j = tile_positions[j];
            let m_j = tile_masses[j];

            let diff = p_j - p_i;
            let dist_sq = dot(diff, diff) + 1.0;
            let inv_dist = inverseSqrt(dist_sq);
            let inv_dist3 = inv_dist * inv_dist * inv_dist;

            acc += diff * (m_j * params.gravity * inv_dist3);
        }

        workgroupBarrier();
    }

    v_i += acc * params.dt;
    p_i += v_i * params.dt;

    particles[i].position = p_i;
    particles[i].velocity = v_i;
}
