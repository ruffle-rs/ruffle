@group(0) @binding(0) var<uniform> color: vec4<f32>;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let x = f32(i32(in_vertex_index / 2u) * 2 - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return VertexOutput(color, vec4<f32>(x, y, 0.0, 1.0));
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}