// Sprite shader for textured quad rendering with texture atlas support.

struct View {
    position: vec2<f32>,
    scale: f32,
    zoom_speed: f32,
    screen_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;

@group(1) @binding(1)
var s_atlas: sampler;

struct VertexInput {
    @location(0) vertex_pos: vec2<f32>,  // Quad corner: (-0.5,-0.5) to (0.5,0.5)
};

struct InstanceInput {
    @location(1) position: vec2<f32>,    // World position (center)
    @location(2) size: vec2<f32>,        // Size in world units
    @location(3) uv_rect: vec4<f32>,     // UV coords (u_min, v_min, u_max, v_max)
    @location(4) tint: vec4<f32>,        // Color tint
    @location(5) rotation: f32,          // Rotation in radians
};

struct VertexOutput {
    @builtin(position) clip_space: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply rotation to local vertex position
    let cos_r = cos(instance.rotation);
    let sin_r = sin(instance.rotation);
    let rotated = vec2<f32>(
        vertex.vertex_pos.x * cos_r - vertex.vertex_pos.y * sin_r,
        vertex.vertex_pos.x * sin_r + vertex.vertex_pos.y * cos_r
    );
    
    // Scale by sprite size and translate to world position
    let world_pos = rotated * instance.size + instance.position;
    
    // Apply camera transformation (same as circle_shader)
    let camera_space = (world_pos - view.position) * view.scale;
    let ndc = camera_space / (view.screen_size * 0.5);
    
    out.clip_space = vec4<f32>(ndc, 0.0, 1.0);
    
    // Interpolate UV coordinates based on vertex position
    // vertex_pos goes from (-0.5,-0.5) to (0.5,0.5), normalize to (0,0)-(1,1)
    let uv_lerp = vertex.vertex_pos + 0.5;
    out.uv = vec2<f32>(
        mix(instance.uv_rect.x, instance.uv_rect.z, uv_lerp.x),
        mix(instance.uv_rect.y, instance.uv_rect.w, uv_lerp.y)
    );
    
    out.tint = instance.tint;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_atlas, s_atlas, in.uv);
    
    // Multiply texture color by tint
    let final_color = tex_color * in.tint;
    
    // Discard fully transparent pixels
    if final_color.a < 0.001 {
        discard;
    }
    
    return final_color;
}
