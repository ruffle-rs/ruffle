/// Shader used for drawing bitmap fills.

#import common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

#if use_push_constants == true
    var<push_constant> transforms: common::Transforms;
    @group(1) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
    @group(1) @binding(1) var texture: texture_2d<f32>;
    @group(1) @binding(2) var texture_sampler: sampler;
#else
    @group(1) @binding(0) var<uniform> transforms: common::Transforms;
    @group(2) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
    @group(2) @binding(1) var texture: texture_2d<f32>;
    @group(2) @binding(2) var texture_sampler: sampler;
#endif

@vertex
fn main_vertex(in: common::VertexInput) -> VertexOutput {
    let matrix_ = textureTransforms.texture_matrix;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return common::srgb_to_linear(textureSample(texture, texture_sampler, in.uv));
}
