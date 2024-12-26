struct VertexInput {
    @builtin(position) position: vec4f,
};

struct VertexOutput {
   @builtin(position) position: vec4f,
   @location(0) color : vec3f
}

@vertex
fn vs_main(vertex_input: VertexInput) -> VertexOutPut {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x , y , 1.0, 1.0);

}

//color of triangle
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
