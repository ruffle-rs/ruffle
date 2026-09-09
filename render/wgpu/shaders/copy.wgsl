/// Shader used for drawing bitmap fills.

// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;

@vertex
fn main_vertex(in: common__VertexInputUv) -> VertexOutput {
    let pos = vec4<f32>((in.position.x * 2.0) - 1.0, -(in.position.y * 2.0) + 1.0, 0.0, 1.0);
    return VertexOutput(pos, in.uv.xy / in.uv.z);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.uv);
}
