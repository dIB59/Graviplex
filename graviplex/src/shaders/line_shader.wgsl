// Line shader for rendering quadtree cell boundaries

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
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) start: vec2<f32>,
    @location(2) end: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_space: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Interpolate between start and end based on vertex position (0 or 1)
    let world_pos = mix(instance.start, instance.end, vertex.position.x);
    
    // Apply camera transformation (same as circle_shader)
    let camera_space = (world_pos - view.position) * view.scale;
    let ndc = camera_space / (view.screen_size * 0.5);

    out.clip_space = vec4<f32>(ndc, 0.0, 1.0);
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
