/// Shader used for drawing bitmap fills.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var<uniform> textureTransforms: TextureTransforms;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(3) @binding(0) var texture_sampler: sampler;

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    let matrix_ = textureTransforms.matrix_;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return srgb_to_linear(textureSample(texture, texture_sampler, in.uv));
}
