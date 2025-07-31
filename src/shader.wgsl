var<private> VERTICES: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.7321,-1.0),
    vec2<f32>( 1.7321,-1.0), // sqrt(3) ≈ 1.7321
    vec2<f32>( 0.0   , 2.0),
);

// Vertex shader
struct View {
    position: vec2<f32>,
    scale: f32,
    xy: u32,
};

@group(0)
@binding(0)
var<uniform> view: View;

struct VertexInput {
    @location(0) vertex_pos: vec2<f32>,         // Vertex.pos
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) radius: f32,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_space: vec4<f32>,
    @location(0) local_space: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) pixel_size: f32,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = vertex.vertex_pos * instance.radius + instance.position;
    let scale = view.scale;
    let screen_pos = (world_pos - view.position) * scale;

    out.clip_space = vec4<f32>(screen_pos, 0.0, 1.0);
    out.local_space = vertex.vertex_pos; // [-1..1] triangle space
    out.color = instance.color;
    out.pixel_size = 1.0 / scale; // optional
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = 1.0 - smoothstep(1.0 - 3.0 * in.pixel_size, 1.0, length(in.local_space));
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
