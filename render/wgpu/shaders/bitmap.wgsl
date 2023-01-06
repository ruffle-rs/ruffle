/// Shader used for drawing bitmap fills.

#import common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

#if use_push_constants == true
    var<push_constant> transforms: common::Transforms;
#else
    @group(1) @binding(0) var<uniform> transforms: common::Transforms;
#endif

@group(2) @binding(0) var<uniform> colorTransforms: common::ColorTransforms;

@group(3) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
@group(3) @binding(1) var texture: texture_2d<f32>;
@group(3) @binding(2) var texture_sampler: sampler;

@vertex
fn main_vertex(in: common::VertexInput) -> VertexOutput {
    let matrix_ = textureTransforms.matrix_;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, in.uv);
    // Texture is premultiplied by alpha.
    // Unmultiply alpha, apply color transform, remultiply alpha.
    if( color.a > 0.0 ) {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = color * colorTransforms.mult_color + colorTransforms.add_color;
        let alpha = clamp(color.a, 0.0, 1.0);
        color = vec4<f32>(color.rgb * alpha, alpha);
    }
    return color;
}
