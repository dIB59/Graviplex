struct View {
    position: vec2<f32>,
    scale: f32,
    zoom_speed: f32,
    screen_size: vec2<f32>,
}

@group(0)
@binding(0)
var<uniform> view: View;

struct VertexInput {
    @location(0) vertex_pos: vec2<f32>,
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
    @location(3) radius: f32
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform to world space
    let world_pos = vertex.vertex_pos * instance.radius + instance.position;
    
    // Apply camera transformation
    let camera_space = (world_pos - view.position) * view.scale;

    let ndc = camera_space / (view.screen_size * 0.5);

    out.clip_space = vec4<f32>(ndc, 0.0, 1.0);
    out.local_space = vertex.vertex_pos;
    out.color = instance.color;
    out.pixel_size = 1.0 / view.scale;
    out.radius = instance.radius;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = length(in.local_space);
    
    // Discard fragments outside the circle
    if distance > 1.0 {
        discard;
    }
    
    // Smooth antialiasing at the circle edge
    let alpha = 1.0 - smoothstep(1.0 - 3.0 * in.pixel_size, 1.0, distance);

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
