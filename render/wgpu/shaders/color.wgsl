/// Shader used for drawing solid color fills.

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main_vertex(in: VertexInput) -> VertexOutput {
    let pos = globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    let color = in.color * transforms.mult_color + transforms.add_color;
    let alpha = clamp(color.a, 0.0, 1.0);
    return VertexOutput(pos, vec4<f32>(color.rgb * alpha, alpha));
}

[[stage(fragment)]]
fn main_fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}
