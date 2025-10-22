struct Uniforms {
    screen_size: vec2<f32>,
}

@group(0) @binding(0) var t_font: texture_2d<f32>;
@group(0) @binding(1) var s_font: sampler;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VOut {
    var out: VOut;
    // Transform from pixel coords to clip space
    let clip_pos = pos * vec2(2.0 / uniforms.screen_size.x, -2.0 / uniforms.screen_size.y) + vec2(-1.0, 1.0);
    out.pos = vec4(clip_pos, 0.0, 1.0);
    out.uv = uv;
    out.color = color; // Already normalized by Unorm8x4
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let a = textureSample(t_font, s_font, in.uv).r;   // 0 or 1
    if a < 0.9 { discard; }                         // punch holes
    return in.color;                                  // solid text colour
}
