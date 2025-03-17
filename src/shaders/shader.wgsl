struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) screen_pos: vec2<f32>,
    @location(2) center: vec2<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexPayload {

    var out: VertexPayload;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.color = vec3<f32>(1.0, 1.0, 1.0);

    let aspect = f32(vertex.position.x) / f32(vertex.position.y);

    out.screen_pos = out.position.xy * 0.5 + vec2<f32>(0.5, 0.5);

    let center = vec2<f32>(
        (out.screen_pos.x + 1) / 3.0,
        (out.screen_pos.y + 1) / 3.0
    );

    out.center = center;
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let dist = distance(in.screen_pos, in.center);
    if dist > 0.1 {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}